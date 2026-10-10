---
{
  "description": "Where DocAnvil is today and what's coming in upcoming releases."
}
---

# Roadmap

DocAnvil follows [Semantic Versioning](https://semver.org) and is stable at 1.x, so upgrades within 1.x won't break your site. Here's where things stand and where they're headed.

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

Fully backward compatible, and every feature is opt-in.

Still to come in 1.2, now that the first version of includes is in people's hands:

- Include one section of a page (such as your README's Installation section) instead of the whole file
- Named code regions (`#region` / `#endregion`), so examples don't break when line numbers shift
- Line highlighting and diff-style markers in code blocks
- Remote includes from a URL

## 1.3.x — Extensibility

A WASM plugin system (v1) for Markdown transformers and anything else template components can't do, additional CLI flags, more diagnostics, and template enhancements. Fully backward compatible.

## 1.4.x+ — Ecosystem Growth

Plugin hooks, performance optimizations, incremental builds, caching, and a plugin SDK crate. Breaking changes require a major release.

## Ideas Under Consideration

Not scheduled yet, but on our radar: a glossary, front-matter schema validation, OpenAPI reference generation, offline documentation bundles, build presets, contrast checks, math support (KaTeX), external link checking, and social cards.

## 2.0 and Beyond

A 2.0 release would only happen if there's a clear need — configuration redesign, plugin API v2, fundamental output model changes, or major architectural shifts. It should be deliberate and well-justified.
