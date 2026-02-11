# Configuration

docs is configured through `docanvil.toml` in the project root.

## Config Sections

```toml
[project]
name = "docs"
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

:::note{title="Tip"}
You can also add custom CSS rules in `theme/custom.css` for full control.
:::

See [[getting-started]] for installation steps.
