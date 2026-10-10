---
{
  "description": "Turn Markdown into fast, searchable documentation sites with a single Rust binary.",
  "toc": false
}
---

::::hero{eyebrow="Markdown-powered documentation" title="Beautiful docs. Built fast."}
Turn Markdown into fast, searchable, beautifully styled documentation sites. One binary, no complicated tooling.

:::buttons
[[guides/getting-started|Get started →]] [View on GitHub ↗](https://github.com/docanvil/docanvil)
:::
::::

## Quick start

```bash
curl -fsSL https://github.com/docanvil/docanvil/releases/latest/download/install.sh | sh
docanvil new my-docs
cd my-docs && docanvil serve
```

Open [http://localhost:3000](http://localhost:3000) and start writing — every save shows up in your browser instantly. On Windows, or prefer Cargo? See [[guides/getting-started|Getting Started]].

## What you get

::::features
:::feature{icon="⚡" title="Fast static builds"}
Plain HTML output you can deploy anywhere, with live reload while you write.
:::
:::feature{icon="🔍" title="Full-text search"}
A client-side search index, built for every page with no setup.
:::
:::feature{icon="🧩" title="Components"}
Notes, tabs, code groups, diagrams — or [[writing/components|your own templates]].
:::
:::feature{icon="🎨" title="Custom themes"}
CSS variables, a [[guides/theming|theme generator]] and full template overrides.
:::
:::feature{icon="🌍" title="Localisation"}
[[guides/localisation|Multi-language docs]] with per-locale navigation and search.
:::
:::feature{icon="🗂️" title="Versioning"}
Keep [[guides/versioning|docs for every release]] side by side, with a version switcher.
:::
::::

## Explore the docs

| Section | What you'll learn |
|---------|-------------------|
| [[guides/getting-started\|Installation]] | Install DocAnvil and create your first project |
| [[guides/configuration\|Configuration]] | Customize `docanvil.toml` and `nav.toml` |
| [[guides/theming\|Theming]] | CSS variables, custom CSS, and template overrides |
| [[guides/localisation\|Localisation]] | Multi-language docs with locale suffixes and language switcher |
| [[writing/markdown\|Markdown]] | All supported Markdown and GFM features |
| [[writing/wiki-links\|Links & Popovers]] | Wiki-link syntax and inline popovers |
| [[writing/components\|Components]] | Notes, tabs, code groups, landing pages and your own templates |
| [[reference/cli\|CLI Commands]] | Every command, flag, and option |
| [[reference/project-structure\|Project Structure]] | Directory layout and page discovery |
| [[reference/css-variables\|CSS Variables]] | Complete variable reference with defaults |
