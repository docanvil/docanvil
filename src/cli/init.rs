use std::path::Path;

use owo_colors::OwoColorize;

use crate::error::Result;

pub fn run(name: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        return Err(crate::error::Error::Render(format!(
            "directory '{}' already exists",
            name
        )));
    }

    // Create project structure
    std::fs::create_dir_all(project_dir.join("docs/guides"))?;
    std::fs::create_dir_all(project_dir.join("theme"))?;

    // Write docanvil.toml
    let config = format!(
        r##"[project]
name = "{name}"
content_dir = "docs"

[build]
output_dir = "dist"

[theme]
custom_css = "theme/custom.css"
# Set CSS variables to customize the theme:
# [theme.variables]
# color-primary = "#6366f1"
# font-body = "Georgia, serif"
"##
    );
    std::fs::write(project_dir.join("docanvil.toml"), config)?;

    // Write initial index.md
    let index = format!(
        r##"# Welcome to {name}

This is your new documentation site, powered by [DocAnvil](https://github.com/docanvil/docanvil).

## Getting Started

Edit this file at `docs/index.md` to start writing your documentation.
Check out the guides to learn more:

- [[getting-started]] — install and run your first site
- [[configuration]] — customize your project settings

### Features

- **Markdown** with GFM extensions (tables, task lists, footnotes)
- **Wiki-style links**: Link to other pages with `[[page-name]]`
- **Custom components**: Use `:::note`, `:::warning`, `:::tabs` directives
- **Theming**: Customize with CSS variables in `docanvil.toml`
- **Live reload**: Run `docanvil serve` for instant preview

:::note{{title="Tip"}}
Run `docanvil serve` in this directory to see your docs with live reloading!
:::

## Customizing the Theme

Edit `theme/custom.css` to override any CSS variable or add your own styles.
You can also set variables directly in `docanvil.toml`:

```toml
[theme.variables]
color-primary = "#10b981"
font-body = "Georgia, serif"
```

## Next Steps

- Add more `.md` files to the `docs/` directory
- Customize your theme in `docanvil.toml` or `theme/custom.css`
- Run `docanvil build` to generate a static site
"##
    );
    std::fs::write(project_dir.join("docs/index.md"), index)?;

    // Write guides/getting-started.md
    let getting_started = format!(
        r##"# Getting Started

Welcome to {name}! This guide walks you through setup and first steps.

## Installation

Install DocAnvil using Cargo:

```bash
cargo install docanvil
```

## Create a New Project

```bash
docanvil init {name}
cd {name}
```

## Start the Dev Server

```bash
docanvil serve
```

Open [http://localhost:3000](http://localhost:3000) in your browser. Changes to
any `.md` file will reload the page automatically.

## Build for Production

```bash
docanvil build
```

Static HTML is written to the `dist/` directory — deploy it anywhere.

See [[configuration]] for details on customizing your project, or head
back to the [[index|home page]].
"##
    );
    std::fs::write(
        project_dir.join("docs/guides/getting-started.md"),
        getting_started,
    )?;

    // Write guides/configuration.md
    let configuration = format!(
        r##"# Configuration

{name} is configured through `docanvil.toml` in the project root.

## Config Sections

```toml
[project]
name = "{name}"
content_dir = "docs"

[build]
output_dir = "dist"

[theme]
custom_css = "theme/custom.css"

[theme.variables]
color-primary = "#6366f1"
```

## Project

| Key           | Description                | Default  |
|---------------|----------------------------|----------|
| `name`        | Site title in the sidebar  | required |
| `content_dir` | Markdown source directory  | `"docs"` |

## Theme Variables

Override any CSS variable under `[theme.variables]`. Common options:

- `color-primary` — accent color used for links and highlights
- `font-body` — base font stack
- `sidebar-width` — sidebar width (e.g. `"280px"`)

:::note{{title="Tip"}}
You can also add custom CSS rules in `theme/custom.css` for full control.
:::

See [[getting-started]] for installation steps.
"##
    );
    std::fs::write(
        project_dir.join("docs/guides/configuration.md"),
        configuration,
    )?;

    // Write nav.toml
    let nav_toml = r##"# Navigation configuration — controls sidebar ordering and structure.
# Remove this file to fall back to auto-discovery (alphabetical order).

[[nav]]
page = "index"

[[nav]]
separator = "Guides"

[[nav]]
page = "guides/getting-started"

[[nav]]
page = "guides/configuration"

# Examples of other nav features:
#
# Unlabeled separator (horizontal line):
# [[nav]]
# separator = true
#
# Collapsible group:
# [[nav]]
# label = "API Reference"
# group = [
#   { page = "api/overview" },
#   { page = "api/endpoints", label = "REST Endpoints" },
# ]
#
# Group with a linked header:
# [[nav]]
# label = "Advanced"
# page = "advanced/index"
# group = [
#   { page = "advanced/plugins" },
#   { page = "advanced/deployment" },
# ]
"##;
    std::fs::write(project_dir.join("nav.toml"), nav_toml)?;

    // Write custom.css starter template
    let custom_css = include_str!("../theme/starter_custom.css");
    std::fs::write(project_dir.join("theme/custom.css"), custom_css)?;

    eprintln!(
        "{} Created project '{}' at {}",
        "✓".green().bold(),
        name.bold(),
        project_dir.display()
    );
    eprintln!();
    eprintln!("  {} {}", "cd".dimmed(), name);
    eprintln!("  {} serve", "docanvil".dimmed());
    eprintln!();

    Ok(())
}
