//! Reader-facing API presentation. Only this module creates trusted HTML;
//! every piece of specification Markdown is rendered with raw HTML disabled.
use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use super::publish::{code, inline, link};
use super::{Api, Location, Operation, Version};
use crate::error::{Error, Result};
use crate::util::html_escape;

pub(super) type Routes = BTreeMap<Location, String>;

#[derive(Default)]
pub(super) struct Content {
    pub markdown: String,
    pub html: String,
}

impl Content {
    fn md(&mut self, markdown: impl AsRef<str>) {
        let markdown = markdown.as_ref();
        let mut options = crate::pipeline::markdown::comrak_options();
        options.render.r#unsafe = false;
        self.html
            .push_str(&comrak::markdown_to_html(markdown, &options));
        self.markdown.push_str(markdown);
        self.markdown.push_str("\n\n");
    }

    fn append(&mut self, other: Self) {
        self.markdown.push_str(&other.markdown);
        self.html.push_str(&other.html);
    }

    fn table(&mut self, markdown: String) {
        // Protect wiki-link separators from GFM's table parser. Keep the
        // original Markdown for the AI exporter, which resolves wiki-links
        // before it emits the table.
        let mut prepared = String::new();
        let mut end = 0;
        for (start, close) in crate::pipeline::wikilinks::find_links(&markdown, &[]) {
            prepared.push_str(&markdown[end..start]);
            prepared.push_str(&markdown[start..close].replace('|', "\\|"));
            end = close;
        }
        prepared.push_str(&markdown[end..]);
        let mut rendered = Self::default();
        rendered.md(prepared);
        self.html.push_str(&rendered.html);
        self.markdown.push_str(&markdown);
        self.markdown.push_str("\n\n");
    }

    fn details(&mut self, label: &str, body: Self, class: &str) {
        self.markdown
            .push_str(&format!("\n**{}**\n\n{}\n", inline(label), body.markdown));
        self.html.push_str(&format!("<details class=\"details {class}\"><summary>{}</summary><div class=\"details-body\">{}</div></details>\n", html_escape(label), body.html));
    }

    fn source(&mut self, value: &Value) {
        let mut body = Self::default();
        body.md(code(value));
        self.details("View source definition", body, "api-source");
    }

    fn media(&mut self, items: Vec<(String, Self)>, id: &str, heading_level: usize) {
        if items.is_empty() {
            return;
        }
        if items.len() == 1 {
            let (name, content) = items.into_iter().next().unwrap();
            self.md(format!(
                "{} {}\n\n",
                "#".repeat(heading_level),
                inline(&name)
            ));
            self.append(content);
            return;
        }
        self.html.push_str("<div class=\"api-media\" data-api-media><div class=\"api-media-tabs\" role=\"tablist\" aria-label=\"Content type\" hidden>");
        for (index, (name, _)) in items.iter().enumerate() {
            self.html.push_str(&format!("<button type=\"button\" role=\"tab\" id=\"{}-tab-{index}\" aria-controls=\"{}-panel-{index}\" aria-selected=\"{}\" tabindex=\"{}\">{}</button>", html_escape(id), html_escape(id), index == 0, if index == 0 { 0 } else { -1 }, html_escape(name)));
        }
        self.html.push_str("</div>");
        for (index, (name, content)) in items.into_iter().enumerate() {
            self.markdown.push_str(&format!(
                "\n{} {}\n\n{}",
                "#".repeat(heading_level),
                inline(&name),
                content.markdown
            ));
            self.html.push_str(&format!("<section class=\"api-media-panel\" id=\"{}-panel-{index}\" data-api-panel><h{heading_level} class=\"api-media-label\">{}</h{heading_level}>{}</section>", html_escape(id), html_escape(&name), content.html));
        }
        self.html.push_str("</div>\n");
    }
}

fn value<'a>(api: &'a Api, location: &Location) -> Result<&'a Value> {
    api.graph
        .value(location)
        .map_err(|e| Error::General(e.to_string()))
}

