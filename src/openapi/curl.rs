//! Static request choices and POSIX command construction, independent of the UI.
use super::{Api, Location, Operation, SourcedValue};
use crate::error::{Error, Result};
use serde::Serialize;
use serde_json::Value;

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}
fn resolve(api: &Api, location: &Location) -> Result<SourcedValue> {
    super::model::sourced(&api.graph, location).map_err(|e| Error::General(e.to_string()))
}

#[derive(Serialize)]
pub(super) struct Server {
    pub label: String,
    pub base: String,
    pub notes: Vec<String>,
}
#[derive(Default, Serialize)]
pub(super) struct Authentication {
    pub label: String,
    pub args: Vec<String>,
    pub query: Vec<String>,
    pub cookies: Vec<String>,
    pub notes: Vec<String>,
}
#[derive(Serialize)]
pub(super) struct Body {
    pub media: String,
    pub label: String,
    pub args: Vec<String>,
    pub notes: Vec<String>,
}
#[derive(Serialize)]
pub(super) struct Choices {
    pub prefix: String,
    pub path: String,
    pub args: Vec<String>,
    pub query: Vec<String>,
    pub cookies: Vec<String>,
    pub notes: Vec<String>,
    pub servers: Vec<Server>,
    pub authentication: Vec<Authentication>,
    pub bodies: Vec<Body>,
}
impl Choices {
    pub fn command(&self, server: usize, authentication: usize, body: usize) -> String {
        let auth = &self.authentication[authentication];
        let mut args = self.args.clone();
        args.extend(auth.args.clone());
        let cookies: Vec<_> = self.cookies.iter().chain(&auth.cookies).cloned().collect();
        if !cookies.is_empty() {
            args.push(format!("--cookie {}", shell_quote(&cookies.join("; "))));
        }
        args.extend(self.bodies[body].args.clone());
        let base = &self.servers[server].base;
        let (base, existing_query) = base.split_once('?').unwrap_or((base, ""));
        let mut query: Vec<_> = self.query.iter().chain(&auth.query).cloned().collect();
        if !existing_query.is_empty() {
            query.insert(0, existing_query.to_owned());
        }
        let mut url = format!(
            "{}/{}",
            base.trim_end_matches('/'),
            self.path.trim_start_matches('/')
        );
        if !query.is_empty() {
            url.push('?');
            url.push_str(&query.join("&"));
        }
        args.push(format!("--url {}", shell_quote(&url)));
        format!(
            "{}{}",
            self.prefix,
            args.iter()
                .map(|arg| format!(" \\\n  {arg}"))
                .collect::<String>()
        )
    }
}

