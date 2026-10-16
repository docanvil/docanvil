---
{
  "title": "Includes",
  "description": "Reuse Markdown across pages, languages and versions with :::include"
}
---

# Includes

Write a piece of content once and use it on as many pages as you like. Install steps, a warning that applies to several guides, a table of CLI flags — put it in a fragment file and include it wherever it belongs. Fix it in one place and every page picks up the change.

## Fragments

A fragment is any Markdown file whose name — or the name of a folder in its path — starts with `_`. DocAnvil never builds fragments as pages, so they stay out of the sidebar, search and sitemap:

```text
docs/
  _shared/
    install.md        ← fragment
  guides/
    _flags.md         ← fragment
    setup.md          ← page
```

## Including a fragment

Put `:::include` on a line of its own. Includes inside blockquotes (`> :::include{…}`) are not supported: move the include outside the quote, or put the blockquote inside the included fragment.

For example:

```markdown
:::include{file="_shared/install.md"}
```

The fragment's Markdown is dropped in at that spot before anything else is rendered, so its headings show up in the table of contents, wiki-links resolve, components render and search indexes the text — exactly as if you'd written it into the page.

### Where paths point

:::include{file="../_shared/paths.md"}

A few more rules:

- **Front matter** at the top of a fragment is ignored.
- **Indentation carries over**: indent the `:::include` line inside a list item and the whole fragment is indented to match.
- **Fragments can include fragments.** Nested paths are relative to the fragment that contains them. If files end up including each other in a loop, DocAnvil stops and tells you which ones.
- **Any `.md` file works** — a fragment, another page, or a README outside the content folder (`file="../README.md"`).
- `:::include` must be alone on its line. Written inside a sentence, it's shown as plain text, and `docanvil doctor` points it out.

## Including a section

Include a heading and its section from any Markdown file:

```markdown
:::include{file="../README.md" section="installation"}
```

`section` is the heading's anchor ID (an optional leading `#` is accepted). The selected heading is included, along with its subsections, until the next heading of equal or higher level. Both `## Heading` and underlined Markdown headings work; headings inside code blocks, lists and blockquotes do not define sections.

Use the generated ID, such as `getting-started` for `## Getting Started`, or give the source heading a stable custom ID with `## Installation {#installation}`. Repeated headings use suffixes such as `installation-1`, just like page anchors.

Only the selected section's includes are expanded. Nested paths remain relative to the source file, and diagnostics retain their original line numbers. Locale fallback works as usual; use the same custom ID in each translation. A missing or empty section produces an include error.

## Translations and versions

When localisation is on, DocAnvil looks for a translated fragment first. A French page including `_shared/install.md` gets `_shared/install.fr.md` if it exists, and the untranslated file otherwise — so translate the fragments that need it and share the rest.

Versions need nothing special: a fragment inside `docs/v2/` is found relative to the v2 pages, and a path starting with `/` points at the same file for every version.

## When something's wrong

A missing file, a loop or an unknown attribute shows up as an error box where the content should be, plus a build warning with the file and line. `docanvil build --strict` fails on it, just like a broken link. `docanvil doctor` checks every include up front, and also lists fragments nothing includes and translated fragments that are missing a language.

## Related Pages

- [[writing/components/code-blocks|Code Blocks]]
- [[reference/project-structure|Project Structure]]
