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

### Related guides

Add `api_operations` to a guide's JSON front matter to list it beneath the header of the operations it explains:

```markdown
---
{
  "title": "Create your first document",
  "api_operations": [
    "example-api/operations/createdocument",
    "example-api/operations/getdocument"
  ]
}
---

# Create your first document
```

Use complete published operation slugs, including the API output namespace. Omit `.html`, fragments, wiki-link brackets and version/locale prefixes. This also works for operations without an `operationId`: use their generated method/path slug. One guide can relate to operations in several APIs, and several guides can relate to the same operation. Repeated entries produce one link.

The **Related guides** block uses each guide's published title and URL, including front-matter slug overrides, and sorts links by title. It appears only when there are related guides. Links stay in the operation's documentation version and locale; translations opt in through their own front matter. There is no cross-language fallback. Git snapshots use the associations and titles from that snapshot.

Draft guides are never listed, including when serving or building with drafts enabled. With OpenAPI configured, malformed `api_operations` values and targets that are missing or are not generated operations produce source-located `openapi-related-guide` warnings in build and doctor. Strict mode rejects these warnings; ordinary builds skip invalid entries and keep valid ones. Without `[[openapi]]`, this field is inactive.

These are static links: no JavaScript is required, and PDF/AI outputs retain them. Guide changes participate in normal serve rebuilding. The default theme exposes `.api-related-guides` for styling. Try the [[guides/documents-api|Documents API walkthrough]], then follow its related link back from the [[example-api/operations/createdocument|create operation]].

### Navigation method badges

Operation links display a compact HTTP-method badge in the default sidebar. This works with tag groups, automatic discovery, explicit page entries and linked groups, including custom labels. Schema and overview links keep their usual appearance. Badges use real text, so the method remains available without JavaScript and to assistive technology; navigation labels in breadcrumbs and exports are unchanged.

When a label begins with its uppercase HTTP method followed by whitespace, that prefix is omitted from the sidebar text: `GET /pets` becomes a GET badge followed by `/pets`. Page headings keep their full titles.

For custom styling, the operation's `<li>` has `nav-openapi-operation` and a method class such as `nav-method-get` or `nav-method-post`. Its link has `nav-operation-link` and `data-http-method="GET"`; the badge and label use `nav-method-badge` and `nav-label`. Method metadata comes from the selected version's specification, independently of page titles or URL spelling.

Add overrides to your custom theme CSS, for example:

```css
.sidebar .nav-method-post {
  --nav-method-bg: #d1fae5;
  --nav-method-color: #065f46;
}

.sidebar .nav-method-badge {
  border-radius: 0.25rem;
}
```

To hide the badges while retaining the method classes and data attributes, use `.sidebar .nav-method-badge { display: none; }`.

## Versions and languages

Directory versions load from their own content directory: with `docs/v1/` and `docs/v2/`, `spec = "openapi.json"` selects `docs/v1/openapi.json` and `docs/v2/openapi.json`. Git versions read the specification and references from the selected snapshot; all inputs must stay inside that repository snapshot. Current OpenAPI configuration applies to every enabled version, so each must contain the configured input.

To introduce an API example without requiring its input in older releases, add `versions = ["master"]` to that API's `[[openapi]]` entry. The names must be enabled documentation versions. Omitting `versions` generates the API in every enabled version.

One specification is shared across enabled locales. Pages receive the usual version/locale prefixes, but API descriptions remain in the specification's original language. Locale-specific specifications are not yet supported.

## API page design

Generated reference pages inherit your theme's colours and fonts. Operation headers group the method, endpoint, description and tags. Authentication/server cards and separate response panels make longer pages easier to scan. Overview and schema pages share the table and section styling. Content stays readable without JavaScript and in print.

To customise the operation header's markup, eject its component:

```sh
docanvil component eject api-operation
```

