use std::path::Path;

use docanvil::openapi::{self, DocumentGraph, Version};
use serde_json::{Value, json};

fn base() -> Value {
    json!({"openapi": "3.1.1", "info": {"title": "Test API", "version": "1"}, "paths": {"/pets": {"get": {"responses": {"200": {"description": "OK"}}}}}})
}

fn write(dir: &Path, name: &str, source: &str) {
    let path = dir.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}

fn load_value(spec: &Value) -> Result<openapi::Api, openapi::OpenApiError> {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "api.json", &spec.to_string());
    openapi::load(&dir.path().join("api.json"))
}

#[test]
fn mixed_files_keep_provenance_cycles_and_effective_request_values() {
    let api = openapi::load(Path::new("tests/fixtures/openapi/store.yaml")).unwrap();
    assert_eq!(api.graph.version, Version::V3_1);
    assert_eq!(api.graph.dependencies().count(), 4);
    assert_eq!(api.operations.len(), 2);
    let get = &api.operations[0];
    assert_eq!(get.operation_id.as_deref(), Some("getPet"));
    assert!(get.source.file.ends_with("paths/pet.yaml"));
    assert_eq!(get.source.pointer, "/get");
    assert_eq!(get.servers[0]["url"], "https://pets.example.test");
    assert_eq!(get.security, vec![json!({"bearer": []})]);
    assert_eq!(get.parameters.len(), 2);
    let expand = get.parameters.iter().find(|p| p.name == "expand").unwrap();
    assert_eq!(expand.definition.value["schema"]["type"], "string");
    assert_eq!(expand.definition.source.pointer, "/get/parameters/0");
    assert!(
        get.parameters
            .iter()
            .find(|p| p.name == "id")
            .unwrap()
            .required
    );
    assert!(get.responses["200"].source.file.ends_with("responses.yaml"));
    assert!(get.responses["200"].reference_origin.is_some());
    let post = &api.operations[1];
    assert_eq!(post.servers, vec![json!({"url": "/"})]);
    assert!(post.security.is_empty());
    assert!(post.security_source.is_some());
    assert!(post.request_body.is_some());
    let recursive = api
        .graph
        .references()
        .find(|r| r.source.pointer == "/properties/friend/$ref")
        .unwrap();
    assert_eq!(recursive.source.file, recursive.target.file);
    assert_eq!(recursive.target.pointer, "");
    assert!(
        !api.graph
            .references()
            .any(|r| r.spelling.contains("literal") || r.spelling.contains("example-data"))
    );
}

#[test]
fn schema_dialects_and_reference_siblings_are_preserved() {
    for version in ["3.0.4", "3.1.1"] {
        let mut spec = base();
        spec["openapi"] = version.into();
        spec["components"] = json!({"schemas": {"Pet": if version == "3.0.4" { json!({"type": "string", "nullable": true}) } else { json!({"type": ["string", "null"]}) }}, "responses": {"OK": {"description": "Original"}}});
        spec["paths"]["/pets"]["get"]["responses"]["200"] =
            json!({"$ref": "#/components/responses/OK", "description": "Override"});
        let api = load_value(&spec).unwrap();
        let schema = api.graph.value(&api.schemas["Pet"]).unwrap();
        assert_eq!(schema, &spec["components"]["schemas"]["Pet"]);
        assert_eq!(
            api.operations[0].responses["200"].value["description"],
            if version == "3.0.4" {
                "Original"
            } else {
                "Override"
            }
        );
    }
}

#[test]
fn json_pointer_escaping_percent_encoding_and_relative_bases() {
    let dir = tempfile::tempdir().unwrap();
    let mut spec = base();
    spec["components"] =
        json!({"schemas": {"Use": {"$ref": "./nested/../a%20b.yaml#/a~1b/~0name/%E2%98%83"}}});
    write(dir.path(), "api.json", &spec.to_string());
    std::fs::create_dir(dir.path().join("nested")).unwrap();
    write(
        dir.path(),
        "a b.yaml",
        "a/b:\n  '~name':\n    '☃':\n      type: string\n",
    );
    let graph = DocumentGraph::load(&dir.path().join("api.json")).unwrap();
    let edge = graph.references().next().unwrap();
    assert_eq!(edge.target.pointer, "/a~1b/~0name/☃");
    assert_eq!(graph.value(&edge.target).unwrap()["type"], "string");
    assert_eq!(graph.dependencies().count(), 2);
}

