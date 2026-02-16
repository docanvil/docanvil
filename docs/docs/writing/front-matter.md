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
| `title` | String | Overrides the page title used in the browser tab, navigation sidebar, search index, and breadcrumbs |
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

### No front matter

Pages without front matter work exactly as before — the title is derived from the filename and no extra meta tags are added.

## Date Format

The `date` field is passed through as-is to the `article:published_time` meta tag. ISO 8601 format (`YYYY-MM-DD`) is recommended for best compatibility with search engines and social platforms.