// Single quotes protect all specification text from shell expansion.
pub(super) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn url_component(value: &str) -> String {
    value
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

/// Serialize scalar and flat container values; nested values require author input.
pub(super) fn serialize_parameter(
    name: &str,
    value: &Value,
    location: &str,
    style: &str,
    explode: bool,
) -> Option<Vec<(String, String)>> {
    let encode = |v: &Value| -> Option<String> {
        if v.is_array() || v.is_object() {
            return None;
        }
        let raw = v
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| v.to_string());
        Some(if location == "header" {
            raw
        } else {
            url_component(&raw)
        })
    };
    let name = if location == "header" {
        name.to_owned()
    } else {
        url_component(name)
    };
    let array = match value.as_array() {
        Some(items) => Some(items.iter().map(&encode).collect::<Option<Vec<_>>>()?),
        None => None,
    };
    let object = match value.as_object() {
        Some(items) => Some(
            items
                .iter()
                .map(|(k, v)| {
                    Some((
                        if location == "header" {
                            k.clone()
                        } else {
                            url_component(k)
                        },
                        encode(v)?,
                    ))
                })
                .collect::<Option<Vec<_>>>()?,
        ),
        None => None,
    };
    let scalar = if array.is_none() && object.is_none() {
        Some(encode(value)?)
    } else {
        None
    };
    let flat = |separator: &str, kv: &str| -> String {
        if let Some(items) = &array {
            items.join(separator)
        } else if let Some(items) = &object {
            items
                .iter()
                .map(|(k, v)| format!("{k}{kv}{v}"))
                .collect::<Vec<_>>()
                .join(separator)
        } else {
            scalar.clone().unwrap_or_default()
        }
    };
    let one = |v| Some(vec![(name.clone(), v)]);
    match (location, style) {
        ("query" | "cookie", "form") if explode => {
            if let Some(items) = array {
                Some(items.into_iter().map(|v| (name.clone(), v)).collect())
            } else if let Some(items) = object {
                Some(items)
            } else {
                one(scalar.unwrap())
            }
        }
        ("query" | "cookie", "form") => one(flat(",", ",")),
        ("query", "spaceDelimited" | "pipeDelimited") if !explode && array.is_some() => one(flat(
            if style == "spaceDelimited" {
                "%20"
            } else {
                "|"
            },
            ",",
        )),
        ("query", "deepObject") if explode && object.is_some() => Some(
            object
                .unwrap()
                .into_iter()
                .map(|(k, v)| (format!("{name}%5B{k}%5D"), v))
                .collect(),
        ),
        ("path" | "header", "simple") => one(flat(",", if explode { "=" } else { "," })),
        ("path", "label") => one(format!(
            ".{}",
            flat(
                if explode { "." } else { "," },
                if explode { "=" } else { "," }
            )
        )),
        ("path", "matrix") if explode && object.is_some() => one(object
            .unwrap()
            .into_iter()
            .map(|(k, v)| format!(";{k}={v}"))
            .collect::<String>()),
        ("path", "matrix") if explode && array.is_some() => one(array
            .unwrap()
            .into_iter()
            .map(|v| format!(";{name}={v}"))
            .collect::<String>()),
        ("path", "matrix") => one(format!(";{name}={}", flat(",", ","))),
        _ => None,
    }
}

fn request_example(api: &Api, object: &Value, source: &Location) -> Result<Option<Value>> {
    if let Some(example) = object.get("example") {
        return Ok(Some(example.clone()));
    }
    if let Some(examples) = object.get("examples").and_then(Value::as_object) {
        for name in examples.keys() {
            let example = resolve(api, &source.child("examples").child(name))?;
            if let Some(value) = example.value.get("value") {
                return Ok(Some(value.clone()));
            }
        }
    }
    Ok(None)
}

fn servers(operation: &Operation) -> Vec<Server> {
    let mut result = Vec::new();
    for server in &operation.servers {
        let mut base = text(server, "url");
        if let Some(variables) = server.get("variables").and_then(Value::as_object) {
            for (name, variable) in variables {
                if let Some(default) = variable.get("default").and_then(Value::as_str) {
                    base = base.replace(&format!("{{{name}}}"), default);
                }
            }
        }
        let mut notes = Vec::new();
        if base.starts_with("//") {
            base = format!("https:{base}");
        }
        if !base.starts_with("https://") && !base.starts_with("http://") {
            notes.push(
                "The server URL is relative. Replace the example origin with your API origin."
                    .into(),
            );
            base = format!("https://api.example.com/{}", base.trim_start_matches('/'));
        }
        if let Some((without_fragment, _)) = base.split_once('#') {
            notes.push("The server URL fragment is omitted from the HTTP request.".into());
            base = without_fragment.to_owned();
        }
        let description = text(server, "description");
        let label = if description.is_empty() {
            base.clone()
        } else {
            format!("{description} — {base}")
        };
        result.push(Server { label, base, notes });
    }
    if result.is_empty() {
        result.push(Server {
            label: "API server".into(),
            base: "https://api.example.com".into(),
            notes: vec!["Replace the example origin with your API origin.".into()],
        });
    }
    result
}

