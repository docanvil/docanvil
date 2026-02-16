---
{
  "title": "CSS Variables"
}
---
# CSS Variables

DocAnvil's default theme is built entirely on CSS custom properties (variables). Override any of them to customize the look of your site without writing complex CSS.

## How to Override

::::tabs
:::tab{title="docanvil.toml"}
Set variables in the `[theme.variables]` section. Omit the `--` prefix:

```toml
[theme.variables]
color-primary = "#059669"
font-body = "Georgia, serif"
content-max-width = "960px"
```
:::
:::tab{title="custom.css"}
Override variables in a `:root` block in your custom CSS file:

```css
:root {
  --color-primary: #059669;
  --font-body: Georgia, serif;
  --content-max-width: 960px;
}
```
:::
::::

## Colors

| Variable | Default | Description |
|----------|---------|-------------|
| `--color-primary` | `#6366f1` | Primary accent color (links, active states, borders) |
| `--color-primary-light` | `#818cf8` | Lighter primary variant (h1 border, blockquote border) |
| `--color-bg` | `#ffffff` | Page background |
| `--color-bg-secondary` | `#f8fafc` | Secondary background (sidebar, table headers, blockquotes) |
| `--color-text` | `#1e293b` | Main text color |
| `--color-text-muted` | `#64748b` | Muted text (headings h4, separators, footer) |
| `--color-border` | `#e2e8f0` | Border color used throughout |
| `--color-link` | `#6366f1` | Link text color |
| `--color-link-hover` | `#4f46e5` | Link hover color |
| `--color-code-bg` | `#f1f5f9` | Background for inline code and code blocks |
| `--color-note-bg` | `#eef2ff` | Note admonition background |
| `--color-note-border` | `#818cf8` | Note admonition left border |
| `--color-warning-bg` | `#fff7ed` | Warning admonition background |
| `--color-warning-border` | `#f97316` | Warning admonition left border |

## Typography

| Variable | Default | Description |
|----------|---------|-------------|
| `--font-body` | `system-ui, -apple-system, "Segoe UI", Roboto, sans-serif` | Body font stack |
| `--font-mono` | `"SF Mono", Consolas, "Liberation Mono", Menlo, monospace` | Monospace font for code |
| `--font-size-base` | `16px` | Base font size |
| `--font-size-sm` | `0.875rem` | Small font size (table headers, footnotes) |
| `--line-height-tight` | `1.3` | Tight line height for headings |
| `--heading-letter-spacing` | `-0.02em` | Letter spacing for headings |

## Layout

| Variable | Default | Description |
|----------|---------|-------------|
| `--sidebar-width` | `260px` | Width of the navigation sidebar |
| `--content-max-width` | `800px` | Maximum width of the content area |

## Sidebar

| Variable | Default | Description |
|----------|---------|-------------|
| `--color-sidebar-hover` | `#eef2ff` | Background on sidebar link hover |
| `--color-sidebar-active-bg` | `#eef2ff` | Background of the active sidebar link |
| `--color-sidebar-active-text` | `#4f46e5` | Text color of the active sidebar link |

## Navigation

| Variable | Default | Description |
|----------|---------|-------------|
| `--nav-filter-bg` | `#ffffff` | Background of the filter input |
| `--nav-filter-border` | `var(--color-border)` | Border color of the filter input |
| `--nav-group-toggle-hover` | `rgba(99, 102, 241, 0.06)` | Background on group toggle hover |

## Shadows

| Variable | Default | Description |
|----------|---------|-------------|
| `--shadow-sm` | `0 1px 2px rgba(0, 0, 0, 0.05)` | Small shadow (code blocks, admonitions) |
| `--shadow-md` | `0 4px 6px -1px rgba(0, 0, 0, 0.07), 0 2px 4px -2px rgba(0, 0, 0, 0.05)` | Medium shadow (popovers, images) |

## Border Radius

| Variable | Default | Description |
|----------|---------|-------------|
| `--radius-sm` | `4px` | Small radius (inline code, nav items, filter input) |
| `--radius-md` | `6px` | Medium radius (code blocks, tables, popovers) |
| `--radius-lg` | `8px` | Large radius (admonitions) |

## Transitions

| Variable | Default | Description |
|----------|---------|-------------|
| `--transition-fast` | `150ms ease` | Fast transitions (hover states) |
| `--transition-normal` | `200ms ease` | Normal transitions (chevron rotation) |

## Focus

| Variable | Default | Description |
|----------|---------|-------------|
| `--color-focus-ring` | `rgba(99, 102, 241, 0.4)` | Focus ring color for keyboard navigation |

:::note
All color values use hex or rgba notation. Font stacks use standard CSS syntax with fallbacks. Sizes accept any CSS length unit (px, rem, em).
:::

## Related Pages

- [[guides/theming|Theming]] — how to apply variable overrides and custom CSS
- [[guides/configuration|Configuration]] — `docanvil.toml` reference