#[test]
fn errors_point_to_reference_sites_and_never_fetch_remote_sources() {
    for (reference, message) in [
        ("missing.yaml#/Pet", "cannot resolve"),
        ("#/components/schemas/Missing", "has no target"),
        ("https://example.test/api.yaml", "relative local-file"),
        ("//example.test/api.yaml", "relative local-file"),
        ("#/bad~2pointer", "invalid JSON Pointer escape"),
        ("#namedAnchor", "named anchors"),
        ("#/%GG", "invalid percent escape"),
    ] {
        let mut spec = base();
        spec["components"] = json!({"schemas": {"Use": {"$ref": reference}}});
        let error = load_value(&spec).unwrap_err();
        assert_eq!(error.location.pointer, "/components/schemas/Use/$ref");
        assert!(error.message.contains(message), "{error}");
    }
}

#[test]
fn reference_cycles_terminate_but_object_only_cycles_cannot_normalize() {
    let mut spec = base();
    spec["components"] = json!({"responses": {"A": {"$ref": "#/components/responses/B"}, "B": {"$ref": "#/components/responses/A"}}});
    spec["paths"]["/pets"]["get"]["responses"]["200"] = json!({"$ref": "#/components/responses/A"});
    let error = load_value(&spec).unwrap_err();
    assert!(error.message.contains("reference-only cycle"));
}

#[test]
fn payloads_and_extension_objects_do_not_load_references() {
    let mut spec = base();
    spec["components"] = json!({"schemas": {"x-valid-schema-name": {"type": "object", "properties": {"x-valid-property": {"$ref": "#/components/schemas/Other"}}, "default": {"$ref": "default-data.json"}}, "Other": {"type": "string"}}});
    spec["x-metadata"] = json!({"$ref": "extension-data.json"});
    spec["paths"]["/pets"]["get"]["responses"]["200"]["content"] = json!({"application/json": {"examples": {"sample": {"value": {"$ref": "example-data.json"}}}}});
    let api = load_value(&spec).unwrap();
    assert_eq!(api.graph.references().count(), 1);
    assert_eq!(api.graph.dependencies().count(), 1);
}

#[test]
fn invalid_request_shapes_and_duplicate_ids_are_actionable() {
    let mut spec = base();
    spec["paths"]["/pets"]["get"]["parameters"] =
        json!([{ "name": "id", "in": "path", "schema": {"type": "string"}}]);
    assert!(
        load_value(&spec)
            .unwrap_err()
            .message
            .contains("required: true")
    );
    spec["paths"]["/pets"]["get"]["parameters"] =
        json!([{ "name": "q", "in": "query"}, {"name": "q", "in": "query"}]);
    assert!(
        load_value(&spec)
            .unwrap_err()
            .message
            .contains("duplicate parameter")
    );
    spec["paths"]["/pets"]["get"]["parameters"] = json!([]);
    spec["paths"]["/pets"]["get"]["operationId"] = "reused".into();
    spec["paths"]["/pets"]["post"] =
        json!({"operationId": "reused", "responses": {"200": {"description": "OK"}}});
    assert!(
        load_value(&spec)
            .unwrap_err()
            .message
            .contains("duplicate operationId")
    );
}

