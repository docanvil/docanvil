---
{
  "description": "Set up your project with docanvil.toml and shape the sidebar with nav.toml."
}
---

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

[syntax]
enabled = true
theme = "base16-ocean.dark"
line_numbers = false

[search]
enabled = true

[charts]
enabled = true
mermaid_version = "11"

[locale]
default = "en"
enabled = ["en", "fr"]
auto_detect = true

[locale.display_names]
en = "English"
fr = "Français"

[locale.flags]
en = "🇺🇸"

[version]
current = "v2"
enabled = ["v1", "v2"]

[version.display_names]
v1 = "v1.0"
v2 = "v2.0 (latest)"

[pdf]
cover_page = true
author = "Your Name"
paper_size = "A4"

[doctor]
max_paragraph_words = 150
heading_adjacent_separator = true

[edit]
repo = "https://github.com/org/repo"
branch = "main"

[last_updated]
enabled = true

[redirects]
"old-faq" = "help/faq"
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
| `site_url` | `None` | Full site URL (e.g. `"https://example.com/"`) for canonical URLs, hreflang tags, and sitemap |
| `draft_links` | `"text"` | How wiki-links to [[writing/front-matter|draft pages]] render in builds that leave drafts out. `"text"` shows the link text quietly; `"warn"` also prints a warning, so `--strict` fails |

:::note{title="Recommended for i18n"}
Setting `site_url` is strongly recommended when using localisation. It enables absolute hreflang URLs, canonical `<link>` tags, and `og:url` meta tags — all important for multilingual SEO.
:::

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

### `[syntax]` Section

| Key | Default | Description |
|-----|---------|-------------|
| `enabled` | `true` | Highlight fenced code blocks at build time |
| `theme` | `"base16-ocean.dark"` | Highlighting theme |
| `line_numbers` | `false` | Number the lines of every code block |

A single block can still opt in or out with `numbers` or `numbers="false"`. See [[writing/code-blocks-from-files|Code Blocks from Files]] for line numbers, captions and showing code straight from source files.

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

### `[locale]` Section

| Key | Default | Description |
|-----|---------|-------------|
| `default` | `None` | Default locale code (e.g. `"en"`). Required to enable i18n. |
| `enabled` | `[]` | List of enabled locale codes (e.g. `["en", "fr", "de"]`) |
| `auto_detect` | `true` | Auto-detect the user's browser language and redirect on first visit |
| `display_names` | `{}` | Human-readable names for locales shown in the language switcher |
| `flags` | `{}` | Flag emoji overrides for locales (e.g. `{"en": "🇺🇸"}` to use US flag instead of default 🇬🇧) |

When both `default` and `enabled` are set, DocAnvil switches to multi-language mode: each locale gets its own URL prefix (`/en/`, `/fr/`), its own navigation and search index, and a language switcher appears in the header.

```toml
[locale]
default = "en"
enabled = ["en", "fr", "de"]
auto_detect = true

[locale.display_names]
en = "English"
fr = "Français"
de = "Deutsch"

[locale.flags]
en = "🇺🇸"    # Use US flag instead of default GB
```

:::note{title="Need details?"}
See [[guides/localisation|Localisation]] for a complete walkthrough of setting up multi-language docs, including file naming, per-locale navigation, and translation coverage.
:::

### `[version]` Section

| Key | Default | Description |
|-----|---------|-------------|
| `current` | *(last in `enabled`)* | The current/latest version code — used for the root redirect and the older-version banner. Defaults to the last entry in `enabled` when not set. |
| `enabled` | `[]` | List of version directory names to build (e.g. `["v1", "v2"]`). Each must have a matching subdirectory in your content directory. |
| `display_names` | `{}` | Human-readable names shown in the version switcher (e.g. `{"v2": "v2.0 (latest)"}`) |

When `enabled` is non-empty, DocAnvil switches to multi-version mode: each version gets its own URL prefix (`/v1/`, `/v2/`), its own navigation and search index, and a version switcher appears in the header. Pages in older versions automatically show a banner linking to the latest version.

```toml
[version]
current = "v2"
enabled = ["v1", "v2"]

[version.display_names]
v1 = "v1.0"
v2 = "v2.0 (latest)"
```

