# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Rules

- **Never add `Co-Authored-By` to commit messages.**
- **Never bump the version in `Cargo.toml`.** Version bumps are handled manually by the maintainer.
- **Keep CLAUDE.md up to date.** After making material changes that affect details in this file (new modules, renamed files, new CLI commands, changed types, new dependencies, etc.), identify the necessary CLAUDE.md updates and confirm them with the user before applying.
- **Tone for written content (docs, README, CLI output, comments):** Friendly, welcoming, and approachable — but grounded and serious. DocAnvil is a production-ready tool for real projects. The voice should reflect that: confident, practical, and focused on helping people ship great docs fast without the usual headaches. Avoid fluff, corporate-speak, or anything patronizing. Emoji are welcome where they add warmth or clarity.

## Project Overview

DocAnvil is a Rust-based static documentation generator that converts Markdown into HTML sites. It features live reloading, custom components, syntax highlighting, search indexing, SEO generation, configurable styling, and static output for deployment anywhere.

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

Unit tests are inline `#[cfg(test)]` modules within their respective source files. Integration tests live in `tests/` — `build_integration.rs` (library-level build pipeline tests), `cli_integration.rs` (binary subprocess tests) and `update_integration.rs` (self-update against a local axum stub, no network), with shared helpers in `integration_helpers.rs`.

## CLI Interface

The binary exposes seven subcommands:

- `docanvil new <name>` — scaffold a new documentation project
- `docanvil serve [--host <addr>] [--port <port>] [--path <path>]` — dev server with hot reload (defaults: 127.0.0.1:3000); builds into a per-project temp dir (never `output_dir`) with `base_url` forced to `/`
- `docanvil build [--path <path>] [--out <path>] [--clean] [--strict]` — generate static HTML site (default output: `dist/`)
- `docanvil theme [--path <path>] [--overwrite]` — interactive color theme generator
- `docanvil doctor [--path <path>] [--fix] [--strict] [--format human|checkstyle|junit]` — project diagnostics with auto-fix
- `docanvil export pdf --out <path> [--path <path>] [--locale <code>]` — export docs as a single PDF (requires Chrome/Chromium)
- `docanvil update [--check] [--yes] [--version <x.y.z>]` — self-update from GitHub releases (refuses to replace `cargo install` copies)

Global flags: `--verbose`, `--quiet`

CLI definition: `src/cli/mod.rs`

## Architecture

### Module Structure

```
src/
  lib.rs                       # Public module re-exports (for integration tests)
  main.rs                      # clap CLI dispatch
  config.rs                    # docanvil.toml parsing (serde + toml)
  project.rs                   # PageInventory, NavNode, file discovery
  nav.rs                       # nav.toml parsing (NavEntry, NavGroupItem, autodiscover)
  search.rs                    # Search index generation (extract sections from HTML)
  seo.rs                       # robots.txt and sitemap.xml generation
  error.rs                     # thiserror Error enum
  diagnostics.rs               # Colored warnings (owo-colors)
  util.rs                      # HTML escape utility

  cli/
    mod.rs                     # Cli struct (clap derive), Command enum
    new.rs                     # docanvil new — scaffold project
    serve.rs                   # docanvil serve — tokio runtime + dev server
    build.rs                   # docanvil build — full build pipeline orchestration
    theme.rs                   # docanvil theme — interactive color theme generator
    doctor.rs                  # docanvil doctor — runs diagnostic checks
    color.rs                   # Hex/RGB/HSL color conversion (used by theme)
    update.rs                  # docanvil update — prompts and output for self-update
    export/
      mod.rs                   # ExportArgs, ExportFormat, dispatch()
      pdf.rs                   # docanvil export pdf — Chrome-based PDF export
      cdp.rs                   # Chrome DevTools Protocol session (CDP over WebSocket)

  doctor/
    mod.rs                     # Diagnostic runner, severity levels, auto-fix support
    checks/
      mod.rs
      config.rs                # Config validation checks
      content.rs               # Markdown content checks
      locale.rs                # Translation coverage checks (i18n)
      output.rs                # Output directory checks
      project.rs               # Project structure checks
      readability.rs           # Markdown readability checks (headings, alt text, paragraph length)
      theme.rs                 # Theme checks
      version.rs               # Versioning checks (current in enabled, version dirs exist and have pages)

  pipeline/
    mod.rs                     # process() — orchestrates all pipeline stages
    directives.rs              # :::directive{attrs} pre-comrak pass (block + inline)
    popovers.rs                # ^[content] → popover HTML conversion
    headings.rs                # Custom heading ID extraction {#id} and auto-generation
    frontmatter.rs             # JSON front matter extraction
    markdown.rs                # comrak rendering with GFM extensions
    syntax.rs                  # syntect-based code block highlighting
    wikilinks.rs               # [[link]] and [[link|text]] resolution against PageInventory
    attributes.rs              # {.class #id} post-comrak injection into HTML tags
    images.rs                  # Relative image path rewriting

  components/
    mod.rs                     # Component trait, ComponentRegistry, ComponentContext
    builtin/
      mod.rs
      note.rs                  # :::note admonition
      warning.rs               # :::warning admonition
      tabs.rs                  # :::tabs container
      code_group.rs            # :::code-group container
      lozenge.rs               # :::lozenge badge/label spans
      mermaid.rs               # :::mermaid diagram blocks

  theme/
    mod.rs                     # Theme resolution (rust-embed + user overrides)
    default/
      layout.html              # Tera template with {% block %} sections
      pdf.html                 # Tera template for PDF export
      style.css                # CSS-variable-based default theme
      docanvil.js              # Client-side JS (live reload, popovers, interactivity)
      starter_custom.css       # Starter file for user custom theme overrides

  render/
    mod.rs
    templates.rs               # Tera engine wrapper, PageContext struct
    assets.rs                  # Static asset + custom CSS copying

  update/
    mod.rs                     # Release lookup (releases/latest redirect), checksum-verified download, extraction, self-replace
    notice.rs                  # Cached once-a-day "new version" notice shown by serve

  server/
    mod.rs                     # axum router setup, server start
    watcher.rs                 # notify file watcher with debounce
    websocket.rs               # WebSocket handler for live reload
```