#[test]
fn parser_rejects_duplicate_keys_and_reports_syntax_errors() {
    for (extension, source) in [
        ("json", "{\"openapi\":\"3.1.1\",\"openapi\":\"3.0.4\"}"),
        ("yaml", "openapi: 3.1.1\nopenapi: 3.0.4\n"),
        ("yaml", "openapi: [unclosed"),
        ("json", "{\"openapi\": "),
        ("yaml", "openapi: 3.1.1\n---\nopenapi: 3.0.4\n"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let file = format!("api.{extension}");
        write(dir.path(), &file, source);
        let error = DocumentGraph::load(&dir.path().join(&file)).unwrap_err();
        assert!(error.location.file.ends_with(file));
        assert!(!error.message.is_empty());
    }
}

#[test]
fn schema_scope_features_are_diagnosed_and_boolean_schemas_are_retained() {
    let mut spec = base();
    spec["components"] = json!({"schemas": {"Allowed": true, "Forbidden": false}});
    let api = load_value(&spec).unwrap();
    assert_eq!(
        api.graph.value(&api.schemas["Forbidden"]).unwrap(),
        &json!(false)
    );
    spec["components"]["schemas"]["Scoped"] =
        json!({"$id": "https://example.test/schema", "type": "string"});
    assert!(
        load_value(&spec)
            .unwrap_err()
            .message
            .contains("schema URI scopes")
    );
}

#[test]
fn boolean_additional_properties_is_valid_in_both_openapi_versions() {
    for version in ["3.0.4", "3.1.1"] {
        let mut spec = base();
        spec["openapi"] = version.into();
        spec["components"] =
            json!({"schemas": {"Closed": {"type": "object", "additionalProperties": false}}});
        assert!(load_value(&spec).is_ok());
        spec["components"]["schemas"]["Closed"] = json!(false);
        assert_eq!(load_value(&spec).is_ok(), version == "3.1.1");
    }
}

#[test]
fn reference_sibling_traversal_depends_on_schema_dialect() {
    let mut spec = base();
    spec["components"] = json!({"schemas": {
        "Pet": {"type": "string"},
        "Use": {"$ref": "#/components/schemas/Pet", "properties": {"child": {"$ref": "missing.yaml"}}}
    }});
    spec["openapi"] = "3.0.4".into();
    assert!(load_value(&spec).is_ok());
    spec["openapi"] = "3.1.1".into();
    assert!(
        load_value(&spec)
            .unwrap_err()
            .message
            .contains("cannot resolve")
    );
}

#[test]
fn unsupported_dialects_and_ambiguous_path_items_are_diagnosed() {
    let mut spec = base();
    spec["jsonSchemaDialect"] = "https://example.test/custom".into();
    assert!(
        load_value(&spec)
            .unwrap_err()
            .message
            .contains("custom schema dialects")
    );
    spec.as_object_mut().unwrap().remove("jsonSchemaDialect");
    spec["components"] =
        json!({"pathItems": {"Pets": {"get": {"responses": {"200": {"description": "OK"}}}}}});
    spec["paths"]["/pets"]["$ref"] = "#/components/pathItems/Pets".into();
    assert!(
        load_value(&spec)
            .unwrap_err()
            .message
            .contains("path-item $ref siblings")
    );
}

#[test]
fn unnormalized_webhooks_and_callbacks_produce_visible_warnings() {
    let mut spec = base();
    spec["webhooks"] = json!({"event": {"post": {"responses": {"204": {"description": "OK"}}}}});
    spec["paths"]["/pets"]["get"]["callbacks"] = json!({"event": {"{$request.query.url}": {"post": {"responses": {"204": {"description": "OK"}}}}}});
    let api = load_value(&spec).unwrap();
    assert_eq!(api.warnings.len(), 2);
    assert!(
        api.warnings
            .iter()
            .any(|w| w.code == "openapi-webhooks-unrendered")
    );
    assert!(
        api.warnings
            .iter()
            .any(|w| w.code == "openapi-callbacks-unrendered")
    );
}

#[test]
fn yaml_aliases_merge_keys_and_scalar_types_survive_loading() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "api.yml",
        r#"
openapi: 3.1.1
info: {title: YAML API, version: '1'}
paths: {}
components:
  schemas:
    Base: &base
      type: object
      properties:
        on: {type: string, example: on}
    Copy:
      <<: *base
      description: Derived schema
"#,
    );
    let api = openapi::load(&dir.path().join("api.yml")).unwrap();
    let schema = api.graph.value(&api.schemas["Copy"]).unwrap();
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["properties"]["on"]["example"], "on");
}