fn resolve(api: &Api, location: &Location) -> Result<super::SourcedValue> {
    super::model::sourced(&api.graph, location).map_err(|e| Error::General(e.to_string()))
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}

fn literal(value: &Value) -> String {
    inline(&serde_json::to_string(value).unwrap())
}

fn example_code(value: &Value, media_type: &str) -> String {
    let Some(source) = value.as_str().filter(|_| !media_type.contains("json")) else {
        return code(value);
    };
    let language = if media_type.contains("yaml") {
        "yaml"
    } else if media_type.contains("xml") {
        "xml"
    } else {
        "text"
    };
    let fence = "`".repeat(3.max(source.split(|c| c != '`').map(str::len).max().unwrap_or(0) + 1));
    format!("\n{fence}{language}\n{source}\n{fence}\n\n")
}

fn reference_label(api: &Api, location: &Location, routes: &Routes) -> Option<String> {
    let edge = api.graph.reference_at(location)?;
    let name = edge
        .target
        .pointer
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.replace("~1", "/").replace("~0", "~"))
        .unwrap_or_else(|| {
            edge.target
                .file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        });
    Some(
        routes
            .get(&edge.target)
            .map(|route| link(route, &name))
            .unwrap_or_else(|| inline(&name)),
    )
}

fn schema_type(api: &Api, schema: &Value, source: &Location, routes: &Routes) -> String {
    if let Some(label) = reference_label(api, source, routes) {
        return label;
    }
    if schema == &Value::Bool(true) {
        return "Any value".into();
    }
    if schema == &Value::Bool(false) {
        return "No value allowed".into();
    }
    let mut kind = match schema.get("type") {
        Some(Value::String(kind)) if kind == "array" => schema
            .get("items")
            .map(|items| {
                format!(
                    "array of {}",
                    schema_type(api, items, &source.child("items"), routes)
                )
            })
            .unwrap_or_else(|| "array".into()),
        Some(Value::String(kind)) => inline(kind),
        Some(Value::Array(types)) => types
            .iter()
            .filter_map(Value::as_str)
            .map(inline)
            .collect::<Vec<_>>()
            .join(" or "),
        _ if schema.get("oneOf").is_some() => "One of the listed schemas".into(),
        _ if schema.get("anyOf").is_some() => "Any of the listed schemas".into(),
        _ if schema.get("allOf").is_some() => "All of the listed schemas".into(),
        _ if schema.get("properties").is_some() => "object".into(),
        _ => "Any type".into(),
    };
    if api.graph.version == Version::V3_0
        && schema.get("type").is_some_and(Value::is_string)
        && schema.get("nullable").and_then(Value::as_bool) == Some(true)
    {
        kind.push_str(" or null");
    }
    kind
}

fn constraints(schema: &Value, version: Version) -> Vec<String> {
    let mut result = Vec::new();
    for (key, label) in [
        ("format", "Format"),
        ("default", "Default"),
        ("const", "Constant"),
        ("minimum", "Minimum"),
        ("maximum", "Maximum"),
        ("exclusiveMinimum", "Exclusive minimum"),
        ("exclusiveMaximum", "Exclusive maximum"),
        ("multipleOf", "Multiple of"),
        ("minLength", "Minimum length"),
        ("maxLength", "Maximum length"),
        ("pattern", "Pattern"),
        ("minItems", "Minimum items"),
        ("maxItems", "Maximum items"),
        ("uniqueItems", "Unique items"),
        ("minProperties", "Minimum properties"),
        ("maxProperties", "Maximum properties"),
    ] {
        if key == "const" && version == Version::V3_0 {
            continue;
        }
        if let Some(value) = schema.get(key) {
            result.push(format!("{label}: {}", literal(value)));
        }
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        result.push(format!(
            "Allowed values: {}",
            values.iter().map(literal).collect::<Vec<_>>().join(", ")
        ));
    }
    if let Some(names) = schema.get("required").and_then(Value::as_array)
        && !names.is_empty()
    {
        result.push(format!(
            "Required properties: {}",
            names.iter().map(literal).collect::<Vec<_>>().join(", ")
        ));
    }
    for (key, label) in [
        ("readOnly", "Read only"),
        ("writeOnly", "Write only"),
        ("deprecated", "Deprecated"),
    ] {
        if schema.get(key).and_then(Value::as_bool) == Some(true) {
            result.push(label.into());
        }
    }
    result
}

