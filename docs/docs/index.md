# DocAnvil

A static documentation generator that turns Markdown into beautiful HTML sites.

## Quick Start

```bash
# Install
cargo install --path .

# Create a new docs project
docanvil new my-docs

# Start the dev server with live reload
cd my-docs
docanvil serve
```

Open [http://localhost:3000](http://localhost:3000) in your browser and start writing.

## Features

- **Markdown with GFM** — tables, task lists, strikethrough, footnotes, and front matter via comrak
- **Wiki-links** — connect pages with double-bracket links
- **Components** — notes, warnings, tabs, code groups, and mermaid diagrams using `:::directive` blocks
- **Full-text search** — client-side search powered by MiniSearch.js with a build-time JSON index
- **Mermaid diagrams** — flowcharts, sequence diagrams, and more via `:::mermaid` blocks
- **Theming** — CSS variables, custom stylesheets, and full template overrides with Tera
- **Live reload** — edit a file and your browser refreshes automatically
- **Static output** — build to plain HTML and deploy anywhere

## Explore the Docs

| Section | What You'll Learn |
|---------|-------------------|
| [[guides/getting-started\|Installation]] | Install DocAnvil and create your first project |
| [[guides/configuration\|Configuration]] | Customize `docanvil.toml` and `nav.toml` |
| [[guides/theming\|Theming]] | CSS variables, custom CSS, and template overrides |
| [[writing/markdown\|Markdown]] | All supported Markdown and GFM features |
| [[writing/wiki-links\|Links & Popovers]] | Wiki-link syntax and inline popovers |
| [[writing/components\|Components]] | Notes, warnings, tabs, and code groups |
| [[reference/cli\|CLI Commands]] | Every command, flag, and option |
| [[reference/project-structure\|Project Structure]] | Directory layout and page discovery |
| [[reference/css-variables\|CSS Variables]] | Complete variable reference with defaults |

:::note{title="Getting started?"}
Run `docanvil serve` in your project directory and open your browser — every change you save will appear instantly.
:::
