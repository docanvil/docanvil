# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DocAnvil is a Rust-based static documentation generator that converts Markdown into HTML sites. It features live reloading, custom components, configurable styling, and static output for deployment anywhere.

## Build Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo test                     # Run all tests
cargo test <test_name>         # Run a single test
cargo clippy                   # Lint
cargo fmt                      # Format code
cargo install --path .         # Install locally
```

## CLI Interface

The binary exposes three subcommands:

- `docanvil init <name>` — scaffold a new documentation project
- `docanvil serve [--host <addr>] [--port <port>]` — dev server with hot reload (defaults: 127.0.0.1:3000)
- `docanvil build [--out <path>] [--clean]` — generate static HTML site (default output: `dist/`)

Global flags: `--verbose`, `--quiet`

## Architecture

### Module Structure

```
src/
  main.rs                    # clap CLI dispatch
  cli/
    mod.rs                   # Cli struct (clap derive), Command enum
    init.rs                  # docanvil init — scaffolds project directory
    serve.rs                 # docanvil serve — starts tokio runtime + server
    build.rs                 # docanvil build — orchestrates full build pipeline
  config.rs                  # docanvil.toml parsing (serde + toml)
  project.rs                 # PageInventory, NavNode, file discovery
  error.rs                   # thiserror Error enum
  diagnostics.rs             # colored warnings (owo-colors)
  pipeline/
    mod.rs                   # process() — full pipeline: directives → markdown → wikilinks → attributes
    markdown.rs              # comrak rendering with GFM extensions
    wikilinks.rs             # [[link]] and [[link|text]] resolution against PageInventory
    directives.rs            # :::directive{attrs} pre-comrak pass (regex + stack-based)
    attributes.rs            # {.class #id} post-comrak injection
  components/
    mod.rs                   # Component trait, ComponentRegistry, ComponentContext
    builtin/
      mod.rs, note.rs, warning.rs, tabs.rs, code_group.rs
  theme/
    mod.rs                   # Theme resolution (rust-embed + user overrides)
    default/
      style.css              # CSS-variable-based default theme
      layout.html            # Tera template with {% block %} sections
  render/
    mod.rs
    templates.rs             # Tera engine wrapper, PageContext struct
    assets.rs                # Static asset + custom CSS copying
  server/
    mod.rs                   # axum router setup, server start
    watcher.rs               # notify file watcher with debounce
    websocket.rs             # WebSocket handler for live reload
```

### Pipeline Flow

```
Markdown source
  → directives.rs    (pre-comrak: :::name{attrs} → component HTML)
  → markdown.rs      (comrak: Markdown → HTML with GFM extensions)
  → wikilinks.rs     (resolve [[links]] against PageInventory)
  → attributes.rs    (inject {.class #id} into preceding HTML tags)
  → templates.rs     (Tera: wrap in layout with nav, CSS, scripts)
  → output file
```

### Key Design Decisions

- **Parser**: comrak with GFM extensions (tables, task lists, strikethrough, footnotes, front matter)
- **Wiki-links**: `[[page-name]]` / `[[page-name|display text]]` resolved against slug inventory
- **Components**: Fenced directives (`:::name{key="val"}`) parsed pre-comrak; inline attributes (`{.class}`) post-comrak
- **Component trait**: `name()` + `render(ctx) -> Result<String>`; registry maps names to `Box<dyn Component>`
- **Styling**: Layered — embedded CSS-variable theme + config overrides + user template overrides (Tera)
- **Templates**: Tera with `{% block %}` sections; embedded defaults via rust-embed, user overrides in `theme/templates/`
- **Server**: axum with tokio; broadcast channel connects file watcher → WebSocket → browser reload
- **Config**: `docanvil.toml` with `[project]`, `[build]`, `[theme]` sections; serde deserialization

### Dependencies

Core: `clap` (CLI), `comrak` (Markdown), `serde`/`toml` (config), `thiserror` (errors), `walkdir` (file discovery), `regex` (directive parsing)

Rendering: `tera` (templates), `rust-embed` (embedded assets)

Server: `axum` (HTTP + WebSocket), `tokio` (async runtime), `tower-http` (static file serving), `notify`/`notify-debouncer-mini` (file watching)

Polish: `miette` (diagnostics), `owo-colors` (colored output)
