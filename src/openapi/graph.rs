use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// Identity of a node in a parsed document. Pointers use RFC 6901 escaping;
/// `file` is canonical so relative spellings share document identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub file: PathBuf,
    pub pointer: String,
}

impl Location {
    pub fn child(&self, key: &str) -> Self {
        Self {
            file: self.file.clone(),
            pointer: format!(
                "{}/{}",
                self.pointer,
                key.replace('~', "~0").replace('/', "~1")
            ),
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.file.display(), self.pointer)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{location}: {message}")]
pub struct OpenApiError {
    pub location: Location,
    pub message: String,
}

impl OpenApiError {
    pub(crate) fn at(location: &Location, message: impl Into<String>) -> Self {
        Self {
            location: location.clone(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    V3_0,
    V3_1,
}

#[derive(Debug, Clone)]
pub struct Reference {
    /// The `$ref` field itself, not its containing object.
    pub source: Location,
    pub target: Location,
    pub spelling: String,
}

/// Original JSON-compatible values plus reference edges. Targets are not
/// substituted into values, preserving schema cycles and source provenance.
#[derive(Debug)]
pub struct DocumentGraph {
    pub root: Location,
    pub version: Version,
    documents: BTreeMap<PathBuf, Value>,
    references: BTreeMap<Location, Reference>,
    schema_nodes: BTreeSet<Location>,
}

// Context is essential: examples, defaults and extension values may contain
// literal `$ref` keys which are not OpenAPI/JSON Schema references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Root,
    PathItem,
    Operation,
    Parameter,
    Header,
    Body,
    Response,
    Media,
    Example,
    Link,
    Callback,
    Schema,
    Security,
}

impl DocumentGraph {
    pub fn load(path: &Path) -> Result<Self, OpenApiError> {
        let input = Location {
            file: path.to_path_buf(),
            pointer: String::new(),
        };
        let file =
            std::fs::canonicalize(path).map_err(|e| OpenApiError::at(&input, e.to_string()))?;
        let root = Location {
            file,
            pointer: String::new(),
        };
        let value = parse(&root)?;
        let version = match value.get("openapi").and_then(Value::as_str) {
            Some(v) if valid_version(v, "3.0.") => Version::V3_0,
            Some(v) if valid_version(v, "3.1.") => Version::V3_1,
            other => {
                return Err(OpenApiError::at(
                    &root.child("openapi"),
                    format!("expected OpenAPI 3.0.x or 3.1.x, found {other:?}"),
                ));
            }
        };
        let mut graph = Self {
            root: root.clone(),
            version,
            documents: BTreeMap::from([(root.file.clone(), value)]),
            references: BTreeMap::new(),
            schema_nodes: BTreeSet::new(),
        };
        let mut queue = VecDeque::from([(root, Kind::Root)]);
        let mut visited = BTreeSet::new();
        while let Some((location, kind)) = queue.pop_front() {
            if !visited.insert((location.clone(), kind)) {
                continue;
            }
            // Cloning this node would clone complete subtrees repeatedly. Collect
            // only traversal work and the optional reference before mutating graph.
            let value = graph.value(&location)?;
            if !value.is_object()
                && !(kind == Kind::Schema && graph.version == Version::V3_1 && value.is_boolean())
            {
                return Err(OpenApiError::at(
                    &location,
                    "expected an object (or a boolean schema in OpenAPI 3.1)",
                ));
            }
            if matches!(kind, Kind::Root | Kind::Operation | Kind::Media)
                && value.get("$ref").is_some()
            {
                return Err(OpenApiError::at(
                    &location.child("$ref"),
                    "$ref is not allowed in this OpenAPI object",
                ));
            }
            if kind == Kind::Schema {
                graph.schema_nodes.insert(location.clone());
            }
            let value = graph.value(&location)?;
            if kind == Kind::Schema
                && !(graph.version == Version::V3_0 && value.get("$ref").is_some())
            {
                check_schema(value, &location, graph.version)?;
                if let Some(dialect) = value.get("$schema") {
                    check_dialect(dialect, &location.child("$schema"))?;
                }
                for key in [
                    "$id",
                    "$anchor",
                    "$dynamicAnchor",
                    "$dynamicRef",
                    "$recursiveRef",
                    "$recursiveAnchor",
                ] {
                    if value.get(key).is_some() {
                        return Err(OpenApiError::at(
                            &location.child(key),
                            "schema URI scopes and named/dynamic anchors are not supported yet",
                        ));
                    }
                }
            }
            if kind == Kind::Root
                && let Some(dialect) = value.get("jsonSchemaDialect")
            {
                check_dialect(dialect, &location.child("jsonSchemaDialect"))?;
            }
            let reference = value
                .get("$ref")
                .map(|v| {
                    v.as_str().map(str::to_owned).ok_or_else(|| {
                        OpenApiError::at(&location.child("$ref"), "$ref must be a string")
                    })
                })
                .transpose()?;
            let children = children(value, &location, kind);
            if let Some(spelling) = reference {
                let source = location.child("$ref");
                let target = resolve(&source, &spelling)?;
                if !graph.documents.contains_key(&target.file) {
                    if graph.documents.len() >= 256 {
                        return Err(OpenApiError::at(
                            &source,
                            "reference graph exceeds 256 documents",
                        ));
                    }
                    let root = Location {
                        file: target.file.clone(),
                        pointer: String::new(),
                    };
                    let value = parse(&root).map_err(|e| {
                        OpenApiError::at(&source, format!("cannot load {spelling:?}: {e}"))
                    })?;
                    graph.documents.insert(target.file.clone(), value);
                }
                graph.value(&target).map_err(|_| {
                    OpenApiError::at(
                        &source,
                        format!("reference {spelling:?} has no target at {target}"),
                    )
                })?;
                graph.references.insert(
                    source.clone(),
                    Reference {
                        source,
                        target: target.clone(),
                        spelling,
                    },
                );
                queue.push_back((target, kind));
                // 3.0 Reference Objects ignore siblings; 3.1 Schema Objects
                // evaluate siblings. Path-item $ref siblings are ambiguous.
                if kind == Kind::PathItem {
                    let node = graph.value(&location)?;
                    if node
                        .as_object()
                        .is_some_and(|m| m.keys().any(|k| k != "$ref" && !k.starts_with("x-")))
                    {
                        return Err(OpenApiError::at(
                            &location,
                            "path-item $ref siblings are not supported; move fields to the target",
                        ));
                    }
                }
                if kind != Kind::Schema || graph.version == Version::V3_0 {
                    continue;
                }
            }
            queue.extend(children);
        }
        Ok(graph)
    }

    pub fn value(&self, location: &Location) -> Result<&Value, OpenApiError> {
        self.documents
            .get(&location.file)
            .and_then(|v| v.pointer(&location.pointer))
            .ok_or_else(|| OpenApiError::at(location, "node not found"))
    }

    pub fn dependencies(&self) -> impl Iterator<Item = &Path> {
        self.documents.keys().map(PathBuf::as_path)
    }

    pub fn references(&self) -> impl Iterator<Item = &Reference> {
        self.references.values()
    }

    pub(crate) fn is_schema(&self, location: &Location) -> bool {
        self.schema_nodes.contains(location)
    }

    pub(crate) fn schemas(&self) -> impl Iterator<Item = &Location> {
        self.schema_nodes.iter()
    }

    pub(crate) fn reference_at(&self, object: &Location) -> Option<&Reference> {
        self.references.get(&object.child("$ref"))
    }

    /// Follow a chain of Reference Objects. Recursive schemas remain edges;
    /// callers should not use this to recursively expand schema properties.
    pub fn dereference(&self, location: &Location) -> Result<Location, OpenApiError> {
        let mut current = location.clone();
        let mut seen = BTreeSet::new();
        while let Some(edge) = self.references.get(&current.child("$ref")) {
            if !seen.insert(current.clone()) {
                return Err(OpenApiError::at(
                    location,
                    "reference-only cycle has no concrete object",
                ));
            }
            current = edge.target.clone();
        }
        Ok(current)
    }
}

// Check the shapes consumed by the renderer, not complete schema validity or
// whether example/default values satisfy a schema.
fn check_schema(value: &Value, source: &Location, version: Version) -> Result<(), OpenApiError> {
    let Some(schema) = value.as_object() else {
        return Ok(());
    };
    let invalid = |key: &str, expected: &str| {
        OpenApiError::at(&source.child(key), format!("{key} must be {expected}"))
    };
    if let Some(kind) = schema.get("type") {
        let valid_type = |value: &Value| {
            value.as_str().is_some_and(|kind| {
                matches!(
                    kind,
                    "object" | "array" | "string" | "number" | "integer" | "boolean"
                ) || (version == Version::V3_1 && kind == "null")
            })
        };
        let valid = valid_type(kind)
            || (version == Version::V3_1
                && kind.as_array().is_some_and(|types| {
                    !types.is_empty()
                        && types.iter().all(valid_type)
                        && types
                            .iter()
                            .filter_map(Value::as_str)
                            .collect::<BTreeSet<_>>()
                            .len()
                            == types.len()
                }));
        if !valid {
            return Err(invalid(
                "type",
                if version == Version::V3_0 {
                    "a single OpenAPI 3.0 type name (use nullable for nullability)"
                } else {
                    "a JSON Schema type name or a non-empty array of unique type names"
                },
            ));
        }
    }
    for key in ["title", "description", "format", "pattern"] {
        if schema.get(key).is_some_and(|v| !v.is_string()) {
            return Err(invalid(key, "a string"));
        }
    }
    for key in ["readOnly", "writeOnly", "deprecated", "uniqueItems"] {
        if schema.get(key).is_some_and(|v| !v.is_boolean()) {
            return Err(invalid(key, "a boolean"));
        }
    }
    if version == Version::V3_0 && schema.get("nullable").is_some_and(|v| !v.is_boolean()) {
        return Err(invalid("nullable", "a boolean in OpenAPI 3.0"));
    }
    for key in ["minimum", "maximum", "multipleOf"] {
        if schema.get(key).is_some_and(|v| !v.is_number()) {
            return Err(invalid(key, "a number"));
        }
    }
    for key in ["exclusiveMinimum", "exclusiveMaximum"] {
        if let Some(v) = schema.get(key) {
            let (valid, expected) = match version {
                Version::V3_0 => (v.is_boolean(), "a boolean in OpenAPI 3.0"),
                Version::V3_1 => (v.is_number(), "a number in OpenAPI 3.1"),
            };
            if !valid {
                return Err(invalid(key, expected));
            }
        }
    }
    for key in [
        "minLength",
        "maxLength",
        "minItems",
        "maxItems",
        "minProperties",
        "maxProperties",
    ] {
        if schema
            .get(key)
            .is_some_and(|v| !v.as_f64().is_some_and(|n| n >= 0.0 && n.fract() == 0.0))
        {
            return Err(invalid(key, "a non-negative integer"));
        }
    }
    if let Some(required) = schema.get("required")
        && !required.as_array().is_some_and(|names| {
            names.iter().all(Value::is_string)
                && names
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<BTreeSet<_>>()
                    .len()
                    == names.len()
        })
    {
        return Err(invalid("required", "an array of unique property names"));
    }
    if schema.get("enum").is_some_and(|v| !v.is_array()) {
        return Err(invalid("enum", "an array"));
    }
    for key in [
        "properties",
        "patternProperties",
        "$defs",
        "definitions",
        "dependentSchemas",
    ] {
        if schema.get(key).is_some_and(|v| !v.is_object()) {
            return Err(invalid(key, "an object of schemas"));
        }
    }
    for key in ["allOf", "oneOf", "anyOf", "prefixItems"] {
        if schema
            .get(key)
            .is_some_and(|v| !v.as_array().is_some_and(|v| !v.is_empty()))
        {
            return Err(invalid(key, "a non-empty array of schemas"));
        }
    }
    Ok(())
}

fn valid_version(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|patch| !patch.is_empty() && patch.bytes().all(|c| c.is_ascii_digit()))
}

fn check_dialect(value: &Value, location: &Location) -> Result<(), OpenApiError> {
    match value.as_str() {
        Some(
            "https://spec.openapis.org/oas/3.1/dialect/base"
            | "https://json-schema.org/draft/2020-12/schema",
        ) => Ok(()),
        _ => Err(OpenApiError::at(
            location,
            "custom schema dialects are not supported yet; expected the OpenAPI 3.1 base or JSON Schema 2020-12 dialect",
        )),
    }
}

fn parse(root: &Location) -> Result<Value, OpenApiError> {
    use serde_saphyr::{DuplicateKeyPolicy, NonFiniteFloatPolicy};
    // A streaming limit also covers files growing after metadata is read.
    use std::io::Read;
    let file =
        std::fs::File::open(&root.file).map_err(|e| OpenApiError::at(root, e.to_string()))?;
    let mut source = String::new();
    file.take(16 * 1024 * 1024 + 1)
        .read_to_string(&mut source)
        .map_err(|e| OpenApiError::at(root, e.to_string()))?;
    if source.len() > 16 * 1024 * 1024 {
        return Err(OpenApiError::at(root, "document exceeds 16 MiB"));
    }
    match root
        .file
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("json") => serde_json::from_str::<UniqueValue>(&source)
            .map(|v| v.0)
            .map_err(|e| OpenApiError::at(root, e.to_string())),
        Some("yaml" | "yml") => {
            let options = serde_saphyr::options! {
                budget: serde_saphyr::budget! { max_documents: 1 },
                emit_comments: false,
                duplicate_keys: DuplicateKeyPolicy::Error,
                strict_booleans: true,
                reject_unsupported_tags: true,
                non_finite_float_policy: NonFiniteFloatPolicy::Reject,
            };
            serde_saphyr::from_str_with_options::<UniqueValue>(&source, options)
                .map(|v| v.0)
                .map_err(|e| OpenApiError::at(root, e.to_string()))
        }
        _ => Err(OpenApiError::at(
            root,
            "expected a .json, .yaml or .yml file",
        )),
    }
}

// Reject duplicates in JSON as well as YAML rather than silently choosing an
// operation/response. Delegate scalar decoding to serde_json's value visitor.
struct UniqueValue(Value);

impl<'de> serde::Deserialize<'de> for UniqueValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = UniqueValue;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a JSON-compatible value")
            }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(UniqueValue(v.into()))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(UniqueValue(v.into()))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(UniqueValue(v.into()))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| UniqueValue(Value::Number(n)))
                    .ok_or_else(|| E::custom("non-finite numbers are not JSON-compatible"))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(UniqueValue(v.into()))
            }
            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(UniqueValue(v.into()))
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Null))
            }
            fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                self.visit_unit()
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(v) = seq.next_element::<UniqueValue>()? {
                    values.push(v.0);
                }
                Ok(UniqueValue(Value::Array(values)))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(serde::de::Error::custom(format!("duplicate key {key:?}")));
                    }
                    values.insert(key, map.next_value::<UniqueValue>()?.0);
                }
                Ok(UniqueValue(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

fn resolve(source: &Location, reference: &str) -> Result<Location, OpenApiError> {
    let (file, fragment) = reference.split_once('#').unwrap_or((reference, ""));
    let file = percent_decode(file).map_err(|m| OpenApiError::at(source, m))?;
    let pointer = percent_decode(fragment).map_err(|m| OpenApiError::at(source, m))?;
    if file.contains([':', '?', '\\', '\0']) || file.starts_with('/') {
        return Err(OpenApiError::at(
            source,
            format!("only relative local-file references are supported: {reference:?}"),
        ));
    }
    if !pointer.is_empty() && !pointer.starts_with('/') {
        return Err(OpenApiError::at(
            source,
            "only JSON Pointer fragments are supported; named anchors are not supported yet",
        ));
    }
    for token in pointer.split('/').skip(1) {
        let mut chars = token.chars();
        while let Some(c) = chars.next() {
            if c == '~' && !matches!(chars.next(), Some('0' | '1')) {
                return Err(OpenApiError::at(
                    source,
                    "invalid JSON Pointer escape; expected ~0 or ~1",
                ));
            }
        }
    }
    let file = if file.is_empty() {
        source.file.clone()
    } else {
        let candidate = source.file.parent().unwrap().join(file);
        std::fs::canonicalize(&candidate).map_err(|e| {
            OpenApiError::at(
                source,
                format!(
                    "cannot resolve {reference:?} ({}): {e}",
                    candidate.display()
                ),
            )
        })?
    };
    Ok(Location { file, pointer })
}

fn percent_decode(value: &str) -> Result<String, &'static str> {
    let mut result = Vec::new();
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = bytes
                .get(i + 1..i + 3)
                .ok_or("invalid percent escape in reference")?;
            let digit = |b: u8| (b as char).to_digit(16).map(|d| d as u8);
            result.push(
                digit(hex[0])
                    .zip(digit(hex[1]))
                    .map(|(a, b)| a * 16 + b)
                    .ok_or("invalid percent escape in reference")?,
            );
            i += 3;
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(result).map_err(|_| "reference is not valid UTF-8")
}

fn children(value: &Value, location: &Location, kind: Kind) -> Vec<(Location, Kind)> {
    let mut result = Vec::new();
    let mut field = |key: &str, child_kind, map: bool, array: bool| {
        if let Some(child) = value.get(key) {
            let base = location.child(key);
            if map {
                if let Some(values) = child.as_object() {
                    result.extend(
                        values
                            .keys()
                            .filter(|k| {
                                !matches!(key, "paths" | "responses") || !k.starts_with("x-")
                            })
                            .map(|k| (base.child(k), child_kind)),
                    );
                }
            } else if array {
                if let Some(values) = child.as_array() {
                    result.extend(
                        (0..values.len()).map(|i| (base.child(&i.to_string()), child_kind)),
                    );
                }
            } else {
                result.push((base, child_kind));
            }
        }
    };
    match kind {
        Kind::Root => {
            field("paths", Kind::PathItem, true, false);
            field("webhooks", Kind::PathItem, true, false);
            if let Some(components) = value.get("components") {
                for (name, kind) in [
                    ("schemas", Kind::Schema),
                    ("parameters", Kind::Parameter),
                    ("headers", Kind::Header),
                    ("requestBodies", Kind::Body),
                    ("responses", Kind::Response),
                    ("examples", Kind::Example),
                    ("links", Kind::Link),
                    ("callbacks", Kind::Callback),
                    ("pathItems", Kind::PathItem),
                    ("securitySchemes", Kind::Security),
                ] {
                    if let Some(map) = components.get(name).and_then(Value::as_object) {
                        // Component names beginning x- are ordinary names.
                        result
                            .extend(map.keys().map(|k| {
                                (location.child("components").child(name).child(k), kind)
                            }));
                    }
                }
            }
        }
        Kind::PathItem => {
            for method in METHODS {
                field(method, Kind::Operation, false, false);
            }
            field("parameters", Kind::Parameter, false, true);
        }
        Kind::Operation => {
            field("parameters", Kind::Parameter, false, true);
            field("requestBody", Kind::Body, false, false);
            field("responses", Kind::Response, true, false);
            field("callbacks", Kind::Callback, true, false);
        }
        Kind::Parameter | Kind::Header => {
            field("schema", Kind::Schema, false, false);
            field("content", Kind::Media, true, false);
            field("examples", Kind::Example, true, false);
        }
        Kind::Body => field("content", Kind::Media, true, false),
        Kind::Response => {
            field("content", Kind::Media, true, false);
            field("headers", Kind::Header, true, false);
            field("links", Kind::Link, true, false);
        }
        Kind::Media => {
            field("schema", Kind::Schema, false, false);
            field("examples", Kind::Example, true, false);
            if let Some(encoding) = value.get("encoding").and_then(Value::as_object) {
                for (name, encoding) in encoding {
                    if let Some(headers) = encoding.get("headers").and_then(Value::as_object) {
                        result.extend(headers.keys().map(|h| {
                            (
                                location
                                    .child("encoding")
                                    .child(name)
                                    .child("headers")
                                    .child(h),
                                Kind::Header,
                            )
                        }));
                    }
                }
            }
        }
        Kind::Callback => {
            if let Some(map) = value.as_object() {
                result.extend(
                    map.keys()
                        .filter(|k| k.as_str() != "$ref" && !k.starts_with("x-"))
                        .map(|k| (location.child(k), Kind::PathItem)),
                );
            }
        }
        Kind::Schema => {
            for key in [
                "properties",
                "patternProperties",
                "$defs",
                "definitions",
                "dependentSchemas",
            ] {
                field(key, Kind::Schema, true, false);
            }
            for key in [
                "items",
                "additionalProperties",
                "unevaluatedProperties",
                "unevaluatedItems",
                "contains",
                "contentSchema",
                "propertyNames",
                "not",
                "if",
                "then",
                "else",
            ] {
                // 3.0 permits boolean additionalProperties even though it
                // does not permit a standalone boolean Schema Object.
                if key == "additionalProperties" && value.get(key).is_some_and(Value::is_boolean) {
                    continue;
                }
                field(key, Kind::Schema, false, false);
            }
            for key in ["allOf", "oneOf", "anyOf", "prefixItems"] {
                field(key, Kind::Schema, false, true);
            }
        }
        Kind::Example | Kind::Link | Kind::Security => {}
    }
    result
}

pub(crate) const METHODS: &[&str] = &[
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];
