mod integration_helpers;

use docanvil::{cli::build, doctor, error::Error};
use integration_helpers::{build_project_strict, create_project, read_output};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

const CONFIG: &str = "[project]\nname = 'API docs'\n[build]\nsite_url = 'https://example.test/manual/'\nbase_url = '/manual/'\n[llms]\nenabled = true\nfull = true\n[[openapi]]\nspec = 'openapi.json'\noutput = 'api'\n";

fn spec() -> Value {
    serde_json::from_str(include_str!("fixtures/openapi/petstore.json")).unwrap()
}

fn write(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

#[test]
fn main_build_links_indexes_and_exports_generated_json_reference() {
    let dir = create_project(
        CONFIG,
        &[(
            "index.md",
            "# Guide\n\nRead [[api/index|API reference]] or [[api/operations/listpets|List pets]].",
        )],
    );
    write(dir.path(), "docs/openapi.json", &spec().to_string());
    write(
        dir.path(),
        "nav.toml",
        "[[nav]]\npage = 'index'\n[[nav]]\nlabel = 'API'\nautodiscover = 'api'\n",
    );
    build_project_strict(dir.path()).unwrap();
    let html = read_output(dir.path(), "index.html");
    assert!(html.contains("href=\"/manual/api/operations/listpets.html\""));
    let operation = read_output(dir.path(), "api/operations/listpets.html");
    assert!(operation.contains("Parameters"));
    assert!(operation.contains("href=\"/manual/api/schemas/pet.html\""));
    let schema = read_output(dir.path(), "api/schemas/pet.html");
    assert!(schema.contains("Properties"));
    assert!(schema.contains("href=\"/manual/api/schemas/pet.html\""));
    for file in ["search-index.json", "sitemap.xml", "llms-full.txt"] {
        assert!(
            read_output(dir.path(), file).contains("api/operations/listpets"),
            "{file}"
        );
        assert!(
            read_output(dir.path(), file).contains("api/schemas/pet"),
            "{file}"
        );
    }
    assert!(!dir.path().join("docs/api").exists());
    let (diagnostics, summary) = doctor::run_checks(dir.path(), true);
    assert_eq!(summary.errors, 0, "{diagnostics:?}");
    assert!(!diagnostics.iter().any(|d| d.check == "broken-wikilink"));
}

#[test]
fn collisions_and_invalid_specs_leave_existing_output_intact() {
    let dir = create_project(CONFIG, &[("index.md", "# Guide")]);
    write(dir.path(), "docs/openapi.json", &spec().to_string());
    build_project_strict(dir.path()).unwrap();
    let previous = read_output(dir.path(), "api/index.html");
    write(dir.path(), "docs/api/index.md", "# Authored reference");
    assert!(
        build_project_strict(dir.path())
            .unwrap_err()
            .to_string()
            .contains("collides")
    );
    assert_eq!(previous, read_output(dir.path(), "api/index.html"));
    let (_, summary) = doctor::run_checks(dir.path(), true);
    assert!(summary.errors > 0);
    // Invalid input independently produces a useful main-tool error.
    write(dir.path(), "docs/openapi.json", "{bad json");
    assert!(
        build_project_strict(dir.path())
            .unwrap_err()
            .to_string()
            .contains("openapi.json")
    );
    assert_eq!(previous, read_output(dir.path(), "api/index.html"));
}

#[test]
fn source_links_point_at_specifications_and_descriptions_cannot_include_files() {
    let config =
        format!("{CONFIG}\n[edit]\nrepo = 'https://github.com/example/project'\nroot = ''\n");
    let dir = create_project(&config, &[("index.md", "# Guide")]);
    let mut api = spec();
    api["paths"]["/pets"]["get"]["description"] =
        ":::include{file=\"../secret.txt\"}\n\n<script>bad()</script>\n\n**Safe Markdown**".into();
    write(dir.path(), "docs/openapi.json", &api.to_string());
    write(dir.path(), "secret.txt", "DO_NOT_INCLUDE_THIS");
    build_project_strict(dir.path()).unwrap();
    let html = read_output(dir.path(), "api/operations/listpets.html");
    assert!(
        html.replace("&#x2F;", "/")
            .contains("https://github.com/example/project/edit/main/docs/openapi.json")
    );
    assert!(html.contains("<strong>Safe Markdown</strong>"));
    assert!(!html.contains("DO_NOT_INCLUDE_THIS"));
    assert!(!html.contains("<script>bad()"));
}

#[test]
fn yaml_references_are_watched_and_rebuilt_by_normal_serve_build() {
    let config = CONFIG.replace("openapi.json", "openapi.yaml");
    let dir = create_project(&config, &[("index.md", "# Guide")]);
    write(
        dir.path(),
        "docs/openapi.yaml",
        "openapi: 3.1.1\ninfo: {title: YAML API, version: '1'}\npaths: {}\ncomponents:\n  schemas:\n    Pet:\n      $ref: '../specs/pet.json'\n",
    );
    write(
        dir.path(),
        "specs/pet.json",
        r#"{"type":"object","description":"Original pet"}"#,
    );
    let output = dir.path().join("serve-output");
    let deps = build::run_with_options(dir.path(), &output, true, None).unwrap();
    assert!(deps.contains(&dir.path().join("specs/pet.json").canonicalize().unwrap()));
    assert!(deps.contains(&dir.path().join("docs/openapi.yaml").canonicalize().unwrap()));
    write(
        dir.path(),
        "specs/pet.json",
        r#"{"type":"object","description":"Changed pet"}"#,
    );
    build::run_with_options(dir.path(), &output, true, None).unwrap();
    assert!(
        fs::read_to_string(output.join("api/schemas/pet.html"))
            .unwrap()
            .contains("Changed pet")
    );
}

#[test]
fn directory_versions_and_locales_use_the_selected_specs_and_urls() {
    let config = format!(
        "{CONFIG}\n[version]\nenabled = ['v1', 'v2']\ncurrent = 'v2'\n[locale]\nenabled = ['en', 'fr']\ndefault = 'en'\n"
    );
    let dir = create_project(
        &config,
        &[
            ("v1/index.md", "# Guide one\n[[api/index]]"),
            ("v2/index.md", "# Guide two\n[[api/index]]"),
        ],
    );
    for version in ["v1", "v2"] {
        let mut api = spec();
        api["info"]["title"] = format!("API {version}").into();
        write(
            dir.path(),
            &format!("docs/{version}/openapi.json"),
            &api.to_string(),
        );
        // Explicit translations keep ordinary locale coverage checks quiet.
        write(
            dir.path(),
            &format!("docs/{version}/index.fr.md"),
            "# Guide traduit\n[[api/index]]",
        );
    }
    build_project_strict(dir.path()).unwrap();
    for version in ["v1", "v2"] {
        for locale in ["en", "fr"] {
            let html = read_output(dir.path(), &format!("{version}/{locale}/api/index.html"));
            assert!(html.contains(&format!("API {version}")));
            assert!(html.contains(&format!(
                "/manual/{version}/{locale}/api/operations/listpets.html"
            )));
            assert!(
                read_output(dir.path(), &format!("{version}/{locale}/search-index.json"))
                    .contains("listpets")
            );
        }
    }
    let (diagnostics, summary) = doctor::run_checks(dir.path(), true);
    assert_eq!(summary.errors, 0, "{diagnostics:?}");
}

#[test]
fn warnings_participate_in_build_strict_mode_and_doctor() {
    let dir = create_project(CONFIG, &[("index.md", "# Guide")]);
    let mut api = spec();
    api["webhooks"] = json!({});
    write(dir.path(), "docs/openapi.json", &api.to_string());
    assert!(matches!(
        build_project_strict(dir.path()),
        Err(Error::StrictWarnings(1))
    ));
    let (diagnostics, _) = doctor::run_checks(dir.path(), true);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.check == "openapi-webhooks-unrendered")
    );
}