fn schema_fields(
    api: &Api,
    schema: &Value,
    source: &Location,
    routes: &Routes,
    depth: usize,
    visiting: &mut BTreeSet<Location>,
) -> Result<Content> {
    let mut out = Content::default();
    if depth >= 5 || !visiting.insert(source.clone()) {
        out.md(
            "Continue with the linked schema definition to explore this nested or recursive value.",
        );
        return Ok(out);
    }
    out.md(format!(
        "**Type:** {}\n\n",
        schema_type(api, schema, source, routes)
    ));
    if let Some(edge) = api.graph.reference_at(source) {
        if !visiting.contains(&edge.target) {
            out.append(schema_fields(
                api,
                value(api, &edge.target)?,
                &edge.target,
                routes,
                depth + 1,
                visiting,
            )?);
        } else {
            out.md("Recursive reference; follow the schema link for its full definition.");
        }
        if api.graph.version == Version::V3_0 {
            visiting.remove(source);
            return Ok(out);
        }
    }
    let description = text(schema, "description");
    if !description.is_empty() {
        out.md(description);
    }
    let constraints = constraints(schema, api.graph.version);
    if !constraints.is_empty() {
        out.md(constraints
            .iter()
            .map(|s| format!("- {s}\n"))
            .collect::<String>());
    }
    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        let mut table = String::from(
            "| Property | Type | Required | Description and constraints |\n| --- | --- | --- | --- |\n",
        );
        for (name, property) in properties {
            let location = source.child("properties").child(name);
            let required = schema
                .get("required")
                .and_then(Value::as_array)
                .is_some_and(|list| list.iter().any(|v| v.as_str() == Some(name)));
            let ignores_siblings =
                api.graph.version == Version::V3_0 && api.graph.reference_at(&location).is_some();
            let mut details = if ignores_siblings {
                Vec::new()
            } else {
                vec![inline(&text(property, "description"))]
            };
            details.extend(constraints_for_property(api, property, &location)?);
            table.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                inline(name),
                schema_type(api, property, &location, routes),
                if required { "Yes" } else { "No" },
                details
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
        out.table(table);
        for (name, property) in properties {
            let location = source.child("properties").child(name);
            if api.graph.version == Version::V3_0 && api.graph.reference_at(&location).is_some() {
                continue;
            }
            if property.get("properties").is_some()
                || property.get("items").is_some()
                || property.get("additionalProperties").is_some()
                || ["allOf", "oneOf", "anyOf"]
                    .iter()
                    .any(|k| property.get(k).is_some())
            {
                let body = schema_fields(api, property, &location, routes, depth + 1, visiting)?;
                out.details(&format!("Explore {name}"), body, "api-schema-branch");
            }
        }
    }
    if let Some(items) = schema.get("items") {
        out.md("**Array items**");
        out.append(schema_fields(
            api,
            items,
            &source.child("items"),
            routes,
            depth + 1,
            visiting,
        )?);
    }
    for (key, label) in [
        ("allOf", "All of — every schema applies"),
        ("oneOf", "One of — exactly one schema applies"),
        ("anyOf", "Any of — one or more schemas apply"),
    ] {
        if let Some(options) = schema.get(key).and_then(Value::as_array) {
            out.md(format!("**{label}**"));
            for (index, option) in options.iter().enumerate() {
                let body = schema_fields(
                    api,
                    option,
                    &source.child(key).child(&index.to_string()),
                    routes,
                    depth + 1,
                    visiting,
                )?;
                out.details(&format!("Schema {}", index + 1), body, "api-schema-branch");
            }
        }
    }
    if let Some(additional) = schema.get("additionalProperties") {
        match additional.as_bool() {
            Some(false) => out.md("Additional properties are not allowed."),
            Some(true) => out.md("Additional properties are allowed."),
            None => {
                let body = schema_fields(
                    api,
                    additional,
                    &source.child("additionalProperties"),
                    routes,
                    depth + 1,
                    visiting,
                )?;
                out.details("Additional property values", body, "api-schema-branch");
            }
        }
    }
    visiting.remove(source);
    Ok(out)
}