Edit `theme/components/api-operation.html`. It receives `attrs.title`, `attrs.method`, `attrs.path`, `attrs.deprecated` (`"true"` or `"false"`), `attrs.endpoint` (whether to repeat the endpoint beneath the title), a `tags` array, and the safely rendered description as `body`. Plain values are escaped automatically; keep `{{ body | safe }}` for the rendered description. The override applies to generated operation pages in builds, previews and PDF exports. Specification descriptions display embedded DocAnvil directives literally.

The header is also available in authored guides:

````markdown
:::api-operation{title="List documents" method="GET" path="/documents"}
Return the documents in your workspace.
:::
````

Schema pages use the matching `api-schema` component for their name, type and description, with a separate property panel and compact constraint summaries. Customise their header with:

```sh
docanvil component eject api-schema
```

`theme/components/api-schema.html` receives `attrs.title`, `attrs.deprecated`, the rendered type/reference as `type_html`, and the safely rendered description as `body`. Keep `type_html | safe` and `body | safe` when displaying these rendered fragments. You can also use `:::api-schema{title="Document" type="object"}` in an authored guide; its body supplies the description. Schema-specific CSS hooks are `.api-schema`, `.api-schema-kind`, `.api-schema-definition` and `.api-schema-constraints`.

Generated headers also receive the copy dropdown as `curl`; render it with `{{ curl | safe }}` to position it in a custom header. Older overrides that omit it receive the dropdown immediately after their header.

CSS hooks include `.api-curl` for the copy dropdown, `.api-reference` for generated API pages, `.api-operation` for the header, `.api-context-card` for authentication and servers, `.api-media` for content types, and `.api-response` for each response. Header surfaces use `--api-header-bg` and `--api-header-border`; `--api-accent` controls the edge accent. Ordinary documentation pages retain their existing styles.

## Initial support

Operation pages show methods and paths, descriptions, authentication, servers, parameter types and constraints, request bodies, response headers and schema links. Examples appear separately from schemas. When a body supports multiple media types, content-type tabs let readers switch between them; every section remains readable without JavaScript and in PDF exports.

Schema fields show requiredness, enums, defaults, nullability, common constraints and read/write distinctions. Nested fields and `oneOf`/`anyOf`/`allOf` branches can be expanded; recursive references link to schema pages. Expansion is bounded to keep large or recursive models readable. **View source definition** retains complete definitions for keywords that do not yet have a dedicated presentation.

Internal and relative local-file JSON Pointer references are supported, including recursive schemas. Remote references, named anchors, schema `$id` scopes, dynamic anchors and custom schema dialects fail with source context. Webhooks and callbacks are checked for references but are not rendered as operations; DocAnvil reports warnings, which fail `build --strict` and `doctor --strict`.

### Schema support matrix

This matrix describes documentation output, not validation of API payloads. Constraint values are displayed; DocAnvil does not evaluate schemas, check examples against them, or combine composition branches into a flattened model.