fn authentication(api: &Api, operation: &Operation) -> Result<Vec<Authentication>> {
    let mut result = Vec::new();
    for requirement in &operation.security {
        let mut choice = Authentication::default();
        let Some(schemes) = requirement.as_object() else {
            choice.label = format!("Authentication option {}", result.len() + 1);
            choice
                .notes
                .push("Configure this authentication option manually.".into());
            result.push(choice);
            continue;
        };
        let mut labels = Vec::new();
        for (name, scopes) in schemes {
            let scope_names = scopes
                .as_array()
                .map(|v| v.iter().filter_map(Value::as_str).collect::<Vec<_>>())
                .unwrap_or_default();
            labels.push(if scope_names.is_empty() {
                name.clone()
            } else {
                format!("{name} ({})", scope_names.join(", "))
            });
            let location = api
                .graph
                .root
                .child("components")
                .child("securitySchemes")
                .child(name);
            if api.graph.value(&location).is_err() {
                choice.notes.push(format!(
                    "Configure authentication for {name}; its security scheme is not defined."
                ));
                continue;
            }
            let scheme = resolve(api, &location)?.value;
            match text(&scheme, "type").as_str() {
                "http" if text(&scheme, "scheme").eq_ignore_ascii_case("basic") => choice
                    .args
                    .push(format!("--user {}", shell_quote("USERNAME:PASSWORD"))),
                "http" if text(&scheme, "scheme").eq_ignore_ascii_case("bearer") => {
                    choice.args.push(format!(
                        "--header {}",
                        shell_quote("Authorization: Bearer REPLACE_TOKEN")
                    ))
                }
                "oauth2" | "openIdConnect" => {
                    choice.args.push(format!(
                        "--header {}",
                        shell_quote("Authorization: Bearer REPLACE_TOKEN")
                    ));
                    if !scope_names.is_empty() {
                        choice.notes.push(format!(
                            "Required token scopes: {}.",
                            scope_names.join(", ")
                        ));
                    }
                }
                "apiKey" => {
                    let key = text(&scheme, "name");
                    match text(&scheme, "in").as_str() {
                        "query" => choice
                            .query
                            .push(format!("{}=REPLACE_API_KEY", url_component(&key))),
                        "cookie" => choice.cookies.push(format!("{key}=REPLACE_API_KEY")),
                        "header" => choice.args.push(format!(
                            "--header {}",
                            shell_quote(&format!("{key}: REPLACE_API_KEY"))
                        )),
                        _ => choice
                            .notes
                            .push(format!("Configure authentication for {name} manually.")),
                    }
                }
                _ => choice
                    .notes
                    .push(format!("Configure authentication for {name} manually.")),
            }
        }
        choice.label = if labels.is_empty() {
            "No authentication".into()
        } else {
            labels.join(" + ")
        };
        result.push(choice);
    }
    if result.is_empty() {
        result.push(Authentication {
            label: "No authentication".into(),
            ..Default::default()
        });
    }
    Ok(result)
}

