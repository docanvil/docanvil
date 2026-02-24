---
{
  "title": "PDF Export"
}
---
# PDF Export

DocAnvil can export your entire documentation site as a single, print-ready PDF using Chrome or Chromium — no external tooling, no Python, no Pandoc.

The exported PDF includes a table of contents, all your pages in navigation order, syntax-highlighted code blocks, Mermaid diagrams, and an optional cover page. Running headers and page numbers are added automatically.

## Prerequisites

PDF export requires **Google Chrome** or **Chromium** to be installed. DocAnvil launches the browser in headless mode and uses the Chrome DevTools Protocol (CDP) to render the page and print it to PDF.

DocAnvil searches for Chrome in these locations (in order):

::::tabs
:::tab{title="macOS"}
- `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`
- `chromium` on PATH
:::
:::tab{title="Windows"}
- `%ProgramFiles%\Google\Chrome\Application\chrome.exe`
- `%ProgramFiles(x86)%\Google\Chrome\Application\chrome.exe`
:::
:::tab{title="Linux"}
- `google-chrome`, `google-chrome-stable`
- `chromium-browser`, `chromium`, `chrome`
- Any of the above on PATH
:::
::::

If Chrome isn't found, the command exits with a clear error message.

## Basic Usage

```bash
docanvil export pdf --out guide.pdf
```

| Option | Required | Default | Description |
|--------|----------|---------|-------------|
| `--out` | Yes | — | Output path for the PDF file |
| `--path` | No | `.` | Path to the project root |
| `--locale` | No | project default | Locale to export. Pass `all` to generate one PDF per enabled locale. |

Parent directories for the output path are created automatically.

:::code-group
```bash
# Export for the current project
docanvil export pdf --out guide.pdf
```

```bash
# Export a project in another directory
docanvil export pdf --out guide.pdf --path ../my-docs
```

```bash
# Suppress progress output (useful in scripts)
docanvil export pdf --out guide.pdf --quiet
```
:::

## Configuration

PDF export is configured in the `[pdf]` section of `docanvil.toml`:

```toml
[pdf]
author = "Your Name"
cover_page = true
paper_size = "A4"
custom_css = "theme/pdf.css"
```

| Key | Default | Description |
|-----|---------|-------------|
| `author` | `None` | Author name shown on the cover page and in the running page header |
| `cover_page` | `false` | Prepend a cover page with the project title and author name |
| `paper_size` | `"A4"` | Paper size — see [Custom Paper Size](#custom-paper-size) for supported values |
| `custom_css` | `None` | Path (relative to the project root) to a CSS file injected into the PDF |

## Cover Page

When `cover_page = true`, a title page is added before the table of contents. It shows the project name as a large centered title with the author below it.

```toml
[pdf]
cover_page = true
author = "The DocAnvil Team"
```

## Custom Paper Size

```toml
[pdf]
paper_size = "Letter"
```

Supported sizes (case-insensitive):

| Size | Dimensions |
|------|------------|
| `A3` | 297 × 420 mm (11.69 × 16.54 in) |
| `A4` | 210 × 297 mm (8.27 × 11.69 in) — **default** |
| `A5` | 148 × 210 mm (5.83 × 8.27 in) |
| `Letter` | 8.5 × 11 in |
| `Legal` | 8.5 × 14 in |
| `Tabloid` | 11 × 17 in |

Unrecognised values fall back to A4 silently.

## Per-Locale Export

If your project uses [[guides/localisation|localisation]], you can generate a separate PDF for each enabled locale in a single command:

```bash
# One PDF per locale — locale code inserted before the extension
docanvil export pdf --out guide.pdf --locale all
# → guide.en.pdf, guide.fr.pdf, guide.de.pdf …
```

You can also export a single specific locale:

```bash
docanvil export pdf --out guide-fr.pdf --locale fr
```

Each PDF uses the locale's own navigation order and page content. Pages without a translation for that locale are silently skipped.

:::note
`--locale all` requires i18n to be configured (`[locale]` with both `default` and `enabled` set in `docanvil.toml`). Running it on a non-i18n project returns a clear error.
:::

## RTL Language Support

Right-to-left locales are detected automatically. When you export in an RTL locale, Chrome lays out the entire PDF right-to-left — no configuration required.

```bash
docanvil export pdf --out guide-ar.pdf --locale ar
```

Supported RTL locale codes: `ar` (Arabic), `he` (Hebrew), `ur` (Urdu), `fa` (Persian/Farsi), `ug` (Uyghur).

## Custom PDF CSS

For fine-grained control over the PDF's appearance, provide a CSS file:

```toml
[pdf]
custom_css = "theme/pdf.css"
```

The file is injected after the default PDF styles, so any rule you write overrides the defaults. Some useful targets:

```css
/* Swap the typeface */
body {
  font-family: "Source Serif 4", serif;
  font-size: 10.5pt;
}

/* Tighter code blocks */
pre {
  font-size: 8.5pt;
}

/* Remove link colouring in print */
a {
  color: inherit;
  text-decoration: none;
}

/* Wider margins */
@page {
  margin: 3cm;
}
```

## Mermaid Diagrams

`:::mermaid` blocks are rendered to SVG before the PDF is captured. DocAnvil waits up to 15 seconds for all diagrams to finish. If any are still pending after the timeout, the PDF is generated with whatever has rendered.

Mermaid.js is loaded from CDN, so Mermaid rendering requires an internet connection. In offline CI environments, disable charts:

```toml
[charts]
enabled = false
```

## Running Headers and Footers

Every page gets:

- **Header left** — project name
- **Header right** — author name (if `author` is configured)
- **Footer right** — page number

These are injected by Chrome's print engine and are not affected by custom CSS.

## Tips

- Run `docanvil build` first to confirm your content is error-free. Broken wiki-links and rendering issues will appear in the PDF just as they do in the HTML site.
- The PDF follows your `nav.toml` order exactly — the table of contents and chapter sequence match what readers see online.
- Use `--quiet` to suppress progress output in automated scripts.

## Related Pages

- [[guides/localisation|Localisation]] — setting up multi-language docs
- [[guides/configuration|Configuration]] — full `docanvil.toml` reference including `[pdf]`
- [[reference/cli|CLI Commands]] — all subcommands and flags