#[test]
fn several_apis_have_independent_namespaces_and_slug_collisions_fail() {
    let config = format!("{CONFIG}\n[[openapi]]\nspec = 'second.json'\noutput = 'other-api'\n");
    let dir = create_project(&config, &[("index.md", "# Guide")]);
    write(dir.path(), "docs/openapi.json", &spec().to_string());
    write(dir.path(), "docs/second.json", &spec().to_string());
    build_project_strict(dir.path()).unwrap();
    assert!(
        dir.path()
            .join("dist/other-api/operations/listpets.html")
            .is_file()
    );
    let mut api = spec();
    api["paths"]["/pets"]["post"]["operationId"] = "list-pets".into();
    api["paths"]["/pets"]["get"]["operationId"] = "list_pets".into();
    write(dir.path(), "docs/openapi.json", &api.to_string());
    assert!(
        build_project_strict(dir.path())
            .unwrap_err()
            .to_string()
            .contains("route collision")
    );
}

#[test]
fn git_snapshots_use_historical_json_and_referenced_schemas() {
    let dir = create_project(CONFIG, &[("index.md", "# Guide\n[[api/index]]")]);
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(dir.path())
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    let mut api = spec();
    api["info"]["title"] = "Historical API".into();
    api["components"]["schemas"]["Pet"] = json!({"$ref": "schemas/pet.json"});
    write(dir.path(), "docs/openapi.json", &api.to_string());
    write(
        dir.path(),
        "docs/schemas/pet.json",
        r#"{"type":"object","description":"Historical schema"}"#,
    );
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.test",
        "commit",
        "-qm",
        "Snapshot",
    ]);
    git(&["tag", "v1"]);
    api["info"]["title"] = "Working API".into();
    write(dir.path(), "docs/openapi.json", &api.to_string());
    write(
        dir.path(),
        "docs/schemas/pet.json",
        r#"{"type":"object","description":"Working schema"}"#,
    );
    write(
        dir.path(),
        "docanvil.toml",
        &format!(
            "{CONFIG}\n[version]\nenabled = ['v1', 'dev']\ncurrent = 'dev'\n[version.git]\nrefs = {{v1 = 'refs/tags/v1'}}\nworking_tree = 'dev'\n"
        ),
    );
    build_project_strict(dir.path()).unwrap();
    assert!(read_output(dir.path(), "v1/api/index.html").contains("Historical API"));
    assert!(read_output(dir.path(), "v1/api/schemas/pet.html").contains("Historical schema"));
    assert!(!read_output(dir.path(), "v1/api/schemas/pet.html").contains("Working schema"));
    assert!(read_output(dir.path(), "dev/api/index.html").contains("Working API"));
    assert!(read_output(dir.path(), "dev/api/schemas/pet.html").contains("Working schema"));
    let (diagnostics, summary) = doctor::run_checks(dir.path(), true);
    assert_eq!(summary.errors, 0, "{diagnostics:?}");
}