:::note{title="Need details?"}
See [[guides/versioning|Versioning]] for a complete walkthrough of setting up multi-version docs, including directory layout, per-version navigation, the version switcher, and composing versioning with i18n.
:::

### `[pdf]` Section

| Key | Default | Description |
|-----|---------|-------------|
| `author` | `None` | Author name shown on the cover page and in the running page header |
| `cover_page` | `false` | Prepend a cover page with the project title and author before the table of contents |
| `paper_size` | `"A4"` | Paper size: `"A3"`, `"A4"`, `"A5"`, `"Letter"`, `"Legal"`, `"Tabloid"` (case-insensitive) |
| `custom_css` | `None` | Path (relative to project root) to a CSS file injected into the PDF output |

:::note{title="Need details?"}
See [[guides/pdf-export|PDF Export]] for the full guide: cover pages, paper sizes, RTL support, per-locale export, and custom CSS.
:::

### `[doctor]` Section

| Key | Default | Description |
|-----|---------|-------------|
| `max_paragraph_words` | `150` | Word count threshold for the `long-paragraph` readability check. Use `0` to disable the check entirely. |
| `heading_adjacent_separator` | `true` | Warn when a heading is directly adjacent to a horizontal rule. Set `false` to opt out. |

The `[doctor]` section configures the `docanvil doctor` readability linter. The default settings are intentionally permissive — tighten the threshold for higher-quality writing standards.

```toml
[doctor]
max_paragraph_words = 100          # Flag paragraphs over 100 words
heading_adjacent_separator = false # Disable the separator-next-to-heading check
```

:::note{title="Need details?"}
See [[reference/cli|CLI Commands → Readability checks]] for the full list of checks, their severities, and what each one catches.
:::

### `[edit]` Section

Adds an "Edit this page" link to every page, pointing at its Markdown source on GitHub, GitLab or Bitbucket. Readers can suggest a fix in their browser, and you get it as a pull request (or merge request) to review like any other change.

| Key | Default | Description |
|-----|---------|-------------|
| `repo` | `None` | Your repository's web address, e.g. `"https://github.com/org/repo"`. Setting it turns edit links on |
| `branch` | `"main"` | The branch edits are made against |
| `provider` | inferred | `"github"`, `"gitlab"` or `"bitbucket"`. Only needed for self-hosted instances; `github.com`, `gitlab.com` and `bitbucket.org` are detected automatically |
| `root` | auto-detected | Where your DocAnvil project lives inside the repository, e.g. `"docs"`. Detected by looking for the nearest `.git` folder, so you only need it when building outside a Git checkout |

```toml
[edit]
repo = "https://github.com/org/repo"
branch = "main"
```

Links always point at the file the page was built from, so translated pages (`page.fr.md`) and older versions (`docs/v1/page.md`) link to their own source. To hide the link on a single page, set `"edit_link": false` in its [[writing/front-matter|front matter]]. Edit links don't appear on the 404 page or in PDF exports.

:::note{title="Self-hosted Git"}
GitHub Enterprise and self-hosted GitLab work by setting `provider`. Bitbucket Server and Data Center aren't supported yet, since they don't offer a direct edit link. `docanvil doctor` warns if your `[edit]` settings can't produce working links.
:::

### `[last_updated]` Section

Shows readers when each page last changed, so they can tell at a glance that the docs are being looked after. The date comes from your Git history, and appears under the page next to "Edit this page".

| Key | Default | Description |
|-----|---------|-------------|
| `enabled` | `false` | Show a "Last updated" date on every page |
| `source` | `"git"` | Where dates come from: `"git"` uses your commit history, `"front-matter"` only uses dates you set yourself and never runs Git |

```toml
[last_updated]
enabled = true
```

With `source = "git"`, a page's date is the most recent commit that changed it, or that changed anything it pulls in with `:::include` or `file="…"`. Update a shared install-steps fragment and every page that includes it moves forward too. DocAnvil uses the commit's author date, so rebasing or squash-merging a pull request doesn't make every page look edited on merge day.

To set a page's date by hand, use `last_updated` in its [[writing/front-matter|front matter]]:

```markdown
---
{
  "last_updated": "2026-10-09"
}
---
```

Set it to `false` to hide the date on that page.

