---
title: CLI Commands
---
# CLI Commands

DocAnvil provides five subcommands: `new`, `theme`, `doctor`, `serve`, and `build`.

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

## `docanvil theme`

Interactively generate a custom color theme for your project.

```bash
docanvil theme [--overwrite] [--path <dir>]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--overwrite` | `false` | Replace existing theme customizations |
| `--path` | `.` | Path to the project root |

The command prompts for two hex colors — a primary accent color and a warning/secondary color — then automatically derives all 14 color-related CSS variables and writes them to a `theme/custom.css` file. It also updates `docanvil.toml` to reference the generated CSS.

If existing theme customizations are detected (`custom_css` or `[theme.variables]` in config), the command exits with a helpful message unless `--overwrite` is passed.

:::code-group
```bash
# Generate a theme for the current project
docanvil theme
```

```bash
# Generate a theme for a project in another directory
docanvil theme --path ../my-docs
```

```bash
# Replace an existing theme
docanvil theme --overwrite
```
:::

### Derived Variables

From the two input colors, the following CSS variables are generated:

| Variable | Derivation |
|----------|-----------|
| `--color-primary` | Primary color as-is |
| `--color-primary-light` | Primary lightened 10% |
| `--color-link` | Same as primary |
| `--color-link-hover` | Primary darkened 10% |
| `--color-sidebar-hover` | Primary tinted to 95% lightness |
| `--color-sidebar-active-bg` | Primary tinted to 95% lightness |
| `--color-sidebar-active-text` | Primary darkened 10% |
| `--color-note-bg` | Primary tinted to 95% lightness |
| `--color-note-border` | Primary lightened 10% |
| `--color-mark-bg` | Primary at 12% opacity |
| `--nav-group-toggle-hover` | Primary at 6% opacity |
| `--color-focus-ring` | Primary at 40% opacity |
| `--color-warning-border` | Secondary color as-is |
| `--color-warning-bg` | Secondary tinted to 95% lightness |

:::note
After generating a theme, run `docanvil serve` to preview the result. You can edit the generated `theme/custom.css` file directly for further tweaks.
:::

## `docanvil doctor`

Diagnose project configuration and content issues before building.

```bash
docanvil doctor [--fix] [--strict] [--path <dir>]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--fix` | `false` | Automatically apply safe fixes (create missing dirs, files) |
| `--strict` | `false` | Exit with code 1 if any warnings or errors are found (for CI) |
| `--path` | `.` | Path to the project root |

The doctor runs five categories of checks:

1. **Project structure** — config file, content directory, index page
2. **Configuration** — TOML parsing, file references (logo, favicon), nav.toml validation
3. **Theme** — custom CSS file existence, layout template Tera syntax
4. **Content** — broken wiki-links, unclosed directives, front-matter YAML errors, duplicate slugs
5. **Output** — output directory writability

If no `docanvil.toml` is found, doctor prints a friendly message suggesting `docanvil new` and exits cleanly.

:::code-group
```bash
# Check the current project
docanvil doctor
```

```bash
# Check and auto-fix safe issues
docanvil doctor --fix
```

```bash
# Use in CI to fail on any issues
docanvil doctor --strict
```

```bash
# Check a project in another directory
docanvil doctor --path ../my-docs
```
:::

The `--fix` flag applies safe, non-destructive fixes:

| Issue | Fix applied |
|-------|-------------|
| Content directory missing | Creates the directory |
| No `index.md` at content root | Creates a minimal index page |
| Custom CSS file not found | Creates an empty CSS file at the configured path |

:::note
Run `docanvil doctor` again after `--fix` to verify all issues are resolved. Some fixes (like creating the content directory) may reveal additional issues on the next run.
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
