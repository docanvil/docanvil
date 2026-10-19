# Larger local specification review

Reviewed on 19 October 2026 using an external local OpenAPI 3.0.0 specification: 762,035 bytes, 211 paths, 257 operations, 262 reusable schemas, 75 declared tags and 370 internal references. No specification, generated reference pages, screenshots, endpoint names or schema definitions were copied into this repository. Detailed evidence remains under `/tmp/docanvil-large-api-review/`.

## Original versus adjusted input

The original was first referenced directly through a temporary project's relative `openapi.spec` path. Build and doctor rejected a class name used as a schema `type`. A schema-aware audit found seven incompatible type declarations: four `null` types in a 3.0 document, two `float` types and one unresolved class-name type. A subsequent build exposed one Response Object with no required description. All internal reference targets resolve; operation IDs are present and unique.

With the maintainer's approval, an adjusted copy was created under `/tmp`, with a local change log:

- Four null-only branches use a nullable string constrained by `enum: [null]` for this review.
- Two float declarations use `type: number` and `format: float`.
- The unresolved class-name type becomes a generic object, explicitly described as a review placeholder. Its missing structure was not inferred.
- The missing response description receives an explicit review placeholder.
- An overview notice identifies the input as an adjusted test copy.

The original's SHA-256 fingerprint was checked again after the review and is unchanged. Results below apply to the adjusted copy; they do not establish that the unmodified original builds or that the generic object describes its intended payload.

## Build and size observations

These are local debug-binary measurements, not release-build or hosted/mobile benchmarks.

| Measure | Observed result |
| --- | --- |
| Strict build | Passes; approximately 9–11 seconds across measured runs |
| Peak resident memory during build | Below 100 MiB in the measured runs |
| Strict doctor | Passes; approximately 1.7 seconds in the first measured run |
| Content pages | 521: overview, 257 operations, 262 schemas and one authored review page; also a generated 404 page |
| Output | Approximately 95 MiB across 558 files, including HTML, assets, search and AI outputs |
| Largest HTML page | Overview, approximately 3 MiB, including the complete source disclosure |
| Search | Approximately 2 MiB and 2,564 indexed sections |
| AI full output | Approximately 1.7 MiB |
| Route stability | The generated HTML path inventory is unchanged across the wrapping fix |

The temporary project sets a localhost `site_url` to avoid a fixture-configuration warning. It has no version or locale duplication, so these sizes describe one scope.

## Browser and print checks

Headless Chrome `155.0.8059.80` exercised the overview, three largest operation pages, a small operation and two largest schema pages. The final pass reports **18/18 checks passing**, no page errors and no failed local HTTP responses.

- Desktop page navigation measured approximately 31–292 ms on localhost in the final run. The overview contains approximately 64,000 DOM elements; a collapsed source disclosure still contributes to HTML parsing cost.
- Initial search returned eight results in approximately 248 ms, including local loading/index construction. Keyboard selection opened the selected result.
- A large operation's source disclosure opens by keyboard. The sampled operation has no multi-media tab group; the separate demo review covers those controls.
- The overview, a large operation and a large schema stay within the page width at 390 and 768 pixels after the wrapping fix.
- The sample operation renders without JavaScript. Browser printing hides fixed navigation and Copy controls and includes the source disclosure; the sample PDF is 13 A4 pages.

All non-local browser requests were blocked. The public MiniSearch library was downloaded separately and fulfilled from a local file during search testing. Private specification content was not sent to a CDN or external API.

## Issue found and fixed locally

Long schema names and schema-list links caused document-wide horizontal overflow at 390 pixels, unlike the small demo. A content-level `overflow-wrap: break-word` rule fixes both affected pages. Field tables retain their existing horizontal scrolling behavior. The fix contains no source-specific names or fixtures.

## Follow-up

The measured build time does not establish a need for caching or incremental rendering. The large overview source disclosure and repeated navigation are concrete places to investigate if output size or slower-device measurements warrant optimization.

Remaining coverage includes physical devices, other browser engines, realistic network/CPU constraints and a full native PDF export of all 521 content pages. Only a representative operation was printed in this larger-spec pass. The original generator's incompatible types and missing description also need correction before it can be built unchanged.

Local artifacts include `inputs/schema.adjusted.json`, `adjustments.json`, build/doctor logs, `browser-results.json`, route inventories, screenshots and `sample-operation.pdf`. They are temporary, untracked and may be removed by the operating system. The temporary static preview is available at `http://127.0.0.1:3001/api/index.html` while its local server is running.