Dates are shown in the page's language ("9 October 2026", "9 octobre 2026"), and also go into the page's `article:modified_time` meta tag and the `<lastmod>` entries in `sitemap.xml`, which helps search engines spot fresh content. Translated pages and older versions each get the date of their own source file.

:::warning{title="Fetch full history in CI"}
Most CI systems clone only the latest commit by default. With that shallow history every page would show the same date, so DocAnvil warns about it and `--strict` builds fail. Fetch the full history instead: `fetch-depth: 0` on GitHub's `actions/checkout`, or `clone: depth: full` on Bitbucket Pipelines. See [[deployment/github-pages|GitHub Pages]] and [[deployment/bitbucket-pipelines|Bitbucket Pipelines]].
:::

A few things to know:

- **Renamed or moved files** show the date of the commit that moved them.
- **Uncommitted changes** don't count: a page shows its last committed date until you commit.
- **Files inside a Git submodule** don't contribute a date.
- **No Git repository?** DocAnvil warns and falls back to front matter dates. Set `source = "front-matter"` if that's what you want, and the warning goes away.

`docanvil doctor` warns about shallow clones and missing repositories, and flags any front matter `last_updated` that isn't a valid date.

### `[redirects]` Section

Keeps old URLs working when pages are renamed, moved or deleted, so bookmarks, search results and links from other sites still land somewhere useful. Each entry maps an old path to where it goes now:

| Key | Default | Description |
|-----|---------|-------------|
| `"old/path"` | — | As many entries as you need: the old path on the left, the new one on the right |
| `unprefixed` | `false` | Redirect each `page.html` to the current version and default language (see below) |

```toml
[redirects]
"old-faq" = "help/faq"
"getting-started/install" = "guides/install"
"/blog/" = "https://blog.example.com"
```

Paths can be written three ways:

- **A page slug**, like `help/faq`: the same thing you'd write in a [[writing/wiki-links|wiki-link]]. A trailing `.html` or `/` is fine. On translated or versioned sites the redirect is written for every language and version where the target page exists, so `"old-faq" = "help/faq"` gives you `/en/old-faq.html`, `/fr/old-faq.html` and so on.
- **A site path** starting with `/`, like `/old-blog.html`, or `/blog/` for `/blog/index.html`. It's used exactly as written, once.
- **Another site**: a target containing `://`, like `https://blog.example.com`. Only on the right-hand side.

Each old URL becomes a small page that sends readers straight on, keeping any `#section` from the link. It also tells search engines where the page went, so its ranking moves to the new URL. It works on any static host, with nothing to set up on the server, and redirect pages never show up in the sidebar, search or `sitemap.xml`.

For a page you've renamed or moved, it's usually simpler to list the old path in the page's own [[writing/front-matter|front matter]] with `redirect_from`. It travels with the page and covers every translation. The `[redirects]` table suits pages you've deleted, or moving lots of pages at once. If both send the same old path to different places, the front matter wins.

**Turning on languages or versions** moves every page: `/guides/install.html` becomes `/en/guides/install.html` or `/v2/guides/install.html`. Set `unprefixed = true` and DocAnvil leaves a redirect at each old address, pointing at the current version in the default language:

```toml
[redirects]
unprefixed = true
```

:::note
`unprefixed` shares the table with your paths, so a page whose slug is literally `unprefixed` needs the site-path form: `"/unprefixed.html" = "…"`.
:::

A redirect to a page that doesn't exist, one whose old path is still a real page (the page always wins), two redirects sending the same path to different places, and redirects that go round in a loop each print a build warning, so `docanvil build --strict` fails. `docanvil doctor` checks every redirect up front and points at the line in `docanvil.toml` or the page's front matter. A redirect to a [[writing/front-matter|draft page]] is skipped quietly until the page is published.

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

By default, the sidebar label is the page's title — its first `# Heading`, or the slug if it has none (`getting-started` becomes "Getting Started"). Override it with `label`:

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
- [[guides/pdf-export|PDF Export]] — full PDF export guide
- [[guides/versioning|Versioning]] — multi-version docs setup and version switcher
- [[reference/project-structure|Project Structure]] — how files map to pages and slugs

:::note
The header includes a filter input that searches page labels in real time. This works with any navigation structure.
:::