fn constraints_for_property(api: &Api, schema: &Value, location: &Location) -> Result<Vec<String>> {
    let mut result = Vec::new();
    // Referenced types stay linked, but their scalar constraints should still
    // be visible without making readers visit another page for an enum/default.
    let mut current = location;
    let mut schema = schema;
    let mut seen = BTreeSet::new();
    while seen.insert(current.clone()) {
        let reference = api.graph.reference_at(current);
        if api.graph.version == Version::V3_1 || reference.is_none() {
            result.extend(constraints(schema, api.graph.version));
        }
        let Some(edge) = reference else { break };
        current = &edge.target;
        schema = value(api, current)?;
    }
    Ok(result)
}

fn servers(out: &mut Content, values: &[Value]) {
    out.md("## Servers");
    for server in values {
        out.md(format!(
            "- **{}** {}\n",
            inline(&text(server, "url")),
            inline(&text(server, "description"))
        ));
        if let Some(variables) = server.get("variables").and_then(Value::as_object) {
            let mut table = String::from(
                "| Variable | Default | Allowed values | Description |\n| --- | --- | --- | --- |\n",
            );
            for (name, variable) in variables {
                table.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    inline(name),
                    variable.get("default").map(literal).unwrap_or_default(),
                    variable.get("enum").map(literal).unwrap_or_default(),
                    inline(&text(variable, "description"))
                ));
            }
            out.table(table);
        }
    }
}

fn security(api: &Api, requirements: &[Value], out: &mut Content) -> Result<()> {
    out.md("## Authentication");
    if requirements.is_empty() {
        out.md("No authentication required.");
        return Ok(());
    }
    for (index, requirement) in requirements.iter().enumerate() {
        if requirements.len() > 1 {
            out.md(format!(
                "**Option {}** — choose one authentication option.\n\n",
                index + 1
            ));
        }
        if let Some(schemes) = requirement.as_object() {
            if schemes.is_empty() {
                out.md("No authentication required for this option.");
            }
            if schemes.len() > 1 {
                out.md("All of the following credentials are required together.");
            }
            for (name, scopes) in schemes {
                let location = api
                    .graph
                    .root
                    .child("components")
                    .child("securitySchemes")
                    .child(name);
                let scheme = if api.graph.value(&location).is_ok() {
                    Some(resolve(api, &location)?)
                } else {
                    None
                };
                let description = match scheme.as_ref().map(|s| &s.value) {
                    Some(s)
                        if text(s, "type") == "http"
                            && text(s, "scheme").eq_ignore_ascii_case("bearer") =>
                    {
                        "Bearer token in the Authorization header.".to_owned()
                    }
                    Some(s) if text(s, "type") == "http" => {
                        format!("HTTP {} authentication.", inline(&text(s, "scheme")))
                    }
                    Some(s) if text(s, "type") == "apiKey" => format!(
                        "API key named {} in {}.",
                        inline(&text(s, "name")),
                        inline(&text(s, "in"))
                    ),
                    Some(s) if text(s, "type") == "oauth2" => "OAuth 2.0 credentials.".into(),
                    Some(s) if text(s, "type") == "openIdConnect" => {
                        "OpenID Connect credentials.".into()
                    }
                    Some(s) => inline(&text(s, "type")),
                    None => {
                        "Security scheme is not defined in components; consult the API provider."
                            .into()
                    }
                };
                out.md(format!("**{}** — {description}\n\n", inline(name)));
                if let Some(scheme) = scheme {
                    let description = text(&scheme.value, "description");
                    if !description.is_empty() {
                        out.md(description);
                    }
                }
                if let Some(scopes) = scopes.as_array().filter(|s| !s.is_empty()) {
                    out.md(format!(
                        "Required scopes: {}",
                        scopes.iter().map(literal).collect::<Vec<_>>().join(", ")
                    ));
                }
            }
        }
    }
    Ok(())
}

