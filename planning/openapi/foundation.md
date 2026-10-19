# OpenAPI loading foundation

The library in `src/openapi/` loads local specifications, retains a reference graph and normalizes request metadata independently of page presentation. Native page planning and rendering consume this shared model.

Native publishing is now integrated through `[[openapi]]`. See the [user guide](../../docs/docs/guides/openapi.md) for configuration and the normal build/serve/doctor workflow. `src/openapi/publish.rs` plans in-memory overview, operation and schema pages after authored front matter establishes its routes. `render.rs` produces readable field/schema content and trusted disclosure/media controls after safely rendering specification Markdown. `PageInventory.generated` records both HTML and export Markdown plus dependencies; `pipeline::process_page*` selects authored or generated rendering. Build, version prescans, site diagnostics and PDF share source assembly.

```rust
let api = docanvil::openapi::load(std::path::Path::new("openapi.yaml"))?;
for operation in &api.operations {
    println!("{} {}", operation.method, operation.path);
}
```

Preview the Documents API in our Master documentation with `cargo run -- serve --path docs`, then open `/master/en/example-api/operations/createdocument.html`. JSON, YAML and mixed-file fixtures live under `tests/fixtures/openapi/`; focused checks are `cargo test --test openapi_integration --test openapi_build`.

## Implemented boundaries

| Area | Behavior |
| --- | --- |
| Inputs | Local `.json`, `.yaml`, `.yml`, OpenAPI 3.0.x and 3.1.x. Basic structure checks; not complete specification validation. |
| Parsing | JSON through Serde; YAML through `serde-saphyr` 2.0. Duplicate map keys fail in both formats. YAML aliases/merge keys are supported within parser budgets; YAML 1.1 boolean inference is disabled. Unsupported tags and non-finite numbers fail. |
| References | `$ref` fields in recognized OpenAPI/schema contexts; internal and relative-file JSON Pointer references, including percent decoding and `~0`/`~1` escapes. Nested references resolve against the containing file. Local files may be outside the root specification's directory. No remote fetching. |
| Identity | Canonical file path plus JSON Pointer. Documents are loaded once. Graph traversal uses a work queue and visited set. Edges preserve original spelling and target identity without substituting or recursively expanding schemas. |
| Source attribution | Operations, parameters, resolved request/response objects and reference edges retain file/pointer locations. Reference-site overrides retain their origin. Parser syntax errors include parser location context; semantic errors use file and JSON Pointer, not line/column spans. |
| Operations | Eight HTTP methods from top-level paths, including referenced Path Items. Operation IDs are optional but must be unique when present. Summary, description, tags and deprecation are retained. |
| Parameters | Resolve Parameter References, merge by `(in, name)`, apply operation overrides and reject duplicates within a list. Validate locations and required path parameters. Original schema/content/style metadata remains available for future rendering/serialization. |
| Servers/security | Apply operation → path → root server precedence, with `/` as the default for missing/empty server lists. Apply operation → root security precedence; explicit `security: []` removes inherited requirements. Retain locations so relative server URLs can be interpreted later. |
| Bodies/responses | Resolve their Reference Objects, preserve media types/examples/schemas, validate basic required fields and response status keys. Response ranges/defaults remain distinct. |
| Dialects | Keep original schema values: 3.0 `nullable` is not converted into 3.1 union types. 3.1 boolean schemas are accepted. Reference Object siblings are ignored in 3.0; 3.1 description/summary overrides are retained. Schema `$ref` sibling traversal applies in 3.1. This is not a JSON Schema evaluator. |
| Warnings | Webhooks and operation callbacks are traversed for references but are not normalized as operations. Unrendered schema features and unknown keywords produce `openapi-schema-unrendered`; keywords outside the selected dialect produce `openapi-schema-dialect`. Native builds and doctor report these source-located warnings, including strict-mode failures. |
| Limits | At most 256 documents, 16 MiB of UTF-8 source per document, JSON's recursion limit, and the YAML parser's default budgets with a single-document limit. These are initial implementation limits, not configurable product settings. |

Recognized traversal includes components, schemas, parameters, headers, request bodies, responses, media types/encodings, Example References, Link References, callbacks and webhooks. Literal examples, defaults, enum/const values, extension payloads and Example Object `value` are not scanned for references. Property/component names beginning `x-` remain ordinary names.

## Explicitly unsupported or deferred

