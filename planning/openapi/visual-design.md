# API reference visual design review

Scope: API pages first. Preserve configurable theme colours and the appearance of ordinary documentation pages. Reviewed against the fictional Documents API and the temporary adjusted large specification; private input and derived artifacts remain outside the repository.

## Changes

- New `api-operation` component automatically renders generated operation headers: HTTP method, title, endpoint when useful, description, tags and deprecation notice. Projects can eject/override it through `theme/components/api-operation.html`, or use it in authored guides.
- Native rendering uses the selected project's component registry, including historical version roots. Specification descriptions keep safe Markdown handling: raw HTML and executable authoring directives are not enabled.
- The header accent uses its own rounded border, so desktop and mobile corners share the same radius.
- Matching `api-schema` headers show the schema name, type/reference and description, with a separate property panel and compact constraint summaries. The header can be ejected and overridden independently. Alias siblings retain their OpenAPI 3.0/3.1 semantics; arrays, compositions, recursive references and Boolean schemas keep their details.
- Compact authentication/server cards, response panels with status badges, consistent single/multiple content-type surfaces, quieter tables and clearer section spacing.
- Overview counts and schema headers share the reference styling. Light/dark surfaces follow theme tokens; print uses restrained borders and exposes all content types. Markdown exports retain their readable representation independently of component markup.
- English and French OpenAPI guides document template context, CSS hooks and component ejection.

## Verification

- 954 Rust tests pass, including template overrides, escaped titles, literal specification directives, authored component use, schema header overrides, alias/array/composition/Boolean schema presentation and existing version/locale/fragment checks.
- Clippy passes with warnings denied; formatting and diff checks pass. Strict documentation build produces 296 pages.
- Large temporary input rebuilds 521 pages. Its original fingerprint is unchanged.
- 33 headless Chrome checks pass: operation structure, keyboard content-type switching, header accessibility in configured light/dark themes, mobile layout, no-JavaScript content/source disclosure and print content visibility. Overview, large/small operations and a large schema fit 390px, 768px and 1440px viewports.
- An additional 34 schema browser checks pass: four public schemas, header accessibility in configured light/dark themes, keyboard/no-JavaScript recursive disclosures, print visibility, operation/guide scope and two large schemas at 390px, 768px and 1440px. Narrow property tables scroll horizontally while keeping description columns readable.
- Native PDF export of the 10-page public preview completes (51 printed pages); schema headers and property output were visually inspected.
- Native Master/English PDF export completes (224 pages). The API operation chapter was visually inspected. This is a sample review, not a visual audit of every exported page.
- Broader axe scans still report existing link-colour and syntax-highlighting contrast findings. The new header passes its own axe checks after adjusting description contrast; shared palettes are unchanged.

Temporary previews and evidence: `/tmp/docanvil-api-design/` and `/tmp/docanvil-large-api-review/`. Local public preview runs on port 3003; the large review remains on port 3001. Scripts and screenshots are temporary, with no private source copied into tracked fixtures or docs.

## Follow-up

Maintainer visual feedback on the direction, then Safari/iOS and physical-device checks. The original browser review's shared palette decisions remain in effect. This pass does not add live API requests.
