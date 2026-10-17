use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::graph::{DocumentGraph, Location, METHODS, OpenApiError, Version};

/// A value together with its resolved source. Reference-site description and
/// summary overrides in 3.1 are applied without losing their origin.
#[derive(Debug, Clone)]
pub struct SourcedValue {
    pub source: Location,
    pub reference_origin: Option<Location>,
    pub value: Value,
}

#[derive(Debug)]
pub struct Parameter {
    pub name: String,
    pub location: String,
    pub required: bool,
    pub definition: SourcedValue,
}

/// Effective request metadata, independent of templates and URL planning.
/// Schemas remain original values/graph edges; no schema dialect is flattened.
#[derive(Debug)]
pub struct Operation {
    pub source: Location,
    pub operation_id: Option<String>,
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub parameters: Vec<Parameter>,
    pub servers: Vec<Value>,
    pub servers_source: Option<Location>,
    pub security: Vec<Value>,
    pub security_source: Option<Location>,
    pub request_body: Option<SourcedValue>,
    pub responses: BTreeMap<String, SourcedValue>,
}

#[derive(Debug, Clone)]
pub struct Warning {
    pub location: Location,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug)]
pub struct Api {
    pub title: String,
    pub api_version: String,
    pub description: Option<String>,
    pub operations: Vec<Operation>,
    pub schemas: BTreeMap<String, Location>,
    pub warnings: Vec<Warning>,
    pub graph: DocumentGraph,
}

