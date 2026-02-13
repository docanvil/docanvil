# 🛣️ DocAnvil Public Roadmap

This roadmap outlines the planned evolution of **DocAnvil**, a Markdown-first static documentation generator.

DocAnvil is currently in **early development (v0.x)**. The focus is on stabilizing the core experience before committing to long-term compatibility guarantees in **v1.0.0**.

> ⚠️ This roadmap is directional, not a promise. Priorities may shift based on feedback and real-world usage.



## 📌 Versioning Overview

DocAnvil follows **Semantic Versioning (SemVer)** with the following intent:

### **0.x — Learning & Shaping**
Rapid iteration while the core model, CLI, and configuration are refined.  
Breaking changes may occur as the tool matures.

### **1.x — Trustworthy Core**
A stable, production-ready documentation generator.  
Upgrades within 1.x should be safe and predictable.

### **2.x — Scale & Expansion**
Support for large documentation ecosystems, including versions, languages, and APIs.



## 🚧 0.x — Learning & Shaping (Current → Pre-1.0)

**Summary:**  
> Prove the core model, harden the CLI, and ensure DocAnvil can fully document itself with confidence.

### Planned Focus Areas
- Continued iteration on the core static site generator
- Improving defaults and error handling
- Using DocAnvil to fully document DocAnvil itself

### Key Features Targeted Before 1.0
- **Broken link detection at build time**
  - Optional `--strict` mode for CI
- **Automatic SEO outputs**
  - `sitemap.xml`
  - `robots.txt`
  - Page-level meta tags from front-matter
- **Improved sidebar navigation**
  - Nested & collapsible sections
  - Better handling of large documentation trees
- **Glossary / reference index**
  - Auto-generated from front-matter or a dedicated glossary file
- **CLI quality-of-life improvements**
  - `docanvil doctor` — detect common configuration and content issues
  - `docanvil new` — scaffold common documentation templates

### Exit Criteria for 1.0.0
- CLI commands and flags are stable
- Configuration format is stable and documented
- Content model (Markdown, components, front-matter) is stable
- DocAnvil’s own documentation is generated using DocAnvil
- Reasonable confidence that upgrades within 1.x will not break sites



## 🎉 1.0.0 — Trustworthy Core

**Summary:**  
> A stable, predictable documentation generator suitable for production use.

### What 1.0.0 Represents
- Stable CLI and configuration
- Reliable builds suitable for CI/CD
- Clear documentation and upgrade expectations
- No required extensions or plugins



## 🧩 1.x — Extensibility & Refinement

**Summary:**  
> Grow DocAnvil’s capabilities without breaking existing sites.

### Planned Enhancements (Minor Releases)
- **Plugin system**
  - Markdown transformers
  - Custom components
  - Build hooks
- **Front-matter schema validation**
  - Early detection of invalid or missing metadata
- **Component registry pattern**
  - Official components
  - Community-contributed extensions
- **Optional content linting**
  - Style checks
  - Spelling checks
  - Consistency rules

### Guiding Rule for 1.x
- All new functionality must be **additive and opt-in**
- Existing documentation sites should continue to build without changes



## 🌍 2.0.0 — Docs at Scale

**Summary:**  
> Enable large, long-lived documentation ecosystems.

### Planned Major Features
- **Multi-version documentation**
  - Versioned builds (e.g. v1, v2, latest)
  - Built-in version switcher UI
- **Internationalization (i18n)**
  - Multi-language builds
  - Language switcher
- **API reference generation**
  - OpenAPI-first support
  - Consistent reference layouts
- **Search improvements**
  - Version- and language-aware indexing and filtering

### Why This Is a Major Release
- URL structures may change
- Configuration and build behavior may evolve
- Migration guidance will be provided



## 📦 2.x+ — Distribution & Compliance

**Summary:**  
> Support more output formats and stricter environments.

### Planned Enhancements
- **PDF export**
  - Full site or section-based output
- **Offline documentation bundles**
  - Self-contained artifacts
- **Build presets**
  - Documentation
  - Handbooks
  - Specifications
- **Accessibility checks during build**
  - Heading structure
  - Alt text
  - Basic contrast warnings



## 🧭 Guiding Principles

- Markdown-first, static output
- Minimal core, powerful extensions
- No built-in CMS or WYSIWYG editor
- Optimized for developers and technical writers
- Predictable builds suitable for CI/CD



## 💬 Feedback & Contributions

Feature ideas and feedback are welcome via:
- GitHub Issues
- GitHub Discussions
- Pull Requests

If a feature fits the guiding principles, we’re happy to explore it.