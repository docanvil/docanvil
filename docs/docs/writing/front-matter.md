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
| `description` | String | Renders as `<meta name="description">` and `<meta property="og:description">` for search engines and link previews |
| `author` | String | Renders as `<meta name="author">` |
| `date` | String | Renders as `<meta property="article:published_time">` for search engines and social sharing |

Unknown fields are silently ignored, so you can add your own custom metadata without causing errors.

## Title Override

By default, DocAnvil derives page titles from filenames — `getting-started.md` becomes "Getting Started". Front matter `title` overrides this everywhere:

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

### Minimal — title only

```markdown
---
{
  "title": "API Reference"
}
---
```

### Full metadata

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

Pages without front matter work exactly as before — the title is derived from the filename and no extra meta tags are added.

## Date Format

The `date` field is passed through as-is to the `article:published_time` meta tag. ISO 8601 format (`YYYY-MM-DD`) is recommended for best compatibility with search engines and social platforms.
