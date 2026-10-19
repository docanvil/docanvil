---
{"title": "OpenAPI reference", "slug": "openapi", "description": "Publish a local OpenAPI JSON or YAML specification alongside your guides."}
---

# OpenAPI reference

Add an OpenAPI 3.0 or 3.1 specification to your content directory and DocAnvil can generate API reference pages during the normal build. The initial integration produces an overview, operation pages and linked reusable schema pages, with search, navigation and export support.

## Explore the example

Open the [[example-api/index|Documents API example]] in the Master documentation to see the generated output. It has four operations, pagination, bearer authentication, request/response examples, reusable errors and a recursive document schema. The server is fictional; this example does not make live requests.

Jump directly to the [[example-api/operations/createdocument#request-body|create operation’s request body]] to see a generated section link.

Its source is `docs/docs/_examples/documents-api.json` in the DocAnvil repository. Our documentation config generates it under `example-api` for Master only.

## Configure an API

For a project with `content_dir = "docs"`, put your specification at `docs/openapi.json`, then add this top-level entry to `docanvil.toml`:

```toml
[[openapi]]
spec = "openapi.json"
output = "api"
```

`spec` is relative to the selected content directory. `.json`, `.yaml` and `.yml` are accepted. If your specification is beside `docanvil.toml`, use `spec = "../openapi.json"`. Relative references inside the specification resolve against the file containing them.

`output` is a URL namespace, defaults to `api`, and accepts slash-separated letters, digits, hyphens and underscores. Add more `[[openapi]]` entries with distinct namespaces to document multiple APIs.

Run the usual commands:

```sh
docanvil build
docanvil serve
docanvil doctor
```

The overview is published at `api/index.html`. Generated pages stay in memory; DocAnvil does not write Markdown into your source tree. Serve watches the specification and its referenced files, including files outside the content directory after a successful build.

## Link and navigate

Link to the overview from a guide with a wiki-link:

```markdown
Read [[api/index|API reference]].
```

If you use `nav.toml`, include the generated namespace:

```toml
[[nav]]
label = "API reference"
autodiscover = "api"
```

Link directly to a generated section by adding its heading ID to the published slug:

```markdown
[[api/operations/listpets#parameters|Parameters]]
[[api/schemas/pet#properties|Pet fields]]
```

Fragments are case-sensitive and percent-encoded fragments are supported. Repeated headings receive suffixes such as `responses-1`; use the ID from the rendered page. Ordinary Markdown links to generated `.html` pages work too, relative to the published guide URL.

Build and doctor check generated API page/fragment targets, including links in API descriptions. Missing targets emit warnings and fail `--strict`; a failed strict build keeps the previous output (unless you requested `--clean`). Valid fragment links are preserved in AI outputs and scoped to the right chapter in PDF. This check does not validate fragments on authored pages, external URLs or arbitrary assets.

An operation with `operationId: listPets` is published at `api/operations/listpets.html`. IDs and schema names are slugified. IDs that normalize to the same route cause a build error. Operations without an ID use a method/path slug with a short deterministic hash. Changing an operation ID changes its URL; provide redirects when needed. These initial route rules may evolve before V1.3 is released.

The overview groups operations by tag. Declare top-level `tags` in the specification to set group order and provide Markdown descriptions. Undeclared tags follow in alphabetical order, then untagged operations. An operation with several tags appears in each group; repeated tags do not repeat an operation within a group. Declared tags without operations remain visible. If there are no tags, the overview keeps a single operation table. Tag names are case-sensitive and declared names must be unique. Automatic navigation and `autodiscover = "api"` (or `"api/operations"`) group operation links by the same tags beneath `Operations`; schema navigation is unchanged. Multiple tags show the same operation link in each group. Breadcrumbs, previous/next links and exports use its first occurrence. Explicit page/group entries in `nav.toml` keep your chosen layout. Tag labels do not affect operation URLs.

Search, sitemap and enabled AI-readable outputs include generated pages. PDF includes generated pages selected by navigation, like authored chapters. Edit links point to the specification or referenced source file, and Git dates account for the specification's loaded dependencies.

## Versions and languages

Directory versions load from their own content directory: with `docs/v1/` and `docs/v2/`, `spec = "openapi.json"` selects `docs/v1/openapi.json` and `docs/v2/openapi.json`. Git versions read the specification and references from the selected snapshot; all inputs must stay inside that repository snapshot. Current OpenAPI configuration applies to every enabled version, so each must contain the configured input.

To introduce an API example without requiring its input in older releases, add `versions = ["master"]` to that API's `[[openapi]]` entry. The names must be enabled documentation versions. Omitting `versions` generates the API in every enabled version.

One specification is shared across enabled locales. Pages receive the usual version/locale prefixes, but API descriptions remain in the specification's original language. Locale-specific specifications are not yet supported.

## Initial support

Operation pages show methods and paths, descriptions, authentication, servers, parameter types and constraints, request bodies, response headers and schema links. Examples appear separately from schemas. When a body supports multiple media types, content-type tabs let readers switch between them; every section remains readable without JavaScript and in PDF exports.

Schema fields show requiredness, enums, defaults, nullability, common constraints and read/write distinctions. Nested fields and `oneOf`/`anyOf`/`allOf` branches can be expanded; recursive references link to schema pages. Expansion is bounded to keep large or recursive models readable. **View source definition** retains complete definitions for keywords that do not yet have a dedicated presentation.

Internal and relative local-file JSON Pointer references are supported, including recursive schemas. Remote references, named anchors, schema `$id` scopes, dynamic anchors and custom schema dialects fail with source context. Webhooks and callbacks are checked for references but are not rendered as operations; DocAnvil reports warnings, which fail `build --strict` and `doctor --strict`.

Descriptions render as Markdown. Raw HTML is omitted, and DocAnvil includes/directives are not executed from specification descriptions. Live requests, parameter serialization, SDK generation and full schema validation remain outside this initial integration.
