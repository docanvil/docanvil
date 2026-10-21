# 🛣️ DocAnvil Public Roadmap

This roadmap outlines where **DocAnvil**, a Markdown-first static documentation generator, has been and where it's headed.

DocAnvil is **stable (1.x)** and used in production. New features are generally additive and opt-in. Review release migration notes for custom templates; 1.3 upgrades the template engine to Tera 2.

> ⚠️ This roadmap is directional, not a promise. Priorities may shift based on feedback and real-world usage.

---

## 📌 Versioning

DocAnvil follows **Semantic Versioning (SemVer)**:

- **Patch releases (1.x.y)** fix bugs and polish existing features
- **Minor releases (1.x)** add features without breaking existing sites
- **A major release (2.0)** only happens if there's a clear need for a breaking change

---

## ✅ Shipped

### 1.0 — Trustworthy core

- Markdown → static HTML with live-reloading dev server (`docanvil serve`)
- Components (`:::note`, `:::tabs`, `:::mermaid` and more), wiki-links, popovers, syntax highlighting
- Client-side search
- Broken link detection with `--strict` mode for CI
- SEO outputs: `sitemap.xml`, `robots.txt`, page-level meta tags from front matter
- Nested, collapsible sidebar navigation via `nav.toml`, with autodiscover
- `docanvil new`, `docanvil theme` and `docanvil doctor`

### 1.1 — Docs at scale

- 🌍 **Localisation**: multi-language builds, per-locale nav and search, language switcher, hreflang SEO
- 🗂️ **Versioning**: versioned builds with a version switcher and older-version banner
- 📄 **PDF export**: `docanvil export pdf` with cover pages, RTL support, per-locale output and custom paper sizes
- 🧹 **Content linting**: readability, heading structure, alt text and link checks in `docanvil doctor`, with Checkstyle and JUnit output for CI
- 📦 **Easy installs**: install scripts for macOS, Linux and Windows, prebuilt binaries for more platforms, and `docanvil update`

### 1.2 — Authoring essentials

- ✏️ **Edit links**: "Edit this page" links to a page's source on GitHub, GitLab or Bitbucket, and "Open in editor" links straight to your local editor from `docanvil serve`
- ♻️ **Includes and code from files**: reuse shared content with `:::include`, pull code blocks from real source files (optionally just some line ranges), and add line numbers to any code block
- 🧩 **Template components**: define your own `:::components` as Tera templates, restyle the built-ins, nest components, and `docanvil component list` / `eject`
- 📝 **Drafts and redirects**: preview draft pages in `docanvil serve` while keeping them out of production builds, and keep old URLs working when pages move
- 🕒 **Last updated dates** from Git history or front matter, on the page and in `sitemap.xml`
- 🤖 **`llms.txt`** and `llms-full.txt`, an AI-friendly index and full-text copy of your docs
- 🧭 **Page headers**: breadcrumbs, front matter descriptions as subtitles, and page titles from the first heading
- **Git-backed versions**: publish release-tag snapshots alongside the working tree, with stable `/latest/` aliases
- **Layout components and icons**: landing pages, cards, steps, details, figures and columns, with bundled Lucide and code-language icons
- **Section includes**: reuse a Markdown heading and its subsections with `section="heading-id"`
- **Code presentation**: highlighted lines, language-aware diffs and theme variables for code surfaces
- **Remote code blocks**: load raw HTTPS source with `url="…"`; remote Markdown includes remain an unscheduled idea
- **Named code regions**: select stable source examples with `{== region="name" ==}` / `{== endregion="name" ==}` markers in comments

---

## 🎯 1.3 — API documentation

Static OpenAPI reference generation is implemented and merged, but **not yet released**. The next step is final release validation.

