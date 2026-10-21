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
            self.html.push_str(
                "<div class=\"api-media api-media-single\"><section class=\"api-media-panel\">",
            );
            self.md(format!(
                "{} {}\n\n",
                "#".repeat(heading_level),
                inline(&name)
            ));
            self.append(content);
            self.html.push_str("</section></div>");
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
    include_intro: bool,
) -> Result<Content> {
    let mut out = Content::default();
    if depth >= 5 || !visiting.insert(source.clone()) {
        out.md(
            "Continue with the linked schema definition to explore this nested or recursive value.",
        );
        return Ok(out);
    }
    if include_intro {
        out.md(format!(
            "**Type:** {}\n\n",
            schema_type(api, schema, source, routes)
        ));
    }
    if let Some(edge) = api.graph.reference_at(source) {
        if !visiting.contains(&edge.target) {
            out.append(schema_fields(
                api,
                value(api, &edge.target)?,
                &edge.target,
                routes,
                depth + 1,
                visiting,
                true,
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
    if include_intro && !description.is_empty() {
        out.md(description);
    }
    let constraints = constraints(schema, api.graph.version);
    if !constraints.is_empty() {
        out.html.push_str("<div class=\"api-schema-constraints\">");
        out.md(constraints
            .iter()
            .map(|s| format!("- {s}\n"))
            .collect::<String>());
        out.html.push_str("</div>");
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
                let body =
                    schema_fields(api, property, &location, routes, depth + 1, visiting, true)?;
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
            true,
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
                    true,
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
                    true,
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

fn curl_examples(api: &Api, operation: &Operation, out: &mut Content, slug: &str) -> Result<()> {
    let choices = super::curl::choices(api, operation)?;
    out.markdown.push_str("## cURL examples\n\n");
    let mut notes = Content::default();
    for note in &choices.notes {
        notes.md(inline(note));
    }
    out.markdown.push_str(&notes.markdown);
    let model = serde_json::to_string(&choices).map_err(|e| Error::General(e.to_string()))?;
    let attr = |value: &str| {
        html_escape(value)
            .replace('[', "&#91;")
            .replace(']', "&#93;")
    };
    let id = format!("{slug}-curl-preview");
    out.html.push_str(&format!("<details class=\"api-curl\" data-api-curl data-api-curl-model=\"{}\"><summary>Copy as cURL</summary><div class=\"api-curl-menu\"><div class=\"api-curl-controls\" data-api-curl-controls hidden>", attr(&model)));
    out.html.push_str(&format!(
        "<label{}>Server<select data-api-curl-server>",
        if choices.servers.len() < 2 {
            " hidden"
        } else {
            ""
        }
    ));
    for (index, server) in choices.servers.iter().enumerate() {
        out.html.push_str(&format!(
            "<option value=\"{index}\">{}</option>",
            attr(&server.label)
        ));
    }
    out.html.push_str("</select></label>");
    out.html.push_str(&format!(
        "<label{}>Authentication<select data-api-curl-auth>",
        if choices.authentication.len() < 2 {
            " hidden"
        } else {
            ""
        }
    ));
    for (index, auth) in choices.authentication.iter().enumerate() {
        out.html.push_str(&format!(
            "<option value=\"{index}\">{}</option>",
            attr(&auth.label)
        ));
    }
    out.html.push_str("</select></label>");
    let media: Vec<_> = choices
        .bodies
        .iter()
        .map(|b| &b.media)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    out.html.push_str(&format!(
        "<label{}>Content type<select data-api-curl-media>",
        if media.len() < 2 { " hidden" } else { "" }
    ));
    for kind in media {
        out.html.push_str(&format!(
            "<option value=\"{}\">{}</option>",
            attr(kind),
            attr(kind)
        ));
    }
    out.html.push_str(
        "</select></label><label data-api-curl-example-label>Example<select data-api-curl-example>",
    );
    for (index, body) in choices
        .bodies
        .iter()
        .enumerate()
        .filter(|(_, b)| b.media == choices.bodies[0].media)
    {
        out.html.push_str(&format!(
            "<option value=\"{index}\">{}</option>",
            attr(&body.label)
        ));
    }
    out.html.push_str(&format!("</select></label><div class=\"api-curl-actions\"><button type=\"button\" data-api-curl-copy>Copy cURL</button><button type=\"button\" data-api-curl-show aria-controls=\"{}\" aria-expanded=\"false\">Show command</button></div><div data-api-curl-selected-notes></div></div><p class=\"api-curl-status\" role=\"status\" aria-live=\"polite\"></p><div class=\"api-curl-command\" id=\"{}\" data-api-curl-preview hidden><pre data-api-curl-preview-code></pre></div><div data-api-curl-fallback>",attr(&id),attr(&id)));
    let mut fallback = Content::default();
    fallback.md("Without JavaScript, commands below use the first server and authentication option. Other available servers and authentication options are listed for manual adjustment.");
    if choices.servers.len() > 1 {
        fallback.md("**Available servers**");
        for server in &choices.servers {
            fallback.md(format!("- {}", inline(&server.label)));
        }
    }
    if choices.authentication.len() > 1 {
        fallback.md("**Authentication alternatives**");
        for auth in &choices.authentication {
            fallback.md(format!("- {}", inline(&auth.label)));
            let mut flags = auth.args.clone();
            flags.extend(auth.query.iter().map(|q| format!("Query: {q}")));
            flags.extend(auth.cookies.iter().map(|c| format!("Cookie: {c}")));
            if !flags.is_empty() {
                fallback.md(fenced_shell(&flags.join("\n")));
            }
            for note in &auth.notes {
                fallback.md(inline(note));
            }
        }
    }
    for note in &choices.servers[0].notes {
        fallback.md(inline(note));
    }
    if choices.authentication.len() == 1 {
        for note in &choices.authentication[0].notes {
            fallback.md(inline(note));
        }
    }
    for (index, body) in choices.bodies.iter().enumerate() {
        let title = format!("{} — {}", body.media, body.label);
        fallback
            .markdown
            .push_str(&format!("### {}\n\n", inline(&title)));
        fallback.html.push_str(&format!(
            "<p class=\"api-curl-example-title\">{}</p>",
            attr(&title)
        ));
        for note in &body.notes {
            fallback.md(inline(note));
        }
        fallback.md(fenced_shell(&choices.command(0, 0, index)));
    }
    // The static representation is shared by no-JavaScript readers and exports.
    out.markdown.push_str(&fallback.markdown);
    out.html
        .push_str(&fallback.html.replace('[', "&#91;").replace(']', "&#93;"));
    out.html.push_str("</div><div class=\"api-curl-notes\">");
    out.html
        .push_str(&notes.html.replace('[', "&#91;").replace(']', "&#93;"));
    out.html.push_str("</div></div></details>");
    Ok(())
}

fn fenced_shell(command: &str) -> String {
    let fence = "`".repeat(3.max(command.split(|c| c != '`').map(str::len).max().unwrap_or(0) + 1));
    format!("{fence}bash\n{command}\n{fence}")
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
                    true,
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
    registry: &crate::components::ComponentRegistry,
) -> Result<Content> {
    let mut out = Content::default();
    let title = operation
        .summary
        .clone()
        .unwrap_or_else(|| format!("{} {}", operation.method, operation.path));
    // Keep a readable Markdown export independent of the visual component.
    let mut header = Content::default();
    header.md(format!("# {}", inline(&title)));
    header.markdown.push_str(&format!(
        "**{} {}**\n\n",
        operation.method,
        inline(&operation.path)
    ));
    if operation.deprecated {
        header.markdown.push_str("**Deprecated.**\n\n");
    }
    if !operation.tags.is_empty() {
        header.markdown.push_str(&format!(
            "Tags: {}\n\n",
            operation
                .tags
                .iter()
                .map(|s| inline(s))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let mut description = Content::default();
    if let Some(value) = &operation.description {
        description.md(value);
    }
    header.markdown.push_str(&description.markdown);
    let mut curl = Content::default();
    curl_examples(api, operation, &mut curl, slug)?;
    let mut context = tera::Context::new();
    context.insert("curl", &curl.html);
    context.insert("attrs", &json!({
        "title": title, "method": operation.method, "path": operation.path,
        "deprecated": if operation.deprecated { "true" } else { "false" },
        "endpoint": if operation.summary.is_some() && title != format!("{} {}", operation.method, operation.path) { "true" } else { "false" }
    }));
    context.insert("tags", &operation.tags);
    context.insert("body", &description.html);
    context.insert("body_raw", &description.markdown);
    context.insert("name", "api-operation");
    context.insert("inline", &false);
    out.markdown.push_str(&header.markdown);
    out.html
        .push_str("<article class=\"api-reference api-reference-operation\">");
    let rendered_header = registry.render_native("api-operation", &context)?;
    out.html.push_str(&rendered_header);
    // Older ejected headers may not render the new optional context field.
    if !rendered_header.contains("data-api-curl") {
        out.html.push_str(&curl.html);
    }
    out.html.push_str(super::publish::RELATED_GUIDES_MARKER);
    out.markdown.push_str(super::publish::RELATED_GUIDES_MARKER);
    out.markdown.push_str("\n\n");
    out.html
        .push_str("<div class=\"api-context\"><section class=\"api-context-card\">");
    security(api, &operation.security, &mut out)?;
    out.html
        .push_str("</section><section class=\"api-context-card\">");
    servers(&mut out, &operation.servers);
    out.html.push_str("</section></div>");
    if !operation.parameters.is_empty() {
        out.html.push_str("<section class=\"api-section\">");
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
    if !operation.parameters.is_empty() {
        out.html.push_str("</section>");
    }
    if let Some(body) = &operation.request_body {
        out.html
            .push_str("<section class=\"api-section api-request\">");
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
        out.html.push_str("</section>");
    }
    out.markdown.push_str(&curl.markdown);
    out.html
        .push_str("<section class=\"api-section api-responses\">");
    out.md("## Responses");
    for (status, response) in &operation.responses {
        let kind = match status.as_bytes().first() {
            Some(b'2') => "success",
            Some(b'4' | b'5') => "error",
            _ => "neutral",
        };
        out.html.push_str(&format!(
            "<section class=\"api-response api-response-{kind}\">"
        ));
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
        out.html.push_str("</section>");
    }
    out.html.push_str("</section>");
    // Preserve normalized values too: source Reference Objects alone would hide
    // inherited parameters and externally defined request/response details.
    out.source(&json!({
        "operation": value(api, &operation.source)?, "effectiveServers": operation.servers,
        "effectiveSecurity": operation.security,
        "effectiveParameters": operation.parameters.iter().map(|p| &p.definition.value).collect::<Vec<_>>(),
        "requestBody": operation.request_body.as_ref().map(|b| &b.value),
        "responses": operation.responses.iter().map(|(s,r)| (s,&r.value)).collect::<BTreeMap<_,_>>()
    }));
    out.html.push_str("</article>");
    Ok(out)
}

pub(super) fn schema(
    api: &Api,
    name: &str,
    source: &Location,
    routes: &Routes,
    registry: &crate::components::ComponentRegistry,
) -> Result<Content> {
    let mut out = Content::default();
    let definition = value(api, source)?;
    let kind = schema_type(api, definition, source, routes);
    let ignores_siblings =
        api.graph.version == Version::V3_0 && api.graph.reference_at(source).is_some();
    let mut description = Content::default();
    if !ignores_siblings {
        description.md(text(definition, "description"));
    }
    let mut rendered_type = Content::default();
    rendered_type.md(&kind);
    let mut context = tera::Context::new();
    context.insert("attrs", &json!({"title": name, "deprecated": if !ignores_siblings && definition.get("deprecated").and_then(Value::as_bool) == Some(true) { "true" } else { "false" }}));
    context.insert("type_html", &rendered_type.html);
    context.insert("body", &description.html);
    context.insert("body_raw", &description.markdown);
    context.insert("name", "api-schema");
    context.insert("inline", &false);
    out.markdown.push_str(&format!(
        "# {}\n\n**Type:** {kind}\n\n{}",
        inline(name),
        description.markdown
    ));
    out.html
        .push_str("<article class=\"api-reference api-reference-schema\">");
    out.html
        .push_str(&registry.render_native("api-schema", &context)?);
    out.html
        .push_str("<section class=\"api-section api-schema-definition\">");
    out.md("## Properties");
    out.append(schema_fields(
        api,
        definition,
        source,
        routes,
        0,
        &mut BTreeSet::new(),
        false,
    )?);
    out.html.push_str("</section>");
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
    out.html.push_str("</article>");
    Ok(out)
}

pub(super) fn overview(
    api: &Api,
    operations: &[(String, &Operation)],
    schemas: &[(String, Location, String)],
    download: &super::download::Download,
) -> Result<Content> {
    let mut out = Content::default();
    let format = download.format();
    out.html.push_str(&format!("<article class=\"api-reference api-reference-overview\"><header class=\"api-page-header\"><div class=\"api-overview-toolbar\"><span class=\"api-eyebrow\">API reference</span><a class=\"api-download\" href=\"{}\" download>Download OpenAPI <span>{format}</span></a></div>", html_escape(&download.name)));
    out.md(format!("# {}", inline(&api.title)));
    if let Some(description) = &api.description {
        out.md(description);
    }
    out.md(format!("**API version:** {}", inline(&api.api_version)));
    out.html.push_str(&format!("<div class=\"api-summary\"><span><strong>{}</strong> operations</span><span><strong>{}</strong> schemas</span></div></header><div class=\"api-context\"><section class=\"api-context-card\">", operations.len(), schemas.len()));
    let root = value(api, &api.graph.root)?;
    servers(
        &mut out,
        root.get("servers")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[json!({"url": "/"})]),
    );
    out.html
        .push_str("</section><section class=\"api-context-card\">");
    security(
        api,
        root.get("security")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default(),
        &mut out,
    )?;
    out.html
        .push_str("</section></div><section class=\"api-section\">");
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
    out.html
        .push_str("</section><section class=\"api-section\">");
    out.md("## Schemas");
    out.md(schemas
        .iter()
        .map(|(name, _, slug)| format!("- {}\n", link(slug, name)))
        .collect::<String>());
    out.html.push_str("</section>");
    out.source(root);
    out.html.push_str("</article>");
    Ok(out)
}

fn operation_table<'a>(
    out: &mut Content,
    operations: impl IntoIterator<Item = (&'a str, &'a Operation)>,
) {
    let mut table = String::from("| Method | Path | Operation |\n| --- | --- | --- |\n");
    let mut count = 0;
    let mut methods = BTreeSet::new();
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
        methods.insert(operation.method.as_str());
        count += 1;
    }
    if count == 0 {
        out.md("No operations.");
    } else {
        let mut rendered = Content::default();
        rendered.table(table);
        rendered.html =
            rendered
                .html
                .replacen("<table>", "<table class=\"api-operation-table\">", 1);
        // Decorate only the native table's method cells; export Markdown keeps
        // plain method text and specification Markdown never enables raw HTML.
        for method in methods {
            rendered.html = rendered.html.replace(
                &format!("<td><strong>{method}</strong></td>"),
                &format!("<td><span class=\"nav-method-badge nav-method-{}\" data-http-method=\"{method}\">{method}</span></td>", method.to_ascii_lowercase()),
            );
        }
        out.append(rendered);
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

#[cfg(test)]
mod curl_tests {
    use super::super::curl::{serialize_parameter, shell_quote};
    use super::*;

    #[test]
    fn serializes_openapi_parameter_styles_without_encoding_delimiters() {
        let cases = [
            (
                "query",
                "form",
                true,
                json!(["a b", "c&d"]),
                vec![("id", "a%20b"), ("id", "c%26d")],
            ),
            (
                "query",
                "form",
                false,
                json!({"x":"a b","y":2}),
                vec![("id", "x,a%20b,y,2")],
            ),
            (
                "query",
                "deepObject",
                true,
                json!({"x":"a&b"}),
                vec![("id%5Bx%5D", "a%26b")],
            ),
            (
                "path",
                "matrix",
                true,
                json!([1, 2]),
                vec![("id", ";id=1;id=2")],
            ),
            (
                "path",
                "label",
                true,
                json!({"x":1,"y":2}),
                vec![("id", ".x=1.y=2")],
            ),
            (
                "header",
                "simple",
                true,
                json!({"x":"a b"}),
                vec![("id", "x=a b")],
            ),
            (
                "query",
                "spaceDelimited",
                false,
                json!(["a b", "c"]),
                vec![("id", "a%20b%20c")],
            ),
        ];
        for (location, style, explode, value, expected) in cases {
            let actual = serialize_parameter("id", &value, location, style, explode).unwrap();
            assert_eq!(
                actual,
                expected
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect::<Vec<_>>()
            );
        }
        assert!(serialize_parameter("id", &json!({"nested":[1]}), "query", "form", true).is_none());
    }

    fn render(spec: Value) -> String {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api.json");
        std::fs::write(&path, spec.to_string()).unwrap();
        let api = super::super::load(&path).unwrap();
        let mut out = Content::default();
        curl_examples(&api, &api.operations[0], &mut out, "test").unwrap();
        out.markdown
    }

    #[test]
    fn generates_effective_url_credentials_and_media_examples() {
        let output = render(json!({
            "openapi":"3.0.3", "info":{"title":"Test", "version":"1"},
            "servers":[{"url":"https://example.test/{version}", "variables":{"version":{"default":"v2"}}}],
            "security":[{"token":[],"key":[]}],
            "components":{"securitySchemes":{
                "token":{"type":"http","scheme":"bearer"},
                "key":{"type":"apiKey","in":"query","name":"api-key"}
            }, "examples":{"body":{"value":{"text":"don't $(touch /tmp/nope)"}}}},
            "paths":{"/items/{id}":{"parameters":[{"name":"id","in":"path","required":true,"example":"a/b ?","schema":{"type":"string"}}],
                "post":{"parameters":[
                    {"name":"tag","in":"query","example":["a b","c&d"],"schema":{"type":"array","items":{"type":"string"}}},
                    {"name":"omitted","in":"query","schema":{"type":"string"}}
                ],"requestBody":{"content":{
                    "application/json":{"examples":{"payload":{"$ref":"#/components/examples/body"}}},
                    "text/plain":{"example":"@literal"},
                    "application/octet-stream":{"schema":{"type":"string","format":"binary"}}
                }},"responses":{"200":{"description":"OK"}}}}
            }
        }));
        assert!(output.contains(
            "https://example.test/v2/items/a%2Fb%20%3F?tag=a%20b&tag=c%26d&api-key=REPLACE_API_KEY"
        ));
        assert!(output.contains("Authorization: Bearer REPLACE_TOKEN"));
        assert!(output.contains("--data-raw '@literal'"));
        assert!(output.contains("--data-binary '@request-body'"));
        assert!(output.contains("don'\\''t $(touch /tmp/nope)"));
        assert!(!output.contains("omitted="));
        assert!(output.contains(" \\\n  --url"));
    }

    #[test]
    fn forms_and_relative_servers_use_literal_fields_or_explicit_files() {
        let output = render(json!({
            "openapi":"3.1.0", "info":{"title":"Forms", "version":"1"},
            "servers":[{"url":"/v1"}], "security":[{}],
            "paths":{"/form":{"post":{
                "parameters":[
                    {"name":"a","in":"cookie","required":true,"example":"a;b","schema":{"type":"string"}},
                    {"name":"b","in":"cookie","required":true,"example":"c d","schema":{"type":"string"}},
                    {"name":"default","in":"query","schema":{"$ref":"#/components/schemas/Default"}},
                    {"name":"nested","in":"query","required":true,"example":{"x":[1]},"schema":{"type":"object"}}
                ],
                "requestBody":{"content":{
                    "application/x-www-form-urlencoded":{"example":{"q":"a&b + c"}},
                    "multipart/form-data":{"example":{"file":"@not-a-file;type=text/plain"}}
                }},"responses":{"200":{"description":"OK"}}
            }}}, "components":{"schemas":{"Default":{"type":"integer","default":10}}}
        }));
        assert!(output.contains(
            "https://api.example.com/v1/form?default=10&nested=REPLACE_SERIALIZED_VALUE"
        ));
        assert!(output.contains("--cookie 'a=a%3Bb; b=c%20d'"));
        assert!(output.contains("--data-raw 'q=a%26b%20%2B%20c'"));
        assert!(output.contains("--form-string 'file=@not-a-file;type=text/plain'"));
        assert!(!output.contains("--header 'Content-Type: multipart/form-data'"));
        assert!(!output.contains("Authorization:"));
        assert!(output.contains("requires custom serialization"));
    }

    #[test]
    #[cfg(unix)]
    fn posix_quoting_preserves_hostile_literal_text() {
        let text = "don't `echo bad` $(echo bad)\n'quoted'";
        let result = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("printf %s {}", shell_quote(text)))
            .output()
            .unwrap();
        assert!(result.status.success());
        assert_eq!(String::from_utf8(result.stdout).unwrap(), text);
    }
}
