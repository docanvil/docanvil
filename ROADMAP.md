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

## 🧩 1.2 — Extensibility

- **Plugin system (v1)**, WASM-based
  - Markdown transformers
  - Custom components
- Additional CLI flags, more diagnostics, and template enhancements

Fully backward compatible.

---

## 🚀 1.3+ — Ecosystem Growth

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
