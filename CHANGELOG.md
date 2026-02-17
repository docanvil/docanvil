# Changelog

All notable changes to DocAnvil will be documented in this file.

## [0.3.2] - 2026-02-17

### Added

- Structured exit codes for CI pipelines — `build`, `doctor`, and all commands now return meaningful exit codes instead of a blanket `1` for every error:
  - `0` — success
  - `1` — general / IO failure
  - `2` — configuration error (missing or invalid `docanvil.toml`)
  - `3` — content validation error (missing content dir, strict-mode warnings, doctor failures)
  - `4` — theme / rendering error
  - `5` — internal error (panic / bug)
- New error variants: `ConfigNotFound`, `DoctorFailed`, and `General` for more precise error categorization
- Panic hook that prints a friendly "this is a bug" message with a link to the issue tracker
- Recovery hints on all error messages and build warnings — every error and warning now includes an actionable suggestion (e.g., "Run 'docanvil doctor --fix' to create it automatically")
- Colored error output matching the doctor's style (`error:` in red, `hint:` dimmed)

### Changed

- `docanvil doctor --strict` now returns exit code `3` (content validation) instead of `1`
- `docanvil theme` no longer calls `process::exit()` directly — missing config returns exit code `2`, overwrite guard returns `0`
- Non-rendering runtime errors (directory exists, runtime setup, watcher, invalid address) now use `General` instead of `Render`, mapping to exit code `1` rather than `4`

## [0.3.1] - 2026-02-16

### Changed

- Deduplicated nav.rs by extracting a `NavItem` trait for shared fields between `NavEntry` and `NavGroupItem`, unifying validation and node-building logic
- Merged near-identical `render_nav` / `render_nav_item` into a single recursive `render_nav_node` with proper indentation at all nesting depths
- Consolidated duplicate HTML escape functions from `code_group.rs` and `popovers.rs` into a shared `util::html_escape`
- Simplified `generate_theme` signature in theme.rs by grouping color parameters into a `ThemeColors` struct (11 params → 6)
- Removed unused `out` parameter from `run_with_options` and the watcher

### Fixed

- Resolved all clippy warnings: removed dead code (`warn_unclosed_directive`, `ConfigNotFound` variant, `DirectiveBlock::depth` field), collapsed nested if-let patterns, removed a needless borrow
- Popover IDs now reset between builds, preventing non-deterministic output during dev-server rebuilds

## [0.3.0] - 2026-02-16

### Changed

- Switched front matter format from YAML to JSON, removing the deprecated `serde_yml` dependency in favor of the already-present `serde_json`
- Extracted inline JavaScript (~460 lines) from `layout.html` into a separate `docanvil.js` file served as an external `<script>`, reducing per-page HTML size
- Production builds (`docanvil build`) now minify the JS automatically via `oxc` (compress + mangle, ~36% reduction)
- External JS file includes a content-hash cachebust query string for reliable cache invalidation
- `docanvil init` now scaffolds `theme/docanvil.js` for user customization alongside `theme/custom.css`
- User JS overrides are supported by placing a `theme/docanvil.js` in the project root

## [0.1.9] - 2026-02-15

### Added

- Interactive `docanvil theme` command to generate a custom color theme
  - Prompts for primary and warning/secondary colors with hex validation
  - Derives all 14 color-related CSS variables from the two inputs
  - Writes a commented `theme/custom.css` and updates `docanvil.toml` automatically
  - `--overwrite` flag to replace existing theme customizations
  - `--path` flag to target a project from any directory
- Dark mode support with configurable `color_mode` setting
  - Three modes: `"light"` (default), `"dark"`, and `"both"` (light + dark with toggle)
  - When `color_mode = "both"`: sun/moon toggle button in the header, OS `prefers-color-scheme` detection, and localStorage persistence across pages
  - When `color_mode = "dark"`: dark mode always active, no toggle
  - Separate color palettes for light and dark modes with independently customizable primary/secondary colors
  - `docanvil theme` generator extended with color mode selection and dual-palette prompts
  - Flash-prevention script to avoid light-mode flicker on dark-mode pages

## [0.1.8] - 2026-02-15

### Added