| Schema feature | OpenAPI 3.0 | OpenAPI 3.1 |
| --- | --- | --- |
| `type` and nullability | Single type name; `nullable: true` adds null only when the same schema declares a type | Single type or an array of unique types, including `null`; boolean schemas show “Any value” or “No value allowed” |
| `properties`, `required`, `items`, `additionalProperties` | Property tables, required-name lists, array items and open/closed/typed maps | Same; boolean subschemas are also supported |
| `allOf`, `oneOf`, `anyOf` | Separate expandable branches; no merging or discriminator-based selection | Same |
| `enum`, `default`, `format`, `minimum`, `maximum`, `multipleOf`, `minLength`, `maxLength`, `pattern`, `minItems`, `maxItems`, `uniqueItems`, `minProperties`, `maxProperties` | Values displayed as constraints/annotations | Same; `const` is also displayed |
| `exclusiveMinimum`, `exclusiveMaximum` | Boolean flags displayed alongside `minimum`/`maximum` | Numeric exclusive bounds displayed directly |
| `description`, `readOnly`, `writeOnly`, `deprecated` | Descriptions and annotations; read/write flags do not filter fields from requests or responses | Same |
| Local `$ref` | Linked targets; siblings ignored, including in property tables | Target and sibling constraints retained; recursive expansion is bounded |
| `$defs`, `$schema` | Dialect warning; no dedicated presentation | `$defs` is traversed for local references; supported `$schema` declarations select an accepted dialect, without a payload evaluator |
| `not`, `discriminator`, `xml`, `externalDocs` | Source only, with a warning | Same |
| `prefixItems`, `contains`, `minContains`, `maxContains`, `patternProperties`, `propertyNames`, `dependentRequired`, `dependentSchemas`, `if`, `then`, `else`, `unevaluatedProperties`, `unevaluatedItems`, `contentEncoding`, `contentMediaType`, `contentSchema`, `$vocabulary` | Dialect warning; source only | Source only, with a warning; references in recognized subschemas are still loaded |
| Metadata and examples | `title` and `example` retained in source; a direct media schema's `example` is also shown beneath it | Same; `examples` and `$comment` retained in source |
| Extensions and other keywords | `x-…` payloads retained without interpretation; other unknown keywords produce a dialect warning | `x-…` payloads retained without interpretation; other unknown/legacy keywords produce a source-only warning |

`openapi-schema-unrendered` reports features without a dedicated presentation; `openapi-schema-dialect` reports keywords outside the selected dialect, including `nullable` in 3.1 and `const` in 3.0. Both warnings include a source file and JSON Pointer and fail strict builds and doctor checks. Source-only metadata listed above and `x-…` extensions do not warn. Unsupported URI scopes, anchors and custom dialects remain errors.

Malformed types and the field shapes consumed by the renderer produce errors rather than incomplete tables: for example, `required` must contain unique property names, composition branches must be non-empty arrays, and exclusive bounds must use the selected dialect's value type. These checks are deliberately partial. They do not establish complete OpenAPI/JSON Schema conformance or enforce relationships between constraints.

Descriptions render as Markdown. Raw HTML is omitted, and DocAnvil includes/directives are not executed from specification descriptions. Live requests, SDK generation and full schema validation remain outside this initial integration.

## Generated cURL examples

The operation header has a **Copy as cURL** dropdown at the top right. Choose a request media type to copy its POSIX shell command. Without JavaScript or clipboard access, open the dropdown to select the command manually. They use the first effective server (expanding variable defaults) and the first authentication alternative. Credentials always use placeholders. Relative or absent server URLs use an example origin that you must replace.

Required parameters are included. Optional parameters are included when they have a supplied example or schema default. Values come from the parameter example, the first inline or locally referenced named example, then the resolved schema example/default. Missing values use `REPLACE_…` placeholders. Query/path values are percent-encoded, and all command arguments use shell quoting.

Supported serialization covers flat scalar/array/object values with query/cookie `form`, path/header `simple`, path `label`/`matrix`, query `deepObject` (exploded objects), and non-exploded `spaceDelimited`/`pipeDelimited` arrays. Nested values, parameter `content` and other styles receive replacement values and an explanation. Reserved characters are encoded even when `allowReserved` is set.

Bodies use the media type's example or first available named example. JSON is serialized as JSON; text examples are preserved literally. Flat object examples support URL-encoded forms and multipart text fields without custom `encoding`. Other bodies use `--data-binary '@request-body'`: create that file with the desired payload. Multipart file/custom-encoding requests require prepared parts and a matching boundary. External examples are not fetched, payloads are not synthesized from schemas, and examples are not validated against schemas.

Basic authentication uses username/password placeholders; bearer, OAuth 2 and OpenID Connect use token placeholders. API keys support headers, query strings and cookies. Schemes within the selected authentication alternative are combined. Other schemes require manual setup. DocAnvil never executes these commands. The examples remain visible without JavaScript and are included in PDF and AI outputs.
