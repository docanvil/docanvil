# Changelog

All notable changes to DocAnvil will be documented in this file.

## [0.1.8] - 2026-02-15

### Added

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