fn body_args(
    media_type: &str,
    media: &Value,
    example: Option<Value>,
) -> (Vec<String>, Vec<String>) {
    let mut args = Vec::new();
    let mut notes = Vec::new();
    let kind = media_type
        .split(';')
        .next()
        .unwrap()
        .trim()
        .to_ascii_lowercase();
    let form = kind == "application/x-www-form-urlencoded" || kind == "multipart/form-data";
    if form
        && media
            .get("encoding")
            .and_then(Value::as_object)
            .is_none_or(|e| e.is_empty())
        && example
            .as_ref()
            .and_then(Value::as_object)
            .is_some_and(|o| !o.is_empty() && o.values().all(|v| !v.is_array() && !v.is_object()))
    {
        let fields = example.as_ref().unwrap().as_object().unwrap();
        if kind == "application/x-www-form-urlencoded" {
            let payload = fields
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}={}",
                        url_component(k),
                        url_component(
                            &v.as_str()
                                .map(str::to_owned)
                                .unwrap_or_else(|| v.to_string())
                        )
                    )
                })
                .collect::<Vec<_>>()
                .join("&");
            args.push(format!(
                "--header {}",
                shell_quote(&format!("Content-Type: {media_type}"))
            ));
            args.push(format!("--data-raw {}", shell_quote(&payload)));
        } else {
            for (key, value) in fields {
                let raw = value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string());
                args.push(format!(
                    "--form-string {}",
                    shell_quote(&format!("{key}={raw}"))
                ));
            }
        }
    } else {
        let payload = if form {
            None
        } else {
            example.and_then(|v| {
                if kind.ends_with("/json") || kind.ends_with("+json") {
                    Some(v.to_string())
                } else {
                    v.as_str().map(str::to_owned)
                }
            })
        };
        let content_type = if kind == "multipart/form-data" {
            notes.push("For multipart requests without a flat supplied example, prepare request-body with encoded parts and replace REPLACE_BOUNDARY with its boundary. Custom encoding and file parts must be prepared explicitly.".into());
            "multipart/form-data; boundary=REPLACE_BOUNDARY"
        } else {
            media_type
        };
        args.push(format!(
            "--header {}",
            shell_quote(&format!("Content-Type: {content_type}"))
        ));
        args.push(format!(
            "{} {}",
            if payload.is_some() {
                "--data-raw"
            } else {
                "--data-binary"
            },
            shell_quote(payload.as_deref().unwrap_or("@request-body"))
        ));
    }
    (args, notes)
}

fn bodies(api: &Api, operation: &Operation) -> Result<Vec<Body>> {
    let mut bodies = Vec::new();
    if let Some(body) = &operation.request_body
        && let Some(content) = body.value.get("content").and_then(Value::as_object)
    {
        for (media_type, media) in content {
            let mut examples = Vec::new();
            if let Some(example) = media.get("example") {
                examples.push(("Example".to_owned(), Some(example.clone()), Vec::new()));
            }
            if let Some(named) = media.get("examples").and_then(Value::as_object) {
                for name in named.keys() {
                    let example = resolve(
                        api,
                        &body
                            .source
                            .child("content")
                            .child(media_type)
                            .child("examples")
                            .child(name),
                    )?;
                    let summary = text(&example.value, "summary");
                    let label = if summary.is_empty() {
                        name.clone()
                    } else {
                        format!("{summary} ({name})")
                    };
                    let mut notes = Vec::new();
                    if example.value.get("externalValue").is_some() {
                        notes.push(format!("Example {name} is external and is not fetched. Prepare request-body with its payload."));
                    }
                    examples.push((label, example.value.get("value").cloned(), notes));
                }
            }
            if examples.is_empty() {
                examples.push(("From request-body file".into(), None, Vec::new()));
            }
            for (label, example, mut notes) in examples {
                let (args, body_notes) = body_args(media_type, media, example);
                notes.extend(body_notes);
                bodies.push(Body {
                    media: media_type.clone(),
                    label,
                    args,
                    notes,
                });
            }
        }
    }
    if bodies.is_empty() {
        bodies.push(Body {
            media: "Request".into(),
            label: "Request".into(),
            args: Vec::new(),
            notes: Vec::new(),
        });
    }
    Ok(bodies)
}

