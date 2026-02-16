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
    std::fs::create_dir_all(project_dir.join("docs/api"))?;
    std::fs::create_dir_all(project_dir.join("docs/reference"))?;
    std::fs::create_dir_all(project_dir.join("theme"))?;
    std::fs::create_dir_all(project_dir.join("assets"))?;

    // Write docanvil.toml
    let config = format!(
        r##"[project]
name = "{name}"
content_dir = "docs"
# logo = "assets/logo.png"
# favicon = "assets/favicon.ico"

[build]
output_dir = "dist"
# site_url = "https://example.com/"

[theme]
custom_css = "theme/custom.css"
# Set CSS variables to customize the theme:
# [theme.variables]
# color-primary = "#6366f1"
# font-body = "Georgia, serif"

# [syntax]
# enabled = true
# theme = "base16-ocean.dark"

# [charts]
# enabled = true
# mermaid_version = "11"

# [search]
# enabled = true
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

## Explore

- [[overview|API Reference]] — browse the API docs
- [[components]] — see the built-in components in action
- [[markdown]] — learn about supported Markdown features

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

    // Write api/overview.md
    let api_overview = r##"# API Overview

This section documents the REST API. Use these pages as a starting point for
writing your own API reference.

## Sections

- [[endpoints|REST Endpoints]] — available routes and methods
- [[authentication]] — how to authenticate requests

:::note{title="Tip"}
API documentation works great with tables and code blocks — see
[[markdown|Markdown Features]] for the full list of supported syntax.
:::

## Quick Example

```bash
curl -H "Authorization: Bearer TOKEN" https://api.example.com/v1/users
```

See [[getting-started]] for project setup instructions.
"##;
    std::fs::write(project_dir.join("docs/api/overview.md"), api_overview)?;

    // Write api/endpoints.md
    let api_endpoints = r##"# REST Endpoints

Below is an example endpoint reference. Replace these with your own API routes.

## Endpoints

| Method | Path             | Description        |
|--------|------------------|--------------------|
| GET    | `/v1/users`      | List all users     |
| POST   | `/v1/users`      | Create a new user  |
| GET    | `/v1/users/:id`  | Get user by ID     |
| DELETE | `/v1/users/:id`  | Delete a user      |

## Example Request

```bash
curl https://api.example.com/v1/users
```

## Example Response

```json
{
  "users": [
    { "id": 1, "name": "Alice" },
    { "id": 2, "name": "Bob" }
  ]
}
```

All requests require a valid token — see [[authentication]] for details.
For an overview of the API, visit [[overview|API Overview]].
"##;
    std::fs::write(project_dir.join("docs/api/endpoints.md"), api_endpoints)?;

    // Write api/authentication.md
    let api_auth = r##"# Authentication

All API requests must include a bearer token in the `Authorization` header.

## Obtaining a Token

```bash
curl -X POST https://api.example.com/v1/auth/token \
  -d '{"username": "alice", "password": "secret"}'
```

Response:

```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 3600
}
```

## Using the Token

Include the token in every request:

```bash
curl -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..." \
  https://api.example.com/v1/users
```

:::warning{title="Security"}
Never commit tokens to version control. Use environment variables or a secrets
manager instead.
:::

See [[endpoints|REST Endpoints]] for the available routes, or head back to the
[[overview|API Overview]].
"##;
    std::fs::write(project_dir.join("docs/api/authentication.md"), api_auth)?;

    // Write reference/components.md
    let ref_components = r##"# Components

DocAnvil includes several built-in components you can use in your Markdown files
with the `:::name{attrs}` directive syntax.

## Note

:::note{title="Information"}
This is a note component. Use it to highlight important information.
:::

## Warning

:::warning{title="Caution"}
This is a warning component. Use it to call out potential issues.
:::

## Tabs

:::tabs
```rust tab="Rust"
fn main() {
    println!("Hello from Rust!");
}
```

```python tab="Python"
print("Hello from Python!")
```

```javascript tab="JavaScript"
console.log("Hello from JavaScript!");
```
:::

## Usage

Directives use fenced syntax:

````markdown
:::note{title="My Title"}
Content goes here.
:::
````

See [[markdown|Markdown Features]] for more syntax, or check the
[[configuration]] page for theme customization options.
"##;
    std::fs::write(
        project_dir.join("docs/reference/components.md"),
        ref_components,
    )?;

    // Write reference/markdown.md
    let ref_markdown = r##"# Markdown Features

DocAnvil supports GitHub Flavored Markdown (GFM) with several extensions.

## Tables

| Feature       | Supported |
|---------------|-----------|
| Tables        | Yes       |
| Task lists    | Yes       |
| Strikethrough | Yes       |
| Footnotes     | Yes       |
| Front matter  | Yes       |

## Task Lists

- [x] Set up project with `docanvil init`
- [x] Start dev server with `docanvil serve`
- [ ] Write your first page
- [ ] Customize the theme

## Strikethrough

This text has ~~strikethrough~~ formatting.

## Footnotes

DocAnvil is built on comrak[^1], which supports GFM footnotes[^2].

[^1]: comrak is a CommonMark + GFM compatible Markdown parser written in Rust.
[^2]: Footnotes render as numbered references at the bottom of the page.

## Wiki-Links

Link to other pages using double-bracket syntax:

- `[[page-name]]` — links using the page title (e.g. [[components]])
- `[[page-name|custom text]]` — links with custom display text (e.g. [[overview|API docs]])

## Code Blocks

Fenced code blocks support syntax highlighting:

```rust
fn greet(name: &str) {
    println!("Hello, {}!", name);
}
```

See [[components]] for the built-in directive components, or visit the
[[overview|API Reference]] for example API documentation.
"##;
    std::fs::write(project_dir.join("docs/reference/markdown.md"), ref_markdown)?;

    // Write nav.toml
    let nav_toml = r##"# Navigation configuration — controls sidebar ordering and structure.
# Remove this file to fall back to auto-discovery (alphabetical order).

# Simple page link
[[nav]]
page = "index"

# Labeled separator — renders as a section heading
[[nav]]
separator = "Guides"

[[nav]]
page = "guides/getting-started"

[[nav]]
page = "guides/configuration"

# Unlabeled separator — renders as a horizontal line
[[nav]]
separator = true

# Collapsible group with a linked header (clicking the label navigates to api/overview)
[[nav]]
label = "API Reference"
page = "api/overview"
group = [
  { separator = "Core" },
  { page = "api/endpoints", label = "REST Endpoints" },
  { page = "api/authentication" },
]

# Collapsible group (no linked header) containing a nested child group
[[nav]]
label = "Reference"
group = [
  { page = "reference/components" },
  { label = "Writing", group = [
    { page = "reference/markdown" },
  ]},
]

# Autodiscover — automatically include all pages from a folder.
# Standalone autodiscover inlines pages at this position:
# [[nav]]
# autodiscover = "guides"
#
# Wrap autodiscovered pages in a collapsible group:
# [[nav]]
# label = "API Reference"
# autodiscover = "api"
#
# Group with clickable header + autodiscovered children:
# [[nav]]
# label = "Guides"
# page = "guides/index"
# autodiscover = "guides"
"##;
    std::fs::write(project_dir.join("nav.toml"), nav_toml)?;

    // Write custom.css starter template
    let custom_css = include_str!("../theme/starter_custom.css");
    std::fs::write(project_dir.join("theme/custom.css"), custom_css)?;

    // Write default docanvil.js for user customization
    let default_js = include_str!("../theme/default/docanvil.js");
    std::fs::write(project_dir.join("theme/docanvil.js"), default_js)?;

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