impl Api {
    pub fn from_graph(graph: DocumentGraph) -> Result<Self, OpenApiError> {
        let root = &graph.root;
        object(graph.value(root)?, root)?;
        let info_location = root.child("info");
        let info = object(graph.value(&info_location)?, &info_location)?;
        let title = required_string(info, "title", &info_location)?;
        let api_version = required_string(info, "version", &info_location)?;
        let description = optional_string(info, "description", &info_location)?;
        let root_value = graph.value(root)?;
        let mut operations = Vec::new();
        let mut warnings = Vec::new();
        if root_value.get("webhooks").is_some() {
            warnings.push(Warning {
                location: root.child("webhooks"),
                code: "openapi-webhooks-unrendered",
                message:
                    "webhook references are loaded, but webhook operations are not normalized yet"
                        .into(),
            });
        }
        let mut ids = BTreeMap::new();
        let paths = root.child("paths");
        if graph.version == Version::V3_0 && root_value.get("paths").is_none() {
            return Err(OpenApiError::at(&paths, "paths is required in OpenAPI 3.0"));
        }
        if let Some(paths_value) = root_value.get("paths") {
            for (path, _) in object(paths_value, &paths)? {
                if path.starts_with("x-") {
                    continue;
                }
                if !path.starts_with('/') {
                    return Err(OpenApiError::at(
                        &paths.child(path),
                        "API paths must start with /",
                    ));
                }
                let item_location = graph.dereference(&paths.child(path))?;
                let item = object(graph.value(&item_location)?, &item_location)?;
                let path_parameters = parameters(
                    &graph,
                    &item_location.child("parameters"),
                    item.get("parameters"),
                )?;
                for method in METHODS {
                    let Some(value) = item.get(*method) else {
                        continue;
                    };
                    let source = item_location.child(method);
                    let operation = object(value, &source)?;
                    if operation.contains_key("callbacks") {
                        warnings.push(Warning {
                            location: source.child("callbacks"),
                            code: "openapi-callbacks-unrendered",
                            message: "callback references are loaded, but callback operations are not normalized yet".into(),
                        });
                    }
                    let operation_id = optional_string(operation, "operationId", &source)?;
                    if let Some(id) = &operation_id {
                        if id.is_empty() {
                            return Err(OpenApiError::at(
                                &source.child("operationId"),
                                "operationId must not be empty",
                            ));
                        }
                        if let Some(previous) = ids.insert(id.clone(), source.clone()) {
                            return Err(OpenApiError::at(
                                &source.child("operationId"),
                                format!("duplicate operationId {id:?}; first used at {previous}"),
                            ));
                        }
                    }
                    let mut effective_parameters: BTreeMap<_, _> = path_parameters
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect();
                    effective_parameters.extend(parameters(
                        &graph,
                        &source.child("parameters"),
                        operation.get("parameters"),
                    )?);
                    let mut normalized_parameters = Vec::new();
                    for ((location, name), definition) in effective_parameters {
                        normalized_parameters.push(Parameter {
                            name,
                            location,
                            required: definition
                                .value
                                .get("required")
                                .and_then(Value::as_bool)
                                .unwrap_or(false),
                            definition,
                        });
                    }
                    let server_source = if operation.contains_key("servers") {
                        Some(source.child("servers"))
                    } else if item.contains_key("servers") {
                        Some(item_location.child("servers"))
                    } else if root_value.get("servers").is_some() {
                        Some(root.child("servers"))
                    } else {
                        None
                    };
                    let mut servers = effective_array(&graph, server_source.as_ref())?;
                    if servers.is_empty() {
                        servers.push(json!({"url": "/"}));
                    }
                    for (index, server) in servers.iter().enumerate() {
                        let loc = server_source
                            .as_ref()
                            .map(|s| s.child(&index.to_string()))
                            .unwrap_or_else(|| root.clone());
                        required_string(object(server, &loc)?, "url", &loc)?;
                    }
                    let security_source = if operation.contains_key("security") {
                        Some(source.child("security"))
                    } else if root_value.get("security").is_some() {
                        Some(root.child("security"))
                    } else {
                        None
                    };
                    let security = effective_array(&graph, security_source.as_ref())?;
                    for (index, requirement) in security.iter().enumerate() {
                        let loc = security_source.as_ref().unwrap().child(&index.to_string());
                        for (name, scopes) in object(requirement, &loc)? {
                            string_array(scopes, &loc.child(name))?;
                        }
                    }
                    let request_body = operation
                        .get("requestBody")
                        .map(|_| sourced(&graph, &source.child("requestBody")))
                        .transpose()?;
                    if let Some(body) = &request_body {
                        let body_object = object(&body.value, &body.source)?;
                        let content = body_object.get("content").ok_or_else(|| {
                            OpenApiError::at(
                                &body.source.child("content"),
                                "request body content is required",
                            )
                        })?;
                        object(content, &body.source.child("content"))?;
                        optional_bool(body_object, "required", &body.source)?;
                    }
                    let mut responses = BTreeMap::new();
                    if let Some(value) = operation.get("responses") {
                        for (status, _) in object(value, &source.child("responses"))? {
                            if status.starts_with("x-") {
                                continue;
                            }
                            if !valid_status(status) {
                                return Err(OpenApiError::at(
                                    &source.child("responses").child(status),
                                    "expected default, an HTTP status code, or a status range such as 2XX",
                                ));
                            }
                            let response =
                                sourced(&graph, &source.child("responses").child(status))?;
                            required_string(
                                object(&response.value, &response.source)?,
                                "description",
                                &response.source,
                            )?;
                            responses.insert(status.clone(), response);
                        }
                    } else if graph.version == Version::V3_0 {
                        return Err(OpenApiError::at(
                            &source.child("responses"),
                            "responses is required in OpenAPI 3.0",
                        ));
                    }
                    if operation.contains_key("responses") && responses.is_empty() {
                        return Err(OpenApiError::at(
                            &source.child("responses"),
                            "responses must contain at least one response",
                        ));
                    }
                    let tags = operation
                        .get("tags")
                        .map(|v| string_array(v, &source.child("tags")))
                        .transpose()?
                        .unwrap_or_default();
                    operations.push(Operation {
                        source: source.clone(),
                        operation_id,
                        method: method.to_uppercase(),
                        path: path.clone(),
                        summary: optional_string(operation, "summary", &source)?,
                        description: optional_string(operation, "description", &source)?,
                        tags,
                        deprecated: optional_bool(operation, "deprecated", &source)?
                            .unwrap_or(false),
                        parameters: normalized_parameters,
                        servers,
                        servers_source: server_source,
                        security,
                        security_source,
                        request_body,
                        responses,
                    });
                }
            }
        }
        let mut schemas = BTreeMap::new();
        if let Some(components) = root_value.get("components") {
            let location = root.child("components");
            let components = object(components, &location)?;
            if let Some(values) = components.get("schemas") {
                for name in object(values, &location.child("schemas"))?.keys() {
                    schemas.insert(name.clone(), location.child("schemas").child(name));
                }
            }
        }
        Ok(Self {
            title,
            api_version,
            description,
            operations,
            schemas,
            warnings,
            graph,
        })
    }
}

