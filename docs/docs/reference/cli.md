---
title: CLI Commands
---
# CLI Commands

DocAnvil provides three subcommands: `new`, `serve`, and `build`.

## Global Flags

| Flag | Description |
|------|-------------|
| `--verbose` | Enable verbose output |
| `--quiet` | Suppress non-error output |

## `docanvil new`

Scaffold a new documentation project.

```bash
docanvil new <name>
```

| Argument | Required | Description |
|----------|----------|-------------|
| `name` | Yes | Directory name for the new project |

Creates a project directory with:

- `docanvil.toml` — project configuration
- `nav.toml` — navigation structure
- `docs/` — content directory with starter pages
- `theme/custom.css` — empty custom stylesheet

:::code-group
```bash
# Create a docs project
docanvil new my-docs
```

```bash
# Create and immediately start serving
docanvil new my-docs && cd my-docs && docanvil serve
```
:::

## `docanvil serve`

Start a development server with live reload.

```bash
docanvil serve [--host <address>] [--port <port>] [--path <dir>]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--host` | `127.0.0.1` | Address to bind the server to |
| `--port` | `3000` | Port number |
| `--path` | `.` | Path to the project root |

The server:

- Builds the site on startup
- Watches all project files for changes (Markdown, TOML, CSS, templates)
- Rebuilds affected pages on file change
- Notifies the browser via WebSocket at `/__docanvil_ws`
- The browser reloads automatically — no manual refresh needed

:::code-group
```bash
# Default: localhost:3000
docanvil serve
```

```bash
# Custom host and port
docanvil serve --host 0.0.0.0 --port 8080
```

```bash
# Verbose output to see rebuild events
docanvil serve --verbose
```

```bash
# Serve a project from another directory
docanvil serve --path ../my-docs
```
:::

## `docanvil build`

Generate the static HTML site for deployment.

```bash
docanvil build [--out <path>] [--clean] [--path <dir>]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--out` | `dist` | Output directory for the generated site |
| `--clean` | `false` | Remove the output directory before building |
| `--strict` | `false` | Emit warnings as errors and exit with non-zero code |
| `--path` | `.` | Path to the project root |

The build pipeline processes each page through:

1. Directive expansion (components)
2. Popover conversion
3. Markdown rendering (comrak with GFM)
4. Wiki-link resolution
5. Inline attribute injection
6. Template wrapping (Tera layout)

Static assets (custom CSS, images) are copied to the output directory.

:::code-group
```bash
# Default build to dist/
docanvil build
```

```bash
# Clean build to a custom directory
docanvil build --out public --clean
```

```bash
# Build a project from another directory
docanvil build --path ../my-docs
```
:::

:::note
Broken wiki-links are reported as warnings during build. Check the output for any "broken link" messages to find references to pages that don't exist.
:::

## Related Pages

- [[guides/getting-started|Installation]] — install and create your first project
- [[guides/configuration|Configuration]] — `docanvil.toml` and `nav.toml` reference
