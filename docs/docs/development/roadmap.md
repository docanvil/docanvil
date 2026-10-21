---
{
  "description": "Where DocAnvil is today and what's coming in upcoming releases."
}
---

# Roadmap

DocAnvil follows [Semantic Versioning](https://semver.org) and is stable at 1.x, with new features generally additive and opt-in; review migration notes when upgrading custom templates. Here's where things stand and where they're headed.

## 1.1.x — Core Feature Enhancements

- ✅ Localisation support (multi-language docs, per-locale nav and search, language switcher)
- ✅ PDF export (`docanvil export pdf` — Chrome-based, cover pages, RTL, per-locale, custom paper size)
- ✅ Doc versioning support
- ✅ Style linting (`docanvil doctor` — readability checks, heading structure, link quality, and more)
- ✅ Install scripts for macOS, Linux and Windows, more prebuilt platforms, and `docanvil update`

## 1.2.x — Authoring Essentials

The everyday features people expect from a docs tool, so new projects have everything they need from day one:

- ✅ "Edit this page" links to a page's source on GitHub, GitLab, Bitbucket or any Git host
- ✅ Open the source file in your editor straight from `docanvil serve`
- ✅ Redirects, so old URLs keep working when pages move
- ✅ Includes, to reuse shared content across pages, locales and versions
- ✅ Code blocks from files, optionally just some line ranges, plus line numbers for any code block
- ✅ Draft pages that show in `docanvil serve` but stay out of production builds
- ✅ Last updated dates from Git history or front matter
- ✅ `llms.txt` output, an AI-friendly index of your docs
- ✅ Template components: define your own components as Tera templates in your theme, no Rust needed
- ✅ Breadcrumbs, front matter descriptions as page subtitles, and page titles from the first heading
- ✅ Highlight selected code lines with `highlight="2,5-8"`, including file and remote source blocks
- ✅ Language-aware diff code blocks with syntax highlighting, red/green line backgrounds and a diff badge, including local `.diff` / `.patch` files and remote sources
- ✅ Theme overrides for code-block surfaces, typography, diff backgrounds and badges
- ✅ Remote code blocks from a raw HTTPS URL (`url="…"`)
- ✅ Named code regions (`{== region="name" ==}` / `{== endregion="name" ==}`), so examples don't break when line numbers shift
- ✅ Landing-page and layout components, bundled icons and expandable navigation groups
- ✅ Git-backed release snapshots, a working-tree version and stable `/latest/` aliases
- ✅ Version- and locale-aware PDF selection, navigation and styling
- ✅ Include a Markdown heading and its section with `section="heading-id"`

Fully backward compatible, and every feature is opt-in.

## 1.3.x — API Documentation

Static OpenAPI reference generation is implemented and merged, but **not yet released**. The next step is final release validation.

- Local OpenAPI 3.0/3.1 JSON/YAML specifications and local references, with a [documented support matrix](../guides/openapi.md#schema-support-matrix)
- Operation and linked schema pages with parameters, request bodies, responses, authentication, constraints and examples
- Stable URLs, specification-ordered tag navigation, HTTP method badges, search and generated-link/anchor validation
- Related guides beside operations, scoped to the same documentation version and locale
- Version-matched specification downloads: original single-file bytes or portable multi-file ZIPs
- Static cURL examples with server, authentication, content-type and example choices, a command preview and credential placeholders; no request execution
- Theme, Git-backed version, locale, PDF, sitemap and AI-output integration, with strict build/doctor diagnostics

Chrome demo and adjusted larger-spec reviews are recorded. Remaining acceptance work covers visual review of the final controls, Safari/Firefox, zoom/touch, slower-device performance, full larger-spec PDF export and final candidate checks.

Interactive requests, remote specifications and additional API formats remain deferred. OpenAPI generation is opt-in. Custom templates must follow the [Tera 2 migration guidance](../guides/theming.md#migrating-custom-templates-to-tera-2).

## 1.4.x+ — Reliability and Documentation Quality

Reproducible remote code sources with explicit refresh, persistent caching and offline builds; broader internal link and anchor validation; optional front-matter schemas; and measured build improvements for larger versioned sites. API reference refinements will follow real-world feedback. This order is directional, not a release promise.

## Extensibility — Deferred

Template components and overrides already cover many presentation needs. A WASM plugin system, hooks and SDK are deferred until concrete workflows demonstrate what templates and first-party generators cannot cover. No fixed release is assigned.

## Ideas Under Consideration

These ideas are unscheduled; they do not expand the initial OpenAPI release:

- API change reports between release specifications, with explicit compatibility rules
- API documentation coverage checks for missing descriptions, examples and error responses
- Translation freshness checks when source pages or shared fragments change
- Searchable offline HTML bundles alongside PDF
- Opt-in compilation or testing of code examples in CI, with explicit execution settings
- Documentation owners, review dates, deprecated-page notices and stale-content checks
- URL migration audits to flag removed routes and suggest redirects
- Search aliases, weighting and filters for guides, operations and schemas
- Build provenance reports with source revisions and remote-source checksums
- Build-time Mermaid rendering for portable diagrams and consistent exports
- Reusable content variables with version/locale scope and missing-value diagnostics
- Scoped lint suppressions with reasons and unused-suppression checks
- Checks for pages absent from navigation, plus opt-in related links or backlinks
- Selective PDF or offline exports by chapter or navigation group
- Documentation change summaries between releases, linked to migration guidance
- Glossary, remote Markdown includes, build presets, contrast checks, math support (KaTeX), external link checking and social cards

Remote code blocks are already supported; remote Markdown includes are a separate idea.

## 2.0 and Beyond

A 2.0 release would only happen if there's a clear need — configuration redesign, plugin API v2, fundamental output model changes, or major architectural shifts. It should be deliberate and well-justified.
