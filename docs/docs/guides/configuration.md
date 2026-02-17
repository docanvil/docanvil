# Configuration

DocAnvil uses two configuration files at the root of your project: `docanvil.toml` for project settings and `nav.toml` for navigation structure.

## docanvil.toml

::::tabs
:::tab{title="Minimal"}
```toml
[project]
name = "My Docs"
```
:::
:::tab{title="Full"}
```toml
[project]
name = "My Docs"
content_dir = "docs"

[build]
output_dir = "dist"
base_url = "/my-project/"

[theme]
custom_css = "theme/custom.css"
color_mode = "both"

[theme.variables]
color-primary = "#059669"
font-body = "Georgia, serif"

[search]
enabled = true

[charts]
enabled = true
mermaid_version = "11"
```
:::
::::

:::warning{title="Required field"}
The `name` field under `[project]` is required. DocAnvil will fail to load without it.
:::

### `[project]` Section

| Key | Default | Description |
|-----|---------|-------------|
| `name` | *(required)* | Project name displayed in the sidebar and page titles |
| `content_dir` | `"docs"` | Directory containing your Markdown files |

### `[build]` Section

| Key | Default | Description |
|-----|---------|-------------|
| `output_dir` | `"dist"` | Directory where the static site is generated |
| `base_url` | `"/"` | URL path prefix for subfolder deployments (e.g. `"/my-project/"`) |

### `[theme]` Section

| Key | Default | Description |
|-----|---------|-------------|
| `name` | `None` | Reserved for future theme selection |
| `custom_css` | `None` | Path to a custom CSS file loaded after the default theme |
| `color_mode` | `"light"` | Color mode: `"light"`, `"dark"`, or `"both"` (light + dark with toggle) |
| `variables` | `{}` | CSS variable overrides injected as `:root` properties |

Variables are specified as key-value pairs where the key is the CSS variable name (without `--`) and the value is any valid CSS value:

```toml
[theme.variables]
color-primary = "#059669"
color-bg = "#fafafa"
font-body = "Inter, sans-serif"
content-max-width = "960px"
```

See [[reference/css-variables|CSS Variables]] for the complete list of available variables.

### `[search]` Section

| Key | Default | Description |
|-----|---------|-------------|
| `enabled` | `true` | Enable or disable full-text search |

When enabled, DocAnvil generates a `search-index.json` file at build time and adds a search input to the header. Search is powered by MiniSearch.js, loaded from a CDN on first use. Set `enabled = false` to remove the search UI and skip index generation.

### `[charts]` Section

| Key | Default | Description |
|-----|---------|-------------|
| `enabled` | `true` | Enable or disable Mermaid diagram rendering |
| `mermaid_version` | `"11"` | Major version of Mermaid.js to load from CDN |

When enabled, pages containing `:::mermaid` blocks will load Mermaid.js and render diagrams client-side. When disabled, `:::mermaid` content renders as preformatted text.

## nav.toml

The navigation file controls the sidebar structure. It uses TOML's array-of-tables syntax and supports pages, separators, and groups.

### Page Entries

The simplest entry links to a page by its slug (the file path relative to `content_dir`, without the `.md` extension):

<pre><code class="language-toml">&#91;[nav]]
page = "index"

&#91;[nav]]
page = "guides/getting-started"
</code></pre>

### Label Overrides

By default, the sidebar label is derived from the slug (`getting-started` becomes "Getting Started"). Override it with `label`:

<pre><code class="language-toml">&#91;[nav]]
page = "guides/getting-started"
label = "Installation"
</code></pre>

### Separators

Add visual dividers between sections. A labeled separator shows text:

<pre><code class="language-toml">&#91;[nav]]
separator = "Guides"
</code></pre>

An unlabeled separator draws a horizontal line:

<pre><code class="language-toml">&#91;[nav]]
separator = true
</code></pre>

### Groups

Groups create collapsible sections in the sidebar. Each group has a `label` and an array of children in `group`:

<pre><code class="language-toml">&#91;[nav]]
label = "Reference"
group = [
  { page = "reference/cli", label = "CLI Commands" },
  { page = "reference/project-structure" },
  { page = "reference/css-variables", label = "CSS Variables" },
]
</code></pre>

### Linked Group Headers

Add a `page` field to make the group header itself a clickable link:

<pre><code class="language-toml">&#91;[nav]]
label = "Writing Content"
page = "writing/markdown"
group = [
  { page = "writing/wiki-links", label = "Links &amp; Popovers" },
  { page = "writing/components" },
]
</code></pre>

Clicking "Writing Content" navigates to the Markdown page, while the arrow expands the group.

### Child Separators

You can add separators inside groups to organize children:

<pre><code class="language-toml">&#91;[nav]]
label = "Reference"
group = [
  { page = "reference/cli", label = "CLI Commands" },
  { separator = "Project" },
  { page = "reference/project-structure" },
  { page = "reference/css-variables", label = "CSS Variables" },
]
</code></pre>

### Autodiscover

You can use the autodiscover option to selectively autodiscover a folder and add it to the navigation:

<pre><code class="language-toml">&#91;[nav]]
autodiscover = "api"
</code></pre>

You can also use autodiscover with a collapsible group:

<pre><code class="language-toml">&#91;[nav]]
label = "Reference"
autodiscover = "reference"
</code></pre>

### Auto-Discovery Fallback

If `nav.toml` is absent, DocAnvil auto-discovers all `.md` files in the content directory and builds the navigation from the directory structure. Files are sorted alphabetically, and directory names become group labels.

## Related Pages

- [[guides/theming|Theming]] — CSS variables, custom CSS, and template overrides
- [[reference/project-structure|Project Structure]] — how files map to pages and slugs

:::note
The header includes a filter input that searches page labels in real time. This works with any navigation structure.
:::
