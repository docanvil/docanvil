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
| Warnings | Webhooks and operation callbacks are traversed for references but are not normalized as operations; callers receive explicit warnings. Native builds and doctor report them, including strict-mode failures. |
| Limits | At most 256 documents, 16 MiB of UTF-8 source per document, JSON's recursion limit, and the YAML parser's default budgets with a single-document limit. These are initial implementation limits, not configurable product settings. |

Recognized traversal includes components, schemas, parameters, headers, request bodies, responses, media types/encodings, Example References, Link References, callbacks and webhooks. Literal examples, defaults, enum/const values, extension payloads and Example Object `value` are not scanned for references. Property/component names beginning `x-` remain ordinary names.

## Explicitly unsupported or deferred

- Remote/absolute-file references, named anchors, `$id` URI scopes, `$anchor`, dynamic/recursive anchor mechanisms and custom schema dialects fail explicitly. The accepted explicit 3.1 dialect identifiers are the OpenAPI 3.1 base dialect and JSON Schema 2020-12. Schema dialect keyword validation remains deferred.
- Path-item `$ref` with sibling fields other than extensions fails, rather than choosing an ambiguous merge rule.
- Reference-only cycles remain representable in the graph; attempting to normalize a request/response/parameter chain without a concrete target fails. Recursive schema properties remain finite edges.
- Discriminator mappings, Link Object `operationRef`, external example URLs and external documentation URLs are retained as source data; this loader does not follow or validate them as `$ref` edges.
- Complete parameter/schema/security validation, path-template matching, parameter serialization, server-variable expansion, example synthesis and request execution remain deferred. A successfully loaded API is not a certification that the input fully conforms to OpenAPI.
- The native renderer provides requiredness, types, common field constraints, enums/defaults/nullability, read/write distinctions, bounded nested/composed schema expansion, authentication, response headers and separate examples. Multiple media types have progressive keyboard-accessible tabs; all media remain visible without JavaScript and in print. Complete definitions are retained in collapsed source disclosures, rather than interpreting every schema keyword. Route rules are initial V1.3 rules and still need review with representative APIs. Specification download/bundling, tag grouping, locale-specific specifications and automatic redirects for changed operation IDs remain deferred.
- Specifications are selected relative to each version's content directory, using the current configuration. Optional `versions` on an API entry restricts it to named enabled versions, allowing new APIs/examples without inputs in older snapshots. One specification is repeated across enabled locales without translation. Serve tracks dependencies after successful builds; a new missing reference outside watched locations may require another specification edit after the file is created.

## Why a document graph and a small model

The graph preserves recursive schemas and source provenance without an infinitely expanded tree. A renderer can link to a schema identity, and serve can watch the exact files read. The operation model contains effective request metadata without depending on HTML or a UI component, leaving room for a later request runner.

`serde-saphyr` supplies YAML parsing, duplicate-key policy, merge/alias handling and parser budgets; it does not decide OpenAPI semantics. The OpenAPI layer uses `serde_json::Value` as its JSON-compatible representation, so unsupported/new fields remain accessible instead of being lost through a closed typed schema. A future dialect-aware schema model can build on these preserved values.

Native integration now exercises guide links, navigation, search, sitemap/AI outputs, source edit links, local dependency rebuilds, directory versions, Git snapshots, locale prefixes and PDF preparation. Follow-up work should exercise larger specifications, refine tag navigation and schema dialect coverage, and review URL stability and validation. Generated-page route collisions fail before staging output is published. Descriptions use Markdown with raw HTML disabled and bypass DocAnvil includes/directives.

Semantics are based on the official [OpenAPI 3.0.4](https://spec.openapis.org/oas/v3.0.4.html) and [3.1.1](https://spec.openapis.org/oas/v3.1.1.html) specifications. Parser options are documented in the [serde-saphyr API](https://docs.rs/serde-saphyr/2.0.0/serde_saphyr/struct.Options.html).

Repository architecture and dependency notes are maintained in `CLAUDE.md`.