- Extended Markdown support: superscript (`^text^`), subscript (`~text~`), highlight (`==text==`), emoji shortcodes (`:smile:`), and description lists
- Custom heading IDs via same-line `{#id}` syntax (e.g., `### Heading {#custom-id}`)
- Copy-to-clipboard button on code blocks — appears on hover, shows checkmark on success
- Clickable anchor links on headings — hover to reveal, click to copy deep link
- Mobile navigation toggle — hamburger menu with slide-out sidebar and backdrop overlay
- Custom 404 page — generated automatically during `docanvil build`
- On-page table of contents — fixed right sidebar built from h2/h3 headings with scroll-spy highlighting
- Previous/next page navigation — sequential links at the bottom of each page based on nav order
- Added `docanvil doctor` command to diagnose project configuration and content issues
  - Checks project structure, config validity, theme files, content health, and output writability
  - Detects broken wiki-links, unclosed directives, front-matter parse errors, and duplicate slugs
  - `--fix` flag to automatically create missing directories, index pages, and CSS files
  - `--strict` flag for CI use (exits with code 1 on any warnings or errors)
  - `--path` flag to target a project from any directory

## [0.1.7] - 2026-02-14

### Added

- Added `--path` option to `serve` and `build` commands to target a project from any directory
- Added front matter handling for the addition of SEO oriented meta tags in the outputted pages
- Added autodiscover option to navigation config so we can autodiscover specific folders

### Changed

- Update the search index building to include breadcrumbs for the full path to results

## [0.1.6] - 2026-02-13

### Added

- Add `--strict` build flag to emit non-zero exit code for use in CI/CD

### Changed

- Moved the `init` command to `new` and updated the docs
- Updated the project to 2024 edition of rust

## [0.1.5] - 2026-02-13

### Added

- Automatic `robots.txt` and `sitemap.xml` generation during `docanvil build`
- Optional `site_url` setting in `[build]` config for absolute sitemap URLs
- Warning when `site_url` is not configured (sitemap falls back to relative URLs)

## [0.1.4] - 2026-02-13

### Added

- Full-text search with client-side MiniSearch.js — build-time JSON index, lazy-loaded on first focus, with prefix/fuzzy matching, keyboard navigation, and click-outside dismiss
- Mermaid diagram support via `:::mermaid` directive — renders flowcharts, sequence diagrams, and other Mermaid charts in the browser
- `[search]` config section in `docanvil.toml` to enable/disable search (enabled by default)
- `[charts]` config section in `docanvil.toml` to enable/disable Mermaid and configure version (enabled by default)
- Search input in header with dropdown results
- `search-index.json` generated during build when search is enabled

## [0.1.3] - 2026-02-12

### Added

- Image asset handling in the build pipeline
- Logo and favicon support via `[project]` config in `docanvil.toml`

### Changed

- Tweaked the default theme styling for project docs

## [0.1.2] - 2026-02-12

### Added

- Server-side syntax highlighting using syntect with the `default-fancy` feature set
- Tabs component support in documentation pages

### Changed

- Restructured the docs navigation
- Updated getting started guide to use tabs instead of codegroup

## [0.1.1] - 2026-02-11

### Added

- Configurable base URL support for deployment to subdirectories
- Configurable navigation structure via `docanvil.toml`
- Popover tooltips for wiki-links in generated docs
- Nested navigation in the sidebar
- GitHub Actions workflow for publishing to GitHub Pages

### Fixed

- Output flag (`--out`) in the `docanvil build` command

### Changed

- Improved generated styles and overall theme polish
- Better popover display for missing page links

## [0.1.0] - 2026-02-11

### Added

- Initial release of DocAnvil
- Markdown to HTML static site generation with comrak (GFM extensions)
- Wiki-link resolution (`[[page]]` and `[[page|text]]`)
- Fenced directive components (`:::note`, `:::warning`, `:::tabs`, `:::code-group`)
- Inline attribute injection (`{.class #id}`)
- Tera-based templating with `{% block %}` overrides
- CSS-variable-based default theme
- Dev server with live reload via axum and WebSocket
- File watcher with debounced rebuilds
- `docanvil init` project scaffolding
- `docanvil serve` dev server
- `docanvil build` static output generation
- Configurable via `docanvil.toml`