pub(super) fn sourced(
    graph: &DocumentGraph,
    origin: &Location,
) -> Result<SourcedValue, OpenApiError> {
    let source = graph.dereference(origin)?;
    let mut value = graph.value(&source)?.clone();
    object(&value, &source)?;
    // Apply outer Reference Object overrides last, retaining correct 3.0 behavior.
    if graph.version == Version::V3_1 && source != *origin {
        let mut current = origin.clone();
        let mut overrides = Vec::new();
        while current != source {
            overrides.push(graph.value(&current)?);
            current = graph
                .reference_at(&current)
                .ok_or_else(|| OpenApiError::at(&current, "missing reference edge"))?
                .target
                .clone();
        }
        for node in overrides.into_iter().rev() {
            for key in ["summary", "description"] {
                if let Some(override_value) = node.get(key) {
                    if !override_value.is_string() {
                        return Err(OpenApiError::at(&origin.child(key), "expected a string"));
                    }
                    value[key] = override_value.clone();
                }
            }
        }
    }
    Ok(SourcedValue {
        reference_origin: (source != *origin).then(|| origin.clone()),
        source,
        value,
    })
}

type ParameterMap = BTreeMap<(String, String), SourcedValue>;

fn parameters(
    graph: &DocumentGraph,
    location: &Location,
    values: Option<&Value>,
) -> Result<ParameterMap, OpenApiError> {
    let mut result = BTreeMap::new();
    let Some(values) = values else {
        return Ok(result);
    };
    let values = values
        .as_array()
        .ok_or_else(|| OpenApiError::at(location, "parameters must be an array"))?;
    for index in 0..values.len() {
        let origin = location.child(&index.to_string());
        let definition = sourced(graph, &origin)?;
        let value = object(&definition.value, &definition.source)?;
        let name = required_string(value, "name", &definition.source)?;
        let parameter_location = required_string(value, "in", &definition.source)?;
        if !["path", "query", "header", "cookie"].contains(&parameter_location.as_str()) {
            return Err(OpenApiError::at(
                &definition.source.child("in"),
                "expected path, query, header or cookie",
            ));
        }
        let required = optional_bool(value, "required", &definition.source)?.unwrap_or(false);
        if parameter_location == "path" && !required {
            return Err(OpenApiError::at(
                &definition.source.child("required"),
                "path parameters must declare required: true",
            ));
        }
        if result
            .insert((parameter_location, name), definition)
            .is_some()
        {
            return Err(OpenApiError::at(
                &origin,
                "duplicate parameter (name, in) in the same list",
            ));
        }
    }
    Ok(result)
}

fn object<'a>(
    value: &'a Value,
    location: &Location,
) -> Result<&'a serde_json::Map<String, Value>, OpenApiError> {
    value
        .as_object()
        .ok_or_else(|| OpenApiError::at(location, "expected an object"))
}

fn required_string(
    value: &serde_json::Map<String, Value>,
    key: &str,
    location: &Location,
) -> Result<String, OpenApiError> {
    optional_string(value, key, location)?
        .ok_or_else(|| OpenApiError::at(&location.child(key), "required string is missing"))
}

fn optional_string(
    value: &serde_json::Map<String, Value>,
    key: &str,
    location: &Location,
) -> Result<Option<String>, OpenApiError> {
    value
        .get(key)
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| OpenApiError::at(&location.child(key), "expected a string"))
        })
        .transpose()
}

fn optional_bool(
    value: &serde_json::Map<String, Value>,
    key: &str,
    location: &Location,
) -> Result<Option<bool>, OpenApiError> {
    value
        .get(key)
        .map(|v| {
            v.as_bool()
                .ok_or_else(|| OpenApiError::at(&location.child(key), "expected a boolean"))
        })
        .transpose()
}

fn string_array(value: &Value, location: &Location) -> Result<Vec<String>, OpenApiError> {
    value
        .as_array()
        .ok_or_else(|| OpenApiError::at(location, "expected an array"))?
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.as_str().map(str::to_owned).ok_or_else(|| {
                OpenApiError::at(&location.child(&i.to_string()), "expected a string")
            })
        })
        .collect()
}

fn effective_array(
    graph: &DocumentGraph,
    source: Option<&Location>,
) -> Result<Vec<Value>, OpenApiError> {
    source
        .map(|source| {
            graph
                .value(source)?
                .as_array()
                .cloned()
                .ok_or_else(|| OpenApiError::at(source, "expected an array"))
        })
        .transpose()
        .map(Option::unwrap_or_default)
}

fn valid_status(status: &str) -> bool {
    let b = status.as_bytes();
    b.len() == 3
        && (b'1'..=b'5').contains(&b[0])
        && (b[1..].iter().all(u8::is_ascii_digit) || &b[1..] == b"XX")
        || status == "default"
}