fn examples(
    api: &Api,
    media: &Value,
    source: &Location,
    media_type: &str,
    out: &mut Content,
) -> Result<()> {
    if let Some(example) = media.get("example") {
        out.md("**Example**");
        out.md(example_code(example, media_type));
    }
    if let Some(examples) = media.get("examples").and_then(Value::as_object) {
        for name in examples.keys() {
            let example = resolve(api, &source.child("examples").child(name))?;
            let title = example
                .value
                .get("summary")
                .and_then(Value::as_str)
                .unwrap_or(name);
            out.md(format!("**Example: {}**", inline(title)));
            let description = text(&example.value, "description");
            if !description.is_empty() {
                out.md(description);
            }
            if let Some(value) = example.value.get("value") {
                out.md(example_code(value, media_type));
            }
            if let Some(url) = example.value.get("externalValue").and_then(Value::as_str) {
                out.md(format!("External example: {}", inline(url)));
            }
        }
    }
    Ok(())
}

fn media(
    api: &Api,
    object: &Value,
    source: &Location,
    routes: &Routes,
    out: &mut Content,
    id: &str,
    heading_level: usize,
) -> Result<()> {
    let mut items = Vec::new();
    if let Some(types) = object.get("content").and_then(Value::as_object) {
        for (name, media) in types {
            let location = source.child("content").child(name);
            let mut body = Content::default();
            if let Some(schema) = media.get("schema") {
                body.append(schema_fields(
                    api,
                    schema,
                    &location.child("schema"),
                    routes,
                    0,
                    &mut BTreeSet::new(),
                )?);
                if !(api.graph.version == Version::V3_0
                    && api.graph.reference_at(&location.child("schema")).is_some())
                    && let Some(example) = schema.get("example")
                {
                    body.md("**Schema example**");
                    body.md(example_code(example, name));
                }
            }
            examples(api, media, &location, name, &mut body)?;
            if media.get("schema").is_none()
                && media.get("example").is_none()
                && media.get("examples").is_none()
            {
                body.md("No schema or example supplied for this content type.");
            }
            items.push((name.clone(), body));
        }
    }
    out.media(items, id, heading_level);
    Ok(())
}

fn headers(
    api: &Api,
    object: &Value,
    source: &Location,
    routes: &Routes,
    out: &mut Content,
) -> Result<()> {
    if let Some(headers) = object.get("headers").and_then(Value::as_object) {
        out.md("**Response headers**");
        let mut table = String::from("| Header | Type | Description |\n| --- | --- | --- |\n");
        for name in headers.keys() {
            let header = resolve(api, &source.child("headers").child(name))?;
            let schema = header.value.get("schema").cloned().unwrap_or(json!({}));
            table.push_str(&format!(
                "| {} | {} | {} |\n",
                inline(name),
                schema_type(api, &schema, &header.source.child("schema"), routes),
                inline(&text(&header.value, "description"))
            ));
        }
        out.table(table);
    }
    Ok(())
}