#[test]
fn normal_cli_build_and_doctor_accept_openapi_configuration() {
    let dir = create_project(CONFIG, &[("index.md", "# Guide\n[[api/index]]")]);
    write(dir.path(), "docs/openapi.json", &spec().to_string());
    assert_cmd::cargo::cargo_bin_cmd!("docanvil")
        .args(["build", "--strict", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    assert_cmd::cargo::cargo_bin_cmd!("docanvil")
        .args(["doctor", "--format", "checkstyle", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    assert!(dir.path().join("dist/api/index.html").is_file());
}

#[test]
fn version_selection_allows_adding_apis_after_older_releases() {
    let config = format!(
        "{}\n[version]\nenabled = ['v1', 'v2']\ncurrent = 'v2'\n",
        CONFIG.replace("output = 'api'", "output = 'api'\nversions = ['v2']")
    );
    let dir = create_project(
        &config,
        &[
            ("v1/index.md", "# Old guide"),
            ("v2/index.md", "# New guide\n[[api/index]]"),
        ],
    );
    write(dir.path(), "docs/v2/openapi.json", &spec().to_string());
    build_project_strict(dir.path()).unwrap();
    assert!(!dir.path().join("dist/v1/api/index.html").exists());
    assert!(read_output(dir.path(), "v2/api/index.html").contains("Pet store API"));
    assert!(!read_output(dir.path(), "v1/search-index.json").contains("listpets"));
    assert!(read_output(dir.path(), "v2/search-index.json").contains("listpets"));
    let (diagnostics, summary) = doctor::run_checks(dir.path(), true);
    assert_eq!(summary.errors, 0, "{diagnostics:?}");
    write(
        dir.path(),
        "docanvil.toml",
        &config.replace("versions = ['v2']", "versions = ['missing']"),
    );
    assert!(
        build_project_strict(dir.path())
            .unwrap_err()
            .to_string()
            .contains("enabled documentation versions")
    );
}

#[test]
fn operation_presentation_separates_fields_examples_and_collapsed_source() {
    let dir = create_project(CONFIG, &[("index.md", "# Guide")]);
    write(
        dir.path(),
        "docs/openapi.json",
        include_str!("../docs/docs/_examples/documents-api.json"),
    );
    build_project_strict(dir.path()).unwrap();
    let html = read_output(dir.path(), "api/operations/createdocument.html");
    let body = html
        .split("<details class=\"details api-source\">")
        .next()
        .unwrap();
    assert!(body.contains("api-method-post"));
    assert!(body.contains("Required body"));
    assert!(body.contains("Bearer token in the Authorization header"));
    assert!(body.contains("Maximum length: 200"));
    assert!(body.contains("Allowed values:"));
    assert!(body.contains("<td>body</td>"));
    assert!(body.contains("<td>Yes</td>"));
    assert!(body.contains("Response headers"));
    assert!(body.contains("Location"));
    assert!(body.contains("Example"));
    assert!(!body.contains("&quot;required&quot;"));
    assert!(!body.contains("&quot;properties&quot;"));
    assert!(html.contains("View source definition"));
    assert!(!html.contains("api-source\" open"));
    assert!(body.contains("data-api-media"));
    assert!(body.contains("aria-controls="));
    assert!(body.contains("application/yaml"));
    // The source HTML exposes every media panel; JavaScript alone selects one.
    assert!(!body.contains("data-api-panel hidden"));
    assert!(body.contains("href=\"/manual/api/schemas/createdocument.html\""));
    let source = read_output(dir.path(), "llms-full.txt");
    assert!(source.contains("View source definition"));
    assert!(source.contains("Maximum length: 200"));
    assert!(source.contains("\"properties\""));
}

#[test]
fn schema_presentation_preserves_composition_nullability_and_recursive_links() {
    let dir = create_project(CONFIG, &[("index.md", "# Guide")]);
    let mut api = spec();
    api["components"]["schemas"]["Variant"] = json!({
        "oneOf": [{"type": ["string", "null"], "enum": ["public", null]}, {"$ref": "#/components/schemas/Pet"}]
    });
    api["components"]["schemas"]["Pet"]["properties"]["secret"] =
        json!({"type":"string", "writeOnly":true});
    write(dir.path(), "docs/openapi.json", &api.to_string());
    build_project_strict(dir.path()).unwrap();
    let html = read_output(dir.path(), "api/schemas/variant.html");
    assert!(html.contains("One of — exactly one schema applies"));
    assert!(html.contains("string or null"));
    assert!(html.contains("Allowed values:"));
    assert!(html.contains("href=\"/manual/api/schemas/pet.html\""));
    let html = read_output(dir.path(), "api/schemas/pet.html");
    assert!(html.contains("Read only"));
    assert!(html.contains("Write only"));
    assert!(html.contains("href=\"/manual/api/schemas/pet.html\""));
}