- Remote/absolute-file references, named anchors, `$id` URI scopes, `$anchor`, dynamic/recursive anchor mechanisms and custom schema dialects fail explicitly. The accepted explicit 3.1 dialect identifiers are the OpenAPI 3.1 base dialect and JSON Schema 2020-12. Renderer-consumed keyword shapes and dialect-specific types/exclusive bounds are checked; full dialect validation remains deferred. Ignored 3.0 Reference Object siblings are not validated as schemas.
- Path-item `$ref` with sibling fields other than extensions fails, rather than choosing an ambiguous merge rule.
- Reference-only cycles remain representable in the graph; attempting to normalize a request/response/parameter chain without a concrete target fails. Recursive schema properties remain finite edges.
- Discriminator mappings, Link Object `operationRef`, external example URLs and external documentation URLs are retained as source data; this loader does not follow or validate them as `$ref` edges.
- Complete parameter/schema/security validation, path-template matching, parameter serialization, server-variable expansion, example synthesis and request execution remain deferred. A successfully loaded API is not a certification that the input fully conforms to OpenAPI.
- The native renderer provides requiredness, types, common field constraints, enums/defaults/nullability, read/write distinctions, bounded nested/composed schema expansion, authentication, response headers and separate examples. Multiple media types have progressive keyboard-accessible tabs; all media remain visible without JavaScript and in print. Complete definitions are retained in collapsed source disclosures, rather than interpreting every schema keyword. Route rules are initial V1.3 rules and still need review with representative APIs. Specification download/bundling, locale-specific specifications and automatic redirects for changed operation IDs remain deferred.
- Specifications are selected relative to each version's content directory, using the current configuration. Optional `versions` on an API entry restricts it to named enabled versions, allowing new APIs/examples without inputs in older snapshots. One specification is repeated across enabled locales without translation. Serve tracks dependencies after successful builds; a new missing reference outside watched locations may require another specification edit after the file is created.

## Schema presentation regression coverage