pub(super) fn operation(
    api: &Api,
    operation: &Operation,
    routes: &Routes,
    slug: &str,
) -> Result<Content> {
    let mut out = Content::default();
    let title = operation
        .summary
        .clone()
        .unwrap_or_else(|| format!("{} {}", operation.method, operation.path));
    out.md(format!("# {}", inline(&title)));
    out.markdown.push_str(&format!(
        "**{} {}**\n\n",
        operation.method,
        inline(&operation.path)
    ));
    out.html.push_str(&format!("<div class=\"api-operation\"><span class=\"api-method api-method-{}\">{}</span><code class=\"api-path\">{}</code>{}</div>", operation.method.to_ascii_lowercase(), html_escape(&operation.method), html_escape(&operation.path), if operation.deprecated { "<span class=\"api-deprecated\">Deprecated</span>" } else { "" }));
    if operation.deprecated {
        out.markdown.push_str("**Deprecated.**\n\n");
    }
    if !operation.tags.is_empty() {
        out.md(format!(
            "Tags: {}",
            operation
                .tags
                .iter()
                .map(|s| inline(s))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let Some(description) = &operation.description {
        out.md(description);
    }
    security(api, &operation.security, &mut out)?;
    servers(&mut out, &operation.servers);
    if !operation.parameters.is_empty() {
        out.md("## Parameters");
        let mut table = String::from(
            "| Name | Location | Type | Required | Description and constraints |\n| --- | --- | --- | --- | --- |\n",
        );
        for parameter in &operation.parameters {
            let schema = parameter
                .definition
                .value
                .get("schema")
                .cloned()
                .unwrap_or(json!({}));
            let location = parameter.definition.source.child("schema");
            let mut description = vec![inline(&text(&parameter.definition.value, "description"))];
            description.extend(constraints_for_property(api, &schema, &location)?);
            table.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                inline(&parameter.name),
                parameter.location,
                schema_type(api, &schema, &location, routes),
                if parameter.required { "Yes" } else { "No" },
                description
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
        out.table(table);
        for (index, parameter) in operation.parameters.iter().enumerate() {
            if parameter.definition.value.get("content").is_some() {
                out.md(format!("### {} content", inline(&parameter.name)));
                media(
                    api,
                    &parameter.definition.value,
                    &parameter.definition.source,
                    routes,
                    &mut out,
                    &format!("{slug}-parameter-{index}"),
                    4,
                )?;
            }
        }
    }
    if let Some(body) = &operation.request_body {
        out.md("## Request body");
        out.md(
            if body.value.get("required").and_then(Value::as_bool) == Some(true) {
                "**Required body.**"
            } else {
                "Optional body."
            },
        );
        let description = text(&body.value, "description");
        if !description.is_empty() {
            out.md(description);
        }
        media(
            api,
            &body.value,
            &body.source,
            routes,
            &mut out,
            &format!("{slug}-request"),
            3,
        )?;
    }
    out.md("## Responses");
    for (status, response) in &operation.responses {
        out.md(format!("### {}", inline(status)));
        out.md(text(&response.value, "description"));
        headers(api, &response.value, &response.source, routes, &mut out)?;
        if response
            .value
            .get("content")
            .and_then(Value::as_object)
            .is_none_or(|m| m.is_empty())
        {
            out.md("No response body documented.");
        }
        media(
            api,
            &response.value,
            &response.source,
            routes,
            &mut out,
            &format!("{slug}-response-{status}"),
            4,
        )?;
    }
    // Preserve normalized values too: source Reference Objects alone would hide
    // inherited parameters and externally defined request/response details.
    out.source(&json!({
        "operation": value(api, &operation.source)?, "effectiveServers": operation.servers,
        "effectiveSecurity": operation.security,
        "effectiveParameters": operation.parameters.iter().map(|p| &p.definition.value).collect::<Vec<_>>(),
        "requestBody": operation.request_body.as_ref().map(|b| &b.value),
        "responses": operation.responses.iter().map(|(s,r)| (s,&r.value)).collect::<BTreeMap<_,_>>()
    }));
    Ok(out)
}

pub(super) fn schema(api: &Api, name: &str, source: &Location, routes: &Routes) -> Result<Content> {
    let mut out = Content::default();
    out.md(format!("# {}", inline(name)));
    out.md("## Properties");
    out.append(schema_fields(
        api,
        value(api, source)?,
        source,
        routes,
        0,
        &mut BTreeSet::new(),
    )?);
    let mut referenced = Vec::new();
    let mut current = source.clone();
    let mut seen = BTreeSet::from([current.clone()]);
    while let Some(edge) = api.graph.reference_at(&current) {
        if !seen.insert(edge.target.clone()) {
            break;
        }
        referenced.push(value(api, &edge.target)?.clone());
        current = edge.target.clone();
    }
    if referenced.is_empty() {
        out.source(value(api, source)?);
    } else {
        out.source(&json!({"schema": value(api, source)?, "referencedDefinitions": referenced}));
    }
    Ok(out)
}

pub(super) fn overview(
    api: &Api,
    operations: &[(String, &Operation)],
    schemas: &[(String, Location, String)],
) -> Result<Content> {
    let mut out = Content::default();
    out.md(format!("# {}", inline(&api.title)));
    if let Some(description) = &api.description {
        out.md(description);
    }
    out.md(format!("**API version:** {}", inline(&api.api_version)));
    let root = value(api, &api.graph.root)?;
    servers(
        &mut out,
        root.get("servers")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[json!({"url": "/"})]),
    );
    security(
        api,
        root.get("security")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default(),
        &mut out,
    )?;
    out.md("## Operations");
    // Declared tags follow author order; undeclared tags follow lexical order.
    // An operation belongs to every tag it names, but only once per group.
    let groups = tag_groups(api)?;
    if groups.is_empty() {
        operation_table(
            &mut out,
            operations
                .iter()
                .map(|(slug, operation)| (slug.as_str(), *operation)),
        );
    } else {
        for (name, description) in groups {
            out.md(format!("### {}", inline(name)));
            if let Some(description) = description {
                out.md(description);
            }
            operation_table(
                &mut out,
                operations
                    .iter()
                    .filter(|(_, operation)| operation.tags.iter().any(|tag| tag == name))
                    .map(|(slug, operation)| (slug.as_str(), *operation)),
            );
        }
        let untagged: Vec<_> = operations
            .iter()
            .filter(|(_, operation)| operation.tags.is_empty())
            .map(|(slug, operation)| (slug.as_str(), *operation))
            .collect();
        if !untagged.is_empty() {
            out.md("### Untagged operations");
            operation_table(&mut out, untagged);
        }
    }
    out.md("## Schemas");
    out.md(schemas
        .iter()
        .map(|(name, _, slug)| format!("- {}\n", link(slug, name)))
        .collect::<String>());
    out.source(root);
    Ok(out)
}

fn operation_table<'a>(
    out: &mut Content,
    operations: impl IntoIterator<Item = (&'a str, &'a Operation)>,
) {
    let mut table = String::from("| Method | Path | Operation |\n| --- | --- | --- |\n");
    let mut count = 0;
    for (slug, operation) in operations {
        let title = operation
            .summary
            .as_deref()
            .or(operation.operation_id.as_deref())
            .unwrap_or(&operation.path);
        table.push_str(&format!(
            "| **{}** | {} | {} |\n",
            operation.method,
            inline(&operation.path),
            link(slug, title)
        ));
        count += 1;
    }
    if count == 0 {
        out.md("No operations.");
    } else {
        out.table(table);
    }
}

/// Shared overview/sidebar tag ordering, retaining specification descriptions.
pub(super) fn tag_groups(api: &Api) -> Result<Vec<(&str, Option<&str>)>> {
    let root = value(api, &api.graph.root)?;
    let mut groups: Vec<_> = root
        .get("tags")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|tag| {
            (
                tag["name"].as_str().unwrap(),
                tag.get("description").and_then(Value::as_str),
            )
        })
        .collect();
    let declared: BTreeSet<_> = groups.iter().map(|(name, _)| *name).collect();
    let extra: BTreeSet<_> = api
        .operations
        .iter()
        .flat_map(|operation| &operation.tags)
        .map(String::as_str)
        .filter(|name| !declared.contains(name))
        .collect();
    groups.extend(extra.into_iter().map(|name| (name, None)));
    Ok(groups)
}