- Local OpenAPI 3.0/3.1 JSON/YAML specifications and local references, with a [documented support matrix](docs/docs/guides/openapi.md#schema-support-matrix)
- Operation and linked schema pages with parameters, request bodies, responses, authentication, constraints and examples
- Stable URLs, specification-ordered tag navigation, HTTP method badges, search and generated-link/anchor validation
- Related guides beside operations, scoped to the same documentation version and locale
- Version-matched specification downloads: original single-file bytes or portable multi-file ZIPs
- Static cURL examples with server, authentication, content-type and example choices, a command preview and credential placeholders; no request execution
- Theme, Git-backed version, locale, PDF, sitemap and AI-output integration, with strict build/doctor diagnostics

Chrome demo and adjusted larger-spec reviews are recorded. Remaining acceptance work covers visual review of the final controls, Safari/Firefox, zoom/touch, slower-device performance, full larger-spec PDF export and final candidate checks.

Interactive requests, remote specifications and additional API formats remain deferred. OpenAPI generation is opt-in. Custom templates must follow the [Tera 2 migration guidance](docs/docs/guides/theming.md#migrating-custom-templates-to-tera-2).

The [V1.3 plan](planning/v1.3-priorities.md) tracks completed scope, evidence and remaining release checks.

---

## 🔍 1.4+ — Build reliability and documentation quality

Prioritise these after the first API reference workflow, adjusting order based on usage:

- Reproducible remote code sources: explicit refresh, persistent cache, integrity checks and offline builds
- Broader internal link and heading-anchor validation beyond the generated API targets covered in 1.3
- Front-matter schema validation for teams with required metadata
- Dependency tracking and measured build improvements for larger versioned sites; incremental builds only with correct invalidation
- API reference refinements informed by real specifications and reader feedback

---

## 🧩 Extensibility — Deferred

Template components, nesting, template overrides and bundled layout components already cover many presentation customisations. Defer a WASM plugin system, hooks and SDK until concrete workflows demonstrate what those tools and first-party OpenAPI support cannot cover. No fixed release is assigned; define lifecycle, compatibility and diagnostics before committing to a plugin API.

---

## 💡 Ideas Under Consideration

Not scheduled yet, but on our radar:

- **Remote Markdown includes**, with explicit reproducibility and relative-link rules
- **Glossary / reference index**, generated from front matter or a dedicated glossary file
- **API change reports**, comparing release specifications and highlighting added, removed and changed operations; compatibility judgments need explicit rules
- **API documentation coverage**, flagging missing descriptions, examples and error responses separately from specification validity
- **Translation freshness**, detecting when source pages or shared fragments change after a translation was reviewed
- **Offline documentation bundles**, searchable, self-contained HTML artifacts alongside PDF
- **Verified code examples**, opt-in commands to compile or test selected samples in CI with explicit execution settings
- **Documentation lifecycle metadata**, owners, review dates and deprecated-page notices, with optional stale-content checks
- **URL migration audits**, comparing published versions to flag removed routes and suggest redirects
- **Search refinements**, aliases, weighting and filters that help readers distinguish guides, API operations and schemas
- **Build provenance reports**, recording source revisions, specification versions and remote-source checksums for traceable releases
- **Portable diagrams**, rendering Mermaid at build time for offline HTML and consistent export output
- **Reusable content variables**, for product names, release numbers and shared values, with explicit version/locale scope and missing-value diagnostics
- **Scoped lint suppressions**, with reasons and unused-suppression checks, so intentional exceptions do not require disabling a rule project-wide
- **Page discovery checks**, flagging published pages absent from navigation and offering opt-in related links or backlinks; intentionally unlisted pages remain supported
- **Selective exports**, producing a PDF or offline bundle from selected chapters or navigation groups
- **Documentation change summaries**, showing added, changed and removed pages between releases with links to migration guidance
- **Build presets** for documentation, handbooks and specifications
- **Contrast checks** alongside the existing accessibility lints
- **Math support** (KaTeX), opt-in like Mermaid charts
- **External link checking** in `docanvil doctor`
- **Social cards** (`og:image`) for shared links

---

## 🌅 2.0 and Beyond

A 2.0 release would only happen if there's a clear need: a configuration redesign, plugin API v2, fundamental output model changes, or major architectural shifts. It should be deliberate and well-justified, with migration guidance.

---

## 🧭 Guiding Principles

- Markdown-first, static output
- Minimal core, powerful extensions
- No built-in CMS or WYSIWYG editor
- Optimized for developers and technical writers
- Predictable builds suitable for CI/CD

---

## 💬 Feedback & Contributions

Feature ideas and feedback are welcome via:
- GitHub Issues
- GitHub Discussions
- Pull Requests

If a feature fits the guiding principles, we're happy to explore it.
