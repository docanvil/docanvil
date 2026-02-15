# Theming

DocAnvil's appearance is customizable through three layers, each building on the last:

1. **CSS variable overrides** in `docanvil.toml` — quick color and font changes
2. **Custom CSS file** — full control over any element
3. **Template overrides** — replace the entire HTML layout with Tera templates

## Quick Start: Theme Generator

The fastest way to customize your site's colors is the interactive theme generator:

```bash
docanvil theme
```

This prompts for a primary and secondary color, then generates a complete `theme/custom.css` with all derived color variables and updates your `docanvil.toml` automatically. Run `docanvil serve` afterward to preview the result.

See [[reference/cli|CLI Commands]] for the full list of options (`--overwrite`, `--path`).

## Dark Mode

DocAnvil supports light mode, dark mode, or both with an automatic toggle. Set the `color_mode` in your `docanvil.toml`:

```toml
[theme]
color_mode = "both"  # "light" (default) | "dark" | "both"
```

|  <div style="width:60px">Mode</div>  | Behavior |
|------|----------|
| `light` | Light palette only (default, current behavior) |
| `dark` | Dark palette only — dark backgrounds, light text |
| `both` | Light as default, with a sun/moon toggle in the header and OS `prefers-color-scheme` auto-detection |

### Using the Theme Generator

The easiest way to set up dark mode is through the theme generator:

```bash
docanvil theme
```

Select "Both (light + dark with toggle)" when prompted for color mode. You'll be asked for separate primary and secondary colors for each mode, and the generator will produce a single `theme/custom.css` with light variables in `:root`, dark variables in `[data-theme="dark"]`, and an `@media (prefers-color-scheme: dark)` block for OS auto-detection.

### How the Toggle Works

When `color_mode = "both"`:

- A sun/moon icon button appears in the header
- On first visit, the OS preference is respected via `prefers-color-scheme`
- Clicking the toggle switches between light and dark and saves the choice to `localStorage`
- The choice persists across page navigations and browser sessions
- A flash-prevention script in `<head>` ensures the page renders in the correct mode immediately

### Manual Dark Mode CSS

If you prefer to write your own dark mode CSS instead of using the generator, structure your `theme/custom.css` like this:

```css
/* Light mode */
:root {
  --color-primary: #6366f1;
  /* ... other light variables ... */
}

/* Dark mode — explicit toggle */
[data-theme="dark"] {
  --color-bg: #0f172a;
  --color-text: #f1f5f9;
  /* ... other dark variables ... */
}

/* Dark mode — OS preference */
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --color-bg: #0f172a;
    --color-text: #f1f5f9;
    /* ... same dark variables ... */
  }
}
```

The `[data-theme="dark"]` selector handles explicit user choice via the toggle, while the `@media` block handles OS-level preference when no explicit choice has been made.

## CSS Variables in Config

The simplest way to customize the theme. Add variables under `[theme.variables]` in `docanvil.toml`:

```toml
[theme.variables]
color-primary = "#059669"
color-primary-light = "#34d399"
color-link = "#059669"
color-link-hover = "#047857"
```

These are injected as a `:root` style block after the default theme, overriding the built-in values. Variable names omit the `--` prefix — DocAnvil adds it automatically.

## Custom CSS File

For more control, point `custom_css` to a stylesheet:

```toml
[theme]
custom_css = "theme/custom.css"
```

This file loads after both the default theme and config variable overrides, so it has the highest CSS specificity. Use it for:

- Additional variable overrides in a `:root` block
- Custom selectors targeting specific elements
- New styles for your own classes (via inline attributes)

## Common Customizations

:::code-group
```toml
# docanvil.toml — change accent color and font
[theme.variables]
color-primary = "#059669"
color-primary-light = "#34d399"
color-link = "#059669"
font-body = "Georgia, serif"
```

```css
/* theme/custom.css — wider content area with dark code blocks */
.content {
  max-width: 960px;
}

.content pre {
  background: #1e293b;
  color: #e2e8f0;
  border-color: #334155;
}

.content pre code {
  color: inherit;
}
```
:::

### Load Order

Styles are applied in this order (last wins):

1. Default theme (`style.css` embedded in the binary)
2. Config variables (`[theme.variables]` → `:root { ... }`)
3. Custom CSS file (`custom_css` path)

:::warning{title="Specificity"}
If a custom CSS rule doesn't seem to take effect, check that your selector is specific enough to override the default theme. Using `.content pre` is more specific than just `pre`.
:::

## Template Overrides

For complete control over the HTML structure, override the default Tera template. Create a file at `theme/templates/layout.html` in your project:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>{{ page_title }} — {{ project_name }}</title>
  <style>{{ default_css | safe }}</style>
  {% if css_overrides %}
  <style>:root { {{ css_overrides }} }</style>
  {% endif %}
  {% if custom_css_path %}
  <link rel="stylesheet" href="/{{ custom_css_path }}">
  {% endif %}
  {% block head %}{% endblock %}
</head>
<body>
  {% block sidebar %}
  <nav class="sidebar">
    <div class="project-name">{{ project_name }}</div>
    {{ nav_html | safe }}
  </nav>
  {% endblock %}

  <main class="content">
    {% block content %}
    {{ content | safe }}
    {% endblock %}

    {% block footer %}
    <div class="footer">
      Built with <a href="https://github.com/docanvil/docanvil">DocAnvil</a>
    </div>
    {% endblock %}
  </main>

  {% block scripts %}{% endblock %}
</body>
</html>
```

### Template Blocks

| Block | Purpose |
|-------|---------|
| `head` | Extra `<head>` content (fonts, meta tags, analytics) |
| `header` | Top header bar with project name and search |
| `sidebar` | The navigation sidebar |
| `content` | Main page content area |
| `footer` | Footer below content |
| `scripts` | JavaScript at end of body |

### Template Variables

| Variable | Type | Description |
|----------|------|-------------|
| `page_title` | String | Title of the current page |
| `project_name` | String | Project name from `docanvil.toml` |
| `default_css` | String | The full default stylesheet (use with `safe` filter) |
| `css_overrides` | String | CSS variable overrides from config |
| `custom_css_path` | String | Path to custom CSS file, if configured |
| `nav_html` | String | Rendered navigation HTML (use with `safe` filter) |
| `content` | String | Rendered page HTML (use with `safe` filter) |
| `live_reload` | Boolean | Whether the dev server is running |
| `search_enabled` | Boolean | Whether full-text search is enabled |
| `mermaid_enabled` | Boolean | Whether Mermaid diagram rendering is enabled |
| `mermaid_version` | String | Mermaid.js major version to load from CDN |
| `color_mode` | String | Color mode: `"light"`, `"dark"`, or `"both"` |

:::note
The default template includes JavaScript for tab switching, sidebar collapse/expand, navigation filtering, popover positioning, search, and Mermaid diagram rendering. If you override the `scripts` block, you'll need to re-implement any of these features you want to keep.
:::

## Related Pages

- [[reference/css-variables|CSS Variables]] — complete list of every variable and its default value
- [[guides/configuration|Configuration]] — `docanvil.toml` and `nav.toml` reference
