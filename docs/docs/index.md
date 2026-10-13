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

```bash title="Quickstart"
curl -fsSL https://github.com/docanvil/docanvil/releases/latest/download/install.sh | sh
docanvil new my-docs
cd my-docs && docanvil serve
```

Open [http://localhost:3000](http://localhost:3000) and start writing — every save shows up in your browser instantly. On Windows, or prefer Cargo? See [[guides/getting-started|Getting Started]].

## What you get

::::features
:::feature{icon="lucide:zap" title="Fast static builds"}
Plain HTML output you can deploy anywhere, with live reload while you write.
:::
:::feature{icon="lucide:search" title="Full-text search"}
A client-side search index, built for every page with no setup.
:::
:::feature{icon="lucide:blocks" title="Components"}
Notes, tabs, code groups, diagrams — or [[writing/components|your own templates]].
:::
:::feature{icon="lucide:palette" title="Custom themes"}
CSS variables, a [[guides/theming|theme generator]] and full template overrides.
:::
:::feature{icon="lucide:globe" title="Localisation"}
[[guides/localisation|Multi-language docs]] with per-locale navigation and search.
:::
:::feature{icon="lucide:layers" title="Versioning"}
Keep [[guides/versioning|docs for every release]] side by side, with a version switcher.
:::
::::

## Explore the docs

::::cards
:::link-card{title="Getting started" href="guides/getting-started.html" icon="lucide:rocket"}
Install DocAnvil and build your first documentation site.
:::
:::link-card{title="Configuration" href="guides/configuration.html" icon="lucide:settings"}
Set up your project, navigation, and build options.
:::
:::link-card{title="Theming" href="guides/theming.html" icon="lucide:palette"}
Customise colours and templates, with a complete [[reference/css-variables|CSS variable reference]].
:::
:::link-card{title="Localisation" href="guides/localisation.html" icon="lucide:globe"}
Publish in multiple languages with navigation and search for each locale.
:::
:::link-card{title="Writing content" href="writing/components.html" icon="lucide:blocks"}
Add components to your [[writing/markdown|Markdown]] and connect pages with [[writing/links-popovers|wiki-links and popovers]].
:::
:::link-card{title="CLI reference" href="reference/cli-commands.html" icon="lucide:terminal"}
Find every command and option, plus the [[reference/project-structure|project directory layout]].
:::
::::
