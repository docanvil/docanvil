# 🛣️ DocAnvil Public Roadmap

This roadmap outlines where **DocAnvil**, a Markdown-first static documentation generator, has been and where it's headed.

DocAnvil is **stable (1.x)** and used in production. Upgrades within 1.x are safe: new functionality is additive and opt-in, and existing sites keep building without changes.

> ⚠️ This roadmap is directional, not a promise. Priorities may shift based on feedback and real-world usage.

---

## 📌 Versioning

DocAnvil follows **Semantic Versioning (SemVer)**:

- **Patch releases (1.1.x)** fix bugs and polish existing features
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

---

## ✍️ 1.2 — Authoring Essentials

The everyday features people expect from a docs tool, so new projects have everything they need from day one.

- **"Edit this page" links**: point readers at a page's Markdown source on GitHub, GitLab, Bitbucket or any Git host, so they can suggest fixes as pull requests. Locale- and version-aware, with a per-page opt-out
- **Open in your editor from `docanvil serve`**: during local development, the edit link opens the source file straight in your editor instead
- **Redirects**: keep old URLs working when pages are renamed or moved
- **Includes**: reuse shared content (install steps, warnings, snippets) across pages, locales and versions
- **Code blocks from files**: pull a code block's contents from a real source file, optionally just some line ranges, so examples stay in sync with code that compiles and is tested. Line numbers for any code block, with hidden lines between ranges clearly marked
- **Draft pages**: mark a page as a draft to preview it with `docanvil serve` while keeping it out of production builds
- **Last updated dates**: show when each page last changed, from Git history or front matter
- **`llms.txt` output**: an AI-friendly index of your docs, generated alongside `sitemap.xml`
- **Template components**: define your own `:::components` as Tera templates in `theme/components/`, with no Rust or build tooling needed. A template with a built-in's name (like `note.html`) restyles that component

Fully backward compatible, and every feature is opt-in.

### Later in 1.2

Follow-ups to includes and code blocks from files, once the first version is in people's hands:

- **Include a section of a page**: pull in one heading's section of a Markdown file (say, the Installation section of your README) instead of the whole file
- **Named code regions**: mark a region in a source file with `#region` / `#endregion` comments and include it by name, so examples don't break when line numbers shift
- **Line highlighting**: draw attention to specific lines in a code block, plus diff-style added/removed markers
- **Remote includes**: include content or code from a URL

---

## 🧩 1.3 — Extensibility

- **Plugin system (v1)**, WASM-based, for what template components can't do
  - Markdown transformers
  - Custom components
- Additional CLI flags, more diagnostics, and template enhancements

Fully backward compatible.

---

## 🚀 1.4+ — Ecosystem Growth

- Plugin hooks and a plugin SDK crate
- Incremental builds and caching for faster rebuilds on large sites
- Performance optimisations

---

## 💡 Ideas Under Consideration

Not scheduled yet, but on our radar:

- **Glossary / reference index**, generated from front matter or a dedicated glossary file
- **Front-matter schema validation**, to catch invalid or missing metadata early
- **API reference generation**, OpenAPI-first
- **Offline documentation bundles**, as self-contained artifacts
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
