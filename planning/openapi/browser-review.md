# Static API browser and print review

First pass: 19 October 2026, based on merged schema-matrix commit `3b8c718` plus the local fixes below. This records a real Chrome review of the fictional Documents API, not acceptance of all browsers or larger specifications.

## Environment and coverage

- Headless Google Chrome `155.0.8059.80`, driven by Playwright Core with a separate temporary profile. No changes to the user's browser profile.
- Local `cargo run -- serve --path docs --port 3000` preview; all nine English API pages and the French create-operation page loaded successfully.
- Desktop 1440 × 1000; narrow viewports at 320, 390 and 768 pixels. These are emulated viewport checks, not physical-device tests.
- Keyboard interaction, light/dark screenshots, JavaScript-disabled content, browser printing and the actual `export pdf` workflow.
- Axe scan scoped to the create-operation page's `.content`, in both themes. This is automated coverage, not a screen-reader audit or accessibility certification.

## Results

| Area | Result |
| --- | --- |
| Overview, tag navigation and links | Overview operation links open their operations; the active operation is visible in its sidebar tag group; schema links open reusable definitions. All demo routes return HTTP 200. |
| Media controls | Initial JSON selection, Right/Left wrapping, Home/End selection and focus movement pass. |
| Fragment links | A direct `#application-yaml` link selects YAML and scrolls beneath the fixed header once smooth scrolling settles. A separate hash-change check also passes. |
| Disclosures | Enter opens nested-schema/source disclosures; Space closes a schema disclosure. Recursive schema content remains bounded and links to its definition. |
| Narrow screens | No document-wide horizontal overflow at the tested widths. Wide field tables scroll within the content area. Actual touch usability remains a manual check. |
| No JavaScript | Both media panels remain visible and tab controls stay hidden. Native disclosures remain available. |
| Browser printing | Both media types and closed schema/source disclosures are included. After the print fix, fixed navigation and Copy/editor controls are absent. The create-operation PDF is nine A4 pages, including raw source definitions. |
| PDF export | Master English export succeeds. Text extraction confirms YAML examples, nested/recursive schema text and source disclosures are present. Selected pages were rendered to images for visual inspection. |
| Runtime | No page errors or failed HTTP responses in the automated demo traversal. |
| Accessibility | The skipped request-media heading level is fixed. Dark content has no Axe violations in this scan. Light content still reports the existing custom-theme link contrast finding below. |

The final browser harness reports 33 passing checks out of 34; the remaining check is light-theme contrast. The Rust suite passes all 948 tests, Clippy passes with warnings denied, strict docs build produces 296 pages and strict doctor reports no issues. Browser findings are separate from doctor diagnostics.

## Fixes from this review

1. **Browser print navigation overlaps content.** The fixed top header repeated on printed pages, covering content near page boundaries; interactive Copy controls also printed. Print CSS now removes web navigation and editor/copy controls, clears web layout offsets and allows code to wrap. It applies to normal documentation pages as well as API pages. The dedicated PDF template continues to provide its own page furniture.
2. **Request media skips a heading level.** Request panels used H4 beneath the H2 Request body section, while single response media headings were peers of the response status. Media headings now use H3 beneath request bodies and H4 beneath parameter-content/response-status sections, consistently in HTML and export Markdown. A regression covers single and multiple response media types.

## Acknowledged finding and remaining review

- **Light-theme link contrast — acknowledged, colours retained:** the docs project's custom `--color-link: #f17a06` measures approximately 2.79:1 against white and 2.54:1 in the version banner. Axe flags schema links and other existing site links. After visual review, the maintainer chose to retain the current theme colours. This remains an acknowledged automated accessibility finding, with no colour change planned in this review. Dark-mode content passes the same scan.
- **Other browsers and assistive technology:** verify Safari/iOS, Firefox, physical touch interaction, zoom and a screen reader. Chrome's handling of closed `<details>` in print does not establish Safari/Firefox behavior.
- **Large specifications:** route usability, dense navigation, long descriptions, unusual schema combinations and performance still need a representative real-world input.
- **Export size:** complete source disclosures account for much of the print/PDF length. Review whether readers want an optional compact export later; no source content was omitted in this pass.

## Suggested hands-on pass

Start the preview and open `/master/en/example-api/operations/createdocument.html`.

1. On desktop, switch JSON/YAML, try the arrow keys and Home/End, and open **View source definition**. Follow a schema link, then return through navigation.
2. On a phone or narrow window, check that the navigation menu, media tabs and sideways field-table scrolling feel usable. Check the same page with browser zoom increased.
3. Open `/master/en/example-api/schemas/document.html` and expand **Explore related**. The recursive schema should remain readable and offer a link to Document.
4. Print the create-operation page with disclosures closed. Confirm both media types and disclosure content are present and that navigation does not cover text, especially on page two.

The maintainer's visual check found the demo acceptable. Further manual checks can focus on Safari/iOS, zoom and physical touch interaction, followed by the larger-spec run.

## Local evidence

Temporary artifacts live under `/tmp/docanvil-browser-review/`: `results.json`, `accessibility.json`, the Playwright review scripts, desktop/mobile/dark/no-JavaScript screenshots, `create-document-print.pdf`, `docanvil-master-en.pdf` and rendered print/PDF page images. They are not committed and may be removed by the operating system. The checks above are the durable review record.

Recreate the full export with:

```sh
cargo run -- export pdf --path docs --locale en --doc-version master --out /tmp/docanvil-master-en.pdf
```