Top-level `install/` holds `install.sh` (macOS/Linux) and `install.ps1` (Windows); both are uploaded as release assets.

### Pipeline Flow

The full rendering pipeline in `src/pipeline/mod.rs` runs these stages in order:

```
Markdown source
  → directives.rs      (pre-comrak: :::name{attrs} → component HTML, block + inline)
  → popovers.rs        (^[content] → popover spans)
  → headings.rs        (extract custom {#id} from headings)
  → markdown.rs        (comrak: Markdown → HTML with GFM extensions)
  → syntax.rs          (syntect: code block syntax highlighting)
  → wikilinks.rs       (resolve [[links]] against PageInventory)
  → attributes.rs      (inject {.class #id} into preceding HTML tags)
  → headings.rs        (inject auto-generated heading IDs)
  → output
```

### Key Design Decisions

- **Parser**: comrak with GFM extensions (tables, task lists, strikethrough, footnotes, front matter)
- **Front matter**: JSON format (not YAML)
- **Wiki-links**: `[[page-name]]` / `[[page-name|display text]]` resolved against slug inventory
- **Components**: Fenced directives (`:::name{key="val"}`) parsed pre-comrak; inline attributes (`{.class}`) post-comrak
- **Component trait**: `name()` + `render(ctx) -> Result<String>`; registry maps names to `Box<dyn Component>`
- **Syntax highlighting**: syntect with theme validation
- **Popovers**: `^[content]` syntax converted to interactive HTML spans
- **Navigation**: `nav.toml` with support for pages, groups, separators, labels, and autodiscover; per-locale `nav.{locale}.toml` and per-version `nav.{version}.toml` fall back to `nav.toml`
- **Search**: HTML sections extracted by heading for client-side search indexing
- **SEO**: Auto-generated robots.txt and sitemap.xml from PageInventory; multilingual hreflang tags (in-page + sitemap), canonical URLs, and og:locale when i18n is enabled
- **Styling**: Layered — embedded CSS-variable theme + config overrides + user template overrides (Tera)
- **Templates**: Tera with `{% block %}` sections; embedded defaults via rust-embed, user overrides in `theme/templates/`
- **Server**: axum with tokio; broadcast channel connects file watcher → WebSocket → browser reload
- **Config**: `docanvil.toml` with `[project]`, `[build]`, `[theme]`, `[syntax]`, `[charts]`, `[search]`, `[locale]`, `[version]`, `[pdf]`, `[doctor]` sections; serde deserialization
- **Versioning**: Version subdirectories inside `content_dir` (`docs/v2/…`, not file suffixes), version-prefixed output (`/v2/page.html`), per-version nav/search, version switcher, and a banner on older versions; combines with i18n (`/v2/en/page.html`)
- **Localisation**: Filename suffix convention (`page.en.md`), locale-prefixed output (`/en/page.html`), per-locale nav/search, language switcher with browser auto-detection
- **Self-update**: Only `docanvil update` and the `serve` notice touch the network; checksums (`SHA256SUMS`, or GitHub's asset digest for releases ≤ v1.1.3) are mandatory; the latest version comes from the `releases/latest` redirect, not the rate-limited API
- **Doctor**: Diagnostic checks with severity levels (Info, Warning, Error) and auto-fix support; includes translation coverage checks when i18n is enabled

### Key Types and Where They Live

| Type | File | Purpose |
|------|------|---------|
| `Config` | `config.rs` | Top-level config with sections: `ProjectConfig`, `BuildConfig`, `ThemeConfig`, `SyntaxConfig`, `ChartsConfig`, `SearchConfig`, `LocaleConfig`, `VersionConfig`, `PdfConfig`, `DoctorConfig` |
| `LocaleConfig` | `config.rs` | i18n config: `default`, `enabled`, `display_names`, `auto_detect`, `flags`. Helpers: `is_i18n_enabled()`, `default_locale()`, `locale_display_name()`, `locale_flag()`. Free fn: `is_rtl_locale(code)` → `bool` |
| `VersionConfig` | `config.rs` | Versioning config: `current`, `enabled`, `display_names`. Helpers on `Config`: `is_versioning_enabled()`, `current_version()`, `version_display_name()` |
| `PageInfo` | `project.rs` | Single page metadata: `source_path`, `output_path`, `title`, `slug`, `locale`, `version` |
| `PageInventory` | `project.rs` | All pages: `pages: HashMap<String, PageInfo>`, `ordered: Vec<String>`. Key methods: `scan()`, `resolve_link()`, `resolve_link_in_locale()`, `nav_tree()`, `nav_tree_for_locale()`, `slug_locale_coverage()` |
| `NavNode` | `project.rs` | Nav tree enum: `Page { label, slug }`, `Group { label, slug, children }`, `Separator { label }` |
| `NavEntry` | `nav.rs` | Parsed nav.toml entry: `page`, `label`, `separator`, `group`, `autodiscover` |
| `Error` | `error.rs` | Variants: `Io`, `ConfigParse { path, source }`, `ConfigNotFound`, `ContentDirNotFound`, `Render`, `General`, `UnsafeOutputDir { path, reason }`, `StrictWarnings`, `DoctorFailed { warnings, errors }`, `ChromeNotFound`, `Update { message, hint }` |
| `Source` | `update/mod.rs` | GitHub web + API base URLs for release lookups; `Source::github()` in prod, pointed at a local server in tests |
| `Component` trait | `components/mod.rs` | `name() -> &str` + `render(&ComponentContext) -> Result<String>` |
| `ComponentContext` | `components/mod.rs` | `attributes: HashMap<String, String>`, `body_raw: String`, `body_html: String` |
| `ComponentRegistry` | `components/mod.rs` | `with_builtins()` registers all builtin components. `render_block()` does lookup + render |
| `PageContext` | `render/templates.rs` | All template data: `page_title`, `content`, `nav_html`, CSS paths, `prev_page`/`next_page`, meta fields, feature flags, locale fields (`current_locale`, `current_flag`, `available_locales`, `locale_auto_detect`), SEO fields (`canonical_url`, `x_default_url`) |
| `LocaleInfo` | `render/templates.rs` | Language switcher data: `code`, `display_name`, `flag`, `url`, `absolute_url`, `is_current`, `has_page` |
| `SitemapLocaleConfig` | `seo.rs` | i18n data for sitemap hreflang: `enabled`, `default_locale`, `slug_coverage` |
| `Diagnostic` | `doctor/mod.rs` | `check`, `category`, `severity: Severity`, `message`, `file`, `line`, `fix: Option<Fix>` |
| `DirectiveBlock` | `pipeline/directives.rs` | Parsed `:::name{attrs}` block: `name`, `attributes`, `body` |
| `PdfConfig` | `config.rs` | PDF export config: `author`, `cover_page`, `custom_css`, `paper_size` (optional, e.g. `"A4"`, `"Letter"`) |
| `DoctorConfig` | `config.rs` | Doctor / linting config: `max_paragraph_words` (default: 150; set to 0 to disable) |

### Build Flow (cli/build.rs)

1. `Config::load(project_root)` → config struct
2. `PageInventory::scan(content_dir, enabled_locales, default_locale, version)` → all pages with slugs (`version` is `None` outside versioned builds)
3. Pre-pass: read sources, extract front matter, apply slug overrides
4. **When versioning enabled:** per-version loop (scan each `content_dir/{version}/`, `load_nav_for_version()`, version-prefixed output and search index, i18n nested inside each version), then a root redirect to the current version and an early return
5. **When i18n enabled:** per-locale loop:
   - `load_nav_for_locale()` or `inventory.nav_tree_for_locale()` → locale-specific nav
   - Render pages with locale-prefixed URLs and locale-aware wiki-links
   - Write per-locale search index (`{locale}/search-index.json`)
   - Emit missing translation warnings
6. **Otherwise:** single-pass rendering (backward compatible)
7. Copy shared assets (JS, CSS), generate robots.txt + sitemap.xml, 404 page

### How to Extend

**Add a builtin component:**
1. Create `src/components/builtin/my_comp.rs` — struct implementing `Component` trait (`name()` + `render()`)
2. Add `pub mod my_comp;` to `src/components/builtin/mod.rs`
3. Register in `ComponentRegistry::with_builtins()` in `src/components/mod.rs`

**Add a pipeline stage:**
1. Create `src/pipeline/my_stage.rs` with a `pub fn process(html: &str, ...) -> String`
2. Add `pub mod my_stage;` to `src/pipeline/mod.rs`
3. Insert the call in the `process()` function chain in `src/pipeline/mod.rs`

**Add a doctor check:**
1. Create `src/doctor/checks/my_check.rs` returning `Vec<Diagnostic>`
2. Add `pub mod my_check;` to `src/doctor/checks/mod.rs`
3. Call it from `run_checks()` in `src/doctor/mod.rs`

**Add a CLI subcommand:**
1. Add variant to `Command` enum in `src/cli/mod.rs`
2. Create `src/cli/my_cmd.rs` with a `pub fn run(...) -> Result<()>`
3. Add `pub mod my_cmd;` to `src/cli/mod.rs`
4. Add match arm in `src/main.rs`

### Conventions

- **Error handling**: `Result<T>` alias from `error.rs`; propagate with `?`; `thiserror` derives `Display`
- **Imports**: Always `use crate::module::Type` (absolute from crate root); no wildcards
- **Tests**: Inline `#[cfg(test)] mod tests` at bottom of each file; `tempfile::tempdir()` for filesystem tests
- **Config defaults**: Each config section struct has `#[serde(default)]` + explicit `impl Default`

### File Complexity (largest files — start here for deep changes)

Rough sizes, so this table doesn't go stale with every PR.

| Lines | File | Notes |
|-------|------|-------|
| ~1.9k | `doctor/checks/readability.rs` | Readability lints (headings, alt text, link text, paragraph length) |
| ~1.6k | `cli/build.rs` | Full build orchestration; the versioned build loop is the largest section |
| ~1.1k | `cli/export/pdf.rs` | PDF export: page assembly, cover page, Chrome printing |
| ~1.0k | `project.rs` | Nav tree construction, page discovery, render_nav() |
| ~700 | `update/mod.rs` | Release lookup, checksum verification, self-replace |
| ~700 | `doctor/mod.rs` | Diagnostic runner, fix application, output formats |
| ~500 | `cli/new.rs` | Project scaffolding templates |

### Dependencies

Core: `clap` (CLI), `comrak` (Markdown), `serde`/`serde_json`/`toml` (config), `thiserror` (errors), `walkdir` (file discovery), `regex` (directive parsing), `slug` (URL slugs)

Rendering: `tera` (templates), `rust-embed` (embedded assets), `syntect` (syntax highlighting), `oxc` (JS minification)

Server: `axum` (HTTP + WebSocket), `tokio` (async runtime), `tower-http` (static file serving), `notify`/`notify-debouncer-mini` (file watching)

Interactive: `dialoguer` (CLI prompts for theme generator), `toml_edit` (preserving TOML formatting)

Polish: `owo-colors` (colored output)

PDF export: `tungstenite` (sync WebSocket for CDP), `base64` (decode CDP `printToPDF` response)

Self-update: `ureq` (HTTP, rustls — no OpenSSL, keeps musl/ARM builds simple), `sha2` (checksums), `flate2`/`tar`/`zip` (archives), `self-replace` (swap the running binary, Windows-safe), `semver`, `dirs` (cache dir)

### CI & Release Workflows

- `ci.yml` (lint + 3-OS test matrix) skips docs-only changes; `docs.yml` builds `docs/` with `--strict` using the checkout's binary on every PR
- Release builds live in `.github/workflows/build-targets.yml` (6 targets: linux x86_64 gnu + musl, linux aarch64 musl, macOS arm64/x86_64, Windows x86_64; the installer picks the static musl builds on Linux, while self-update keeps a binary on its own libc and ARM is musl-only), reused by `release.yml`
- `ci.yml` runs that build plus an installer smoke test (real releases, incl. Windows PowerShell 5.1) only on `release/v*` PRs or PRs labelled `release-build` — normal PRs stay on the 3-OS test matrix
- Releases ship archives + `SHA256SUMS` + `install.sh` + `install.ps1`
- In `release.yml`, crates.io `publish` needs the GitHub `release` job, since a crates.io publish can't be undone
- Never interpolate `${{ github.head_ref }}` (or other PR-controlled values) directly into `run:`; pass via `env:`
