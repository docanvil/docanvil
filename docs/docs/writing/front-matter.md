---
{
  "title": "Front Matter",
  "description": "Add page metadata with JSON front matter for titles, SEO, and more"
}
---

# Front Matter

Front matter is a block of JSON metadata at the top of a Markdown file, wrapped in `---` delimiters. DocAnvil parses front matter and uses it to set page titles, generate SEO meta tags, and populate Open Graph metadata in the HTML output.

## Basic Syntax

Place a JSON block at the very start of your Markdown file:

```markdown
---
{
  "title": "Getting Started",
  "description": "Learn how to install and configure DocAnvil",
  "author": "Jane Doe",
  "date": "2024-01-15"
}
---

Your page content starts here.
```

The front matter block is stripped from the rendered output — it only affects metadata.

## Supported Fields

All fields are optional. You can include any combination of them or omit front matter entirely.

| Field | Type | Effect |
|-------|------|--------|
| `title` | String | Overrides the page title used in the browser tab, navigation sidebar, search index, breadcrumbs, and URL slug |
| `slug` | String | Overrides the URL slug directly — takes priority over the title-derived slug |
| `description` | String | Shown as a subtitle under the page title, and renders as `<meta name="description">` and `<meta property="og:description">` for search engines and link previews |
| `author` | String | Renders as `<meta name="author">` |
| `date` | String | Renders as `<meta property="article:published_time">` for search engines and social sharing |
| `edit_link` | Boolean | Set to `false` to hide the "Edit this page" link (and "Open in editor" under `docanvil serve`) on this page (see `[edit]` in [[guides/configuration\|Configuration]]) |
| `last_updated` | String or Boolean | The date the page last changed, as `"YYYY-MM-DD"`. Overrides the date from Git history. Set to `false` to hide the date on this page (see `[last_updated]` in [[guides/configuration\|Configuration]]) |
| `draft` | Boolean | Set to `true` to keep the page out of production builds while you work on it (see [Draft Pages](#draft-pages)) |
| `redirect_from` | List of strings | Old paths that should send readers to this page, so links to them keep working (see [Redirects](#redirects)) |
| `llms` | Boolean | Set to `false` to leave this page out of `llms.txt` and `llms-full.txt` (see `[llms]` in [[guides/configuration\|Configuration]]) |
| `toc` | Boolean | Set to `false` to hide the on-page table of contents, e.g. on a [[writing/components|landing page]] |

Unknown fields are silently ignored, so you can add your own custom metadata without causing errors.

## Title Override

By default, a page's title is its first `# Heading`, or, if it has none, its filename (`getting-started.md` becomes "Getting Started"). Front matter `title` overrides this everywhere:

- The `<title>` tag in the HTML head
- The navigation sidebar label
- The search index
- Breadcrumb trails
- The URL slug and output filename

```markdown
---
{
  "title": "Quick Start Guide"
}
---

# Getting Started with DocAnvil

Content here...
```

In this example, the sidebar and browser tab show "Quick Start Guide" while the page content displays its own `# Getting Started with DocAnvil` heading.

:::note
You usually don't need `title` at all — the `# Heading` already names the page everywhere. Reach for it when you want a shorter label than the heading, or a URL based on a different name. A title taken from the heading never changes the URL, so you can reword headings freely.
:::

### Clean URLs from Titles

When a `title` is set, the page's URL slug is derived from the title instead of the filename. This is especially useful for files with organizational prefixes:

| Filename | Title | Output URL |
|----------|-------|------------|
| `01-introduction.md` | `"Introduction"` | `/introduction.html` |
| `03-setup-guide.md` | `"Setup Guide"` | `/setup-guide.html` |
| `guides/01-basics.md` | `"The Basics"` | `/guides/the-basics.html` |

The directory prefix is always preserved — only the filename portion changes.

:::note{title="Index pages are exempt"}
Pages named `index.md` keep their slug regardless of the `title` field. The `index` URL is a well-known convention and is never overridden by title. Use the explicit `slug` field if you need to change it.
:::

## Slug Override

For full control over the output URL, use the `slug` field. It takes priority over both the filename and the title-derived slug.

```markdown
---
{
  "title": "Getting Started with DocAnvil",
  "slug": "quickstart"
}
---
```

This page will be written to `/quickstart.html` while still displaying "Getting Started with DocAnvil" as the page title.

The `slug` value is normalized to a URL-safe format automatically — spaces become hyphens and special characters are removed.

### Backward-Compatible Links

When a slug changes (via `title` or `slug`), wiki-links using the old filename-based slug still resolve correctly. For example, if `01-setup.md` gets the title "Setup Guide", both `01-setup` and `setup-guide` will link to the same page.

That covers links inside your docs. The old URL itself is gone, though, so if the page was already published, add the old slug to [`redirect_from`](#redirects) to keep bookmarks and links from other sites working.

## Draft Pages

Mark a page as a draft to keep working on it in the open — commit it, review it, preview it — without publishing it yet:

```markdown
---
{
  "draft": true
}
---

# New Deployment Guide

Still being written...
```

- **`docanvil serve`** shows drafts like any other page, with a **Draft** banner at the top so you don't forget.
- **`docanvil build`** leaves drafts out completely: no HTML file, and no entry in the sidebar, search, sitemap or previous/next links. The build output tells you how many it skipped.
- **`docanvil export pdf`** leaves them out too.

When the page is ready, remove `"draft": true` (or set it to `false`) and it's published with your next build.

### Links and navigation to drafts

You don't need to tidy up around a draft before building:

- A `nav.toml` entry pointing at a draft is skipped. A group whose pages are all drafts disappears until one is published.
- A wiki-link from a published page to a draft shows its text without a link, so readers never land on a missing page.
- A draft translation (say `guide.fr.md`) isn't reported as a missing translation.

If you'd rather catch links to unpublished pages, set `draft_links = "warn"` under `[build]` in your [[guides/configuration|configuration]]. Those links then print a warning, and `docanvil build --strict` fails.

### Previewing drafts

`docanvil build --drafts` includes drafts in a static build, which is handy for preview deploys of a pull request. Drafts keep their banner and get a `noindex` tag, so search engines skip them even if the preview is public.

:::note
Drafts are about publishing, not privacy. The Markdown is still in your repository, so don't use `draft` for anything that must stay secret.
:::

## Redirects

When you rename or move a page, its old URL stops working: bookmarks, search results and links from other sites end up on the 404 page. List the old paths in the page's front matter and DocAnvil leaves a redirect behind at each one:

```markdown
---
{
  "redirect_from": ["setup", "getting-started/install"]
}
---

# Installation
```

Each entry is an old slug: the page's old path from the content folder, without `.md`, just like a wiki-link. It isn't relative to the page's own folder, so if `setup.md` moved to `guides/install.md`, the entry is `"setup"`. A single string works too: `"redirect_from": "setup"`.

- **Translated pages:** put `redirect_from` on one translation (usually the default language) and every language gets its own redirect. `/en/setup.html` goes to the English page and `/fr/setup.html` to the French one.
- **Versioned sites:** the redirect stays in the page's own version. `docs/v2/guides/install.md` with `"redirect_from": ["setup"]` redirects `/v2/setup.html`.
- **Site paths:** an entry starting with `/`, like `"/old/install.html"`, is used exactly as written and points at the page in the current version and default language.

Readers land on the page straight away, at the same `#section` if the old link had one, and search engines are told the page has moved. For redirects that don't belong to a single page (deleted pages, bulk moves, links to another site) and for keeping URLs working when you turn on languages or versions, see `[redirects]` in [[guides/configuration|Configuration]].

:::note
A redirect never replaces a real page. If a path in `redirect_from` is still a page, the page stays and the build warns you.
:::

## SEO Meta Tags

When front matter fields are present, DocAnvil generates the corresponding HTML meta tags in the page `<head>`:

```html
<meta name="description" content="Learn how to install and configure DocAnvil">
<meta property="og:description" content="Learn how to install and configure DocAnvil">
<meta name="author" content="Jane Doe">
<meta property="article:published_time" content="2024-01-15">
```

Every page also gets these Open Graph tags automatically, regardless of front matter:

```html
<meta property="og:title" content="Getting Started">
<meta property="og:type" content="article">
```

## Examples

Here are a few common front-matter patterns to get you started.

### Minimal — title only

Just a title is enough to override the page's title (from its `# Heading` or filename) and set the `<title>` tag.

```markdown
---
{
  "title": "API Reference"
}
---
```

### Full metadata

Include `description`, `author`, and `date` to populate Open Graph and `<meta>` tags.

```markdown
---
{
  "title": "Deployment Guide",
  "description": "Deploy your DocAnvil site to Netlify, Vercel, or GitHub Pages",
  "author": "Documentation Team",
  "date": "2024-06-01"
}
---
```

### Custom slug

Use `slug` to control the output URL regardless of the filename.

```markdown
---
{
  "title": "Frequently Asked Questions",
  "slug": "faq"
}
---
```

This outputs to `/faq.html` instead of `/frequently-asked-questions.html`.

### No front matter

Pages without front matter work exactly as before — the title comes from the first `# Heading` (or the filename) and no extra meta tags are added.

## Date Format

The `date` field is passed through as-is to the `article:published_time` meta tag. ISO 8601 format (`YYYY-MM-DD`) is recommended for best compatibility with search engines and social platforms.

`date` is when a page was first published; `last_updated` is when it last changed. `last_updated` must be a real `YYYY-MM-DD` date (or `false`). Anything else is ignored, and `docanvil doctor` reports it as an error.