The [public schema matrix](../../docs/docs/guides/openapi.md#schema-support-matrix) is the supported-or-diagnosed contract for the first static release. It distinguishes presentation, source-only annotations, source-only features with warnings, and hard errors. A rendered keyword does not imply complete dialect support or payload validation.

| Behavior | Regression coverage |
| --- | --- |
| Composition, union nullability, read/write annotations and recursive schema links | `schema_presentation_preserves_composition_nullability_and_recursive_links` in `tests/openapi_build.rs` |
| 3.0 property references ignore sibling descriptions, constraints and nested properties, while retaining the target's constraints and the containing object's requiredness | `property_reference_siblings_follow_the_schema_version` in `tests/openapi_build.rs`; checks HTML and AI output |
| 3.1 property references retain sibling descriptions, constraints and nested properties alongside the target's constraints | The same paired-version regression |
| Common constraints, object requiredness, array items, open/closed/typed maps, composition, nullable types and boolean schemas | `schema_matrix_constraints_objects_arrays_and_boolean_schemas_are_published` in `tests/openapi_build.rs` |
| Reference-chain sibling constraints, reference-only cycles and composition-level required names | `chained_property_references_preserve_dialect_constraints_and_terminate_cycles` in `tests/openapi_build.rs` |
| Malformed types, scalar annotations, required-name lists, bounds and schema containers have precise keyword locations | `malformed_rendered_schema_keywords_report_the_exact_location` and `schema_types_and_exclusive_bounds_follow_the_selected_dialect` in `tests/openapi_integration.rs` |
| Source-only constraints and unknown/legacy keywords warn once per schema location | `source_only_schema_keywords_have_source_located_warnings` in `tests/openapi_integration.rs` |
| Literal values/extensions and ignored 3.0 reference siblings do not become schemas; referenced files retain provenance and are visited once | `schema_diagnostics_ignore_literals_extensions_and_ignored_reference_siblings` and `advanced_schema_references_are_resolved_once_with_file_provenance` in `tests/openapi_integration.rs` |
| URI scopes/custom dialects fail; cross-dialect keywords warn | `schema_uri_scopes_and_custom_dialects_are_rejected_at_the_keyword` and `keywords_from_another_schema_dialect_warn_instead_of_claiming_support` in `tests/openapi_integration.rs` |
| Warnings reach strict build/doctor; source-only values remain in HTML/AI output and failed strict builds preserve existing output | `schema_source_only_and_dialect_warnings_reach_build_doctor_and_exports` in `tests/openapi_build.rs` |

Ignored 3.0 siblings remain available in the source disclosure. `title`, schema examples, `$comment` and `x-…` extension payloads may remain source-only without warnings as documented in the matrix. `$defs` stores reusable schemas, whose references are loaded without a warning. Other unrendered semantics are diagnosed instead of implying complete documentation. Unknown 3.1 keywords are legal extension points, so their warning indicates presentation coverage rather than invalid input. References under unknown keywords are not traversed.

Checks cover the shapes used by rendering, not every specification requirement: constraint consistency, positive `multipleOf`, example validity, format/regex evaluation, contextual read/write filtering and complete schema/object validation remain deferred. Schema-valued recognized keywords are traversed even when their presentation is source-only; those targets can still produce parse/reference/schema-shape errors. No composition flattening or discriminator selection is attempted. Required-name lists stay visible even when corresponding properties live in another branch.

## Why a document graph and a small model

The graph preserves recursive schemas and source provenance without an infinitely expanded tree. A renderer can link to a schema identity, and serve can watch the exact files read. The operation model contains effective request metadata without depending on HTML or a UI component, leaving room for a later request runner.

`serde-saphyr` supplies YAML parsing, duplicate-key policy, merge/alias handling and parser budgets; it does not decide OpenAPI semantics. The OpenAPI layer uses `serde_json::Value` as its JSON-compatible representation, so unsupported/new fields remain accessible instead of being lost through a closed typed schema. A future dialect-aware schema model can build on these preserved values.

Native integration now exercises guide links, navigation, search, sitemap/AI outputs, source edit links, local dependency rebuilds, directory versions, Git snapshots, locale prefixes and PDF preparation. Follow-up work should exercise larger specifications, refine schema dialect coverage, and review URL stability and validation. Generated-page route collisions fail before staging output is published. Descriptions use Markdown with raw HTML disabled and bypass DocAnvil includes/directives.

Semantics are based on the official [OpenAPI 3.0.4](https://spec.openapis.org/oas/v3.0.4.html) and [3.1.1](https://spec.openapis.org/oas/v3.1.1.html) specifications. Parser options are documented in the [serde-saphyr API](https://docs.rs/serde-saphyr/2.0.0/serde_saphyr/struct.Options.html).

Repository architecture and dependency notes are maintained in `CLAUDE.md`.

The overview groups operations under declared tags in specification order, with their descriptions, then undeclared tags in lexical order and untagged operations last. Multiple tags list an operation in each matching group without changing its route. Declared tag names are validated for uniqueness and metadata shape. Automatic navigation and folder autodiscovery use `PageInventory.generated_nav` to replace generated operation leaves with tag groups while preserving authored siblings. Locale/version prefixes continue to come from the selected inventory. Explicit navigation entries retain author control. Repeated operation links use the first occurrence for breadcrumbs, linear previous/next order and exports.

## Navigation method badges

Operation methods are retained by canonical slug in `PageInventory.operation_methods`. The build passes each scope's map to `project::render_nav_with_methods`, including the current version's 404 sidebar. Navigation trees and their labels remain unchanged, so explicit page entries, linked groups, automatic discovery and repeated tag membership share the same metadata. The default sidebar renders real-text method badges, per-method classes and `data-http-method` hooks; schema/authored links remain ordinary links. A matching uppercase method prefix followed by whitespace is omitted from rendered operation-link text, keeping `GET /path` from repeating the GET badge; canonical titles and breadcrumb/export labels are preserved. See the user guide's [navigation badge section](../../docs/docs/guides/openapi.md#navigation-method-badges) for CSS selectors and palette overrides.

## Generated links and anchors

Generated HTML receives final heading IDs once during page planning. Heading IDs avoid existing native/control IDs and naturally suffixed headings; publishing, doctor, search and exports then consume the same IDs. Wiki-links accept `page#fragment`; fragments are case-sensitive and percent-decoded once for validation. AI outputs retain them. PDF scopes headings/native IDs and their local links/control relationships to their chapter, and rewrites relative or site-root page links with fragments (discarding query strings).

`src/pipeline/links.rs` shares generated-target checks between publishing and doctor. Ordinary/reference-style Markdown links are inspected in rendered HTML; relative URLs resolve against the published page, not the specification source file. Missing routes inside the selected API namespaces and missing generated-page IDs produce warnings that fail strict mode. Doctor reports `broken-generated-link` with the guide/specification file; wiki-links in authored source also retain line numbers. Strict warning failures are checked before staging is synced to output. `--clean` still explicitly removes previous output before building.

This is scoped validation for generated API targets, not a complete site link crawler. Authored-page fragments, external URLs, arbitrary assets, cross-version target inventories and dynamically created component links are not fully audited. Browser/print review and a larger specification remain release acceptance work.