pub(super) fn choices(api: &Api, operation: &Operation) -> Result<Choices> {
    let mut data=Choices {
        prefix:format!("curl --globoff --request {}",shell_quote(&operation.method.to_uppercase())),
        path:operation.path.clone(),args:Vec::new(),query:Vec::new(),cookies:Vec::new(),
        notes:vec!["POSIX shell examples. Replace placeholder values and credentials before running. Optional parameters without examples or defaults are omitted. Bodies without a supplied example are read from a local file.".into()],
        servers:servers(operation),authentication:authentication(api,operation)?,bodies:bodies(api,operation)?,
    };
    for parameter in &operation.parameters {
        let example = request_example(
            api,
            &parameter.definition.value,
            &parameter.definition.source,
        )?;
        let schema = if parameter.definition.value.get("schema").is_some() {
            Some(resolve(api, &parameter.definition.source.child("schema"))?.value)
        } else {
            None
        };
        let example = example.or_else(|| {
            schema
                .as_ref()
                .and_then(|s| s.get("example").or_else(|| s.get("default")))
                .cloned()
        });
        if !parameter.required && example.is_none() {
            continue;
        }
        let value = example.unwrap_or_else(|| Value::String(format!("REPLACE_{}", parameter.name)));
        let definition = &parameter.definition.value;
        let default_style = if parameter.location == "query" || parameter.location == "cookie" {
            "form"
        } else {
            "simple"
        };
        let style = definition
            .get("style")
            .and_then(Value::as_str)
            .unwrap_or(default_style);
        let explode = definition
            .get("explode")
            .and_then(Value::as_bool)
            .unwrap_or(style == "form");
        let serialized =
            serialize_parameter(&parameter.name, &value, &parameter.location, style, explode);
        let serialized = match serialized.filter(|_| definition.get("content").is_none()) {
            None => {
                data.notes.push(format!("Parameter {} requires custom serialization. Replace its placeholder using the parameter's style, explode and content rules.", parameter.name));
                vec![(
                    url_component(&parameter.name),
                    "REPLACE_SERIALIZED_VALUE".into(),
                )]
            }
            Some(serialized) => serialized,
        };
        match parameter.location.as_str() {
            "path" => {
                data.path = data
                    .path
                    .replace(&format!("{{{}}}", parameter.name), &serialized[0].1)
            }
            "query" => data.query.extend(
                serialized
                    .into_iter()
                    .map(|(name, value)| format!("{name}={value}")),
            ),
            "header" => data.args.push(format!(
                "--header {}",
                shell_quote(&format!("{}: {}", parameter.name, serialized[0].1))
            )),
            "cookie" => data.cookies.extend(
                serialized
                    .into_iter()
                    .map(|(name, value)| format!("{name}={value}")),
            ),
            _ => {}
        }
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn choices_for(spec: Value) -> Choices {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("spec.json");
        std::fs::write(&path, spec.to_string()).unwrap();
        let api = crate::openapi::load(&path).unwrap();
        choices(&api, &api.operations[0]).unwrap()
    }

    #[test]
    fn selections_combine_servers_auth_alternatives_and_named_body_examples() {
        let data = choices_for(json!({
            "openapi":"3.1.0","info":{"title":"Choices","version":"1"},
            "servers":[
                {"url":"https://prod.test/{version}?fixed=yes#ignored","variables":{"version":{"default":"v2"}},"description":"Production"},
                {"url":"//sandbox.test/v2","description":"Sandbox"}
            ],
            "security":[{"bearer":[],"queryKey":[],"cookieKey":[]},{"basic":[]},{}],
            "components":{
                "securitySchemes":{
                    "bearer":{"type":"http","scheme":"bearer"},
                    "queryKey":{"type":"apiKey","in":"query","name":"api key"},
                    "cookieKey":{"type":"apiKey","in":"cookie","name":"key"},
                    "basic":{"type":"http","scheme":"basic"}
                },
                "examples":{"published":{"summary":"Published document","value":{"state":"published","text":"don't $(echo unsafe)"}}},
                "requestBodies":{"Body":{"content":{
                    "application/json":{"examples":{"draft":{"value":{"state":"draft"}},"published":{"$ref":"#/components/examples/published"}}},
                    "text/plain":{"example":"@literal"}
                }}}
            },
            "paths":{"/items/{id}":{"post":{
                "parameters":[
                    {"name":"id","in":"path","required":true,"example":"a/b","schema":{"type":"string"}},
                    {"name":"limit","in":"query","schema":{"type":"integer","default":20}},
                    {"name":"session","in":"cookie","required":true,"example":"a b","schema":{"type":"string"}}
                ],
                "requestBody":{"$ref":"#/components/requestBodies/Body"},"responses":{"200":{"description":"OK"}}
            }}}
        }));
        assert_eq!(data.servers.len(), 2);
        assert_eq!(data.authentication.len(), 3);
        assert_eq!(data.bodies.len(), 3);
        assert_eq!(data.bodies[1].label, "Published document (published)");
        let published = data.command(0, 0, 1);
        assert!(published.contains(
            "https://prod.test/v2/items/a%2Fb?fixed=yes&limit=20&api%20key=REPLACE_API_KEY"
        ));
        assert!(published.contains("Authorization: Bearer REPLACE_TOKEN"));
        assert!(published.contains("--cookie 'session=a%20b; key=REPLACE_API_KEY'"));
        assert!(published.contains("published"));
        assert!(published.contains("don'\\''t $(echo unsafe)"));
        let sandbox = data.command(1, 1, 0);
        assert!(sandbox.contains("https://sandbox.test/v2/items/a%2Fb?limit=20"));
        assert!(sandbox.contains("--user 'USERNAME:PASSWORD'"));
        assert!(!sandbox.contains("REPLACE_API_KEY"));
        assert!(!sandbox.contains("Authorization:"));
        assert!(sandbox.contains("draft"));
        let anonymous = data.command(1, 2, 2);
        assert!(!anonymous.contains("--user"));
        assert!(!anonymous.contains("Authorization:"));
        assert!(!anonymous.contains("REPLACE_API_KEY"));
        assert!(anonymous.contains("Content-Type: text/plain"));
        assert!(anonymous.contains("--data-raw '@literal'"));
    }

    #[test]
    fn effective_overrides_and_external_examples_do_not_fetch_or_invent_credentials() {
        let data = choices_for(json!({
            "openapi":"3.0.3","info":{"title":"Choices","version":"1"},
            "servers":[{"url":"https://unused.test"}],"security":[{"bearer":[]}],
            "paths":{"/items":{"post":{
                "servers":[{"url":"/local"}],"security":[],
                "requestBody":{"content":{"application/json":{"examples":{"remote":{"externalValue":"https://unreachable.invalid/example.json"}}}}},
                "responses":{"200":{"description":"OK"}}
            }}}
        }));
        assert_eq!(data.servers.len(), 1);
        assert_eq!(data.authentication[0].label, "No authentication");
        assert_eq!(data.bodies[0].label, "remote");
        assert!(data.bodies[0].notes[0].contains("not fetched"));
        let command = data.command(0, 0, 0);
        assert!(command.contains("https://api.example.com/local/items"));
        assert!(command.contains("--data-binary '@request-body'"));
        assert!(!command.contains("unused.test"));
        assert!(!command.contains("Authorization:"));
    }

    #[test]
    fn choice_data_is_not_a_cartesian_product_of_commands() {
        let mut spec = json!({"openapi":"3.1.0","info":{"title":"Choices","version":"1"},"servers":[],"security":[],"paths":{"/items":{"post":{"requestBody":{"content":{"application/json":{"examples":{}}}},"responses":{"200":{"description":"OK"}}}}}});
        for index in 0..30 {
            spec["servers"]
                .as_array_mut()
                .unwrap()
                .push(json!({"url":format!("https://server{index}.test")}));
            spec["security"].as_array_mut().unwrap().push(json!({}));
            spec["paths"]["/items"]["post"]["requestBody"]["content"]["application/json"]["examples"]
                [format!("example{index}")] = json!({"value":{"index":index}});
        }
        let data = choices_for(spec);
        assert_eq!(data.servers.len(), 30);
        assert_eq!(data.authentication.len(), 30);
        assert_eq!(data.bodies.len(), 30);
        assert!(serde_json::to_vec(&data).unwrap().len() < 30_000);
    }
}
