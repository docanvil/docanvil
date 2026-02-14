# Markdown

DocAnvil renders Markdown using comrak with GitHub Flavored Markdown (GFM) extensions enabled. Everything you'd expect from standard Markdown works, plus tables, task lists, strikethrough, footnotes, and front matter.

## Text Formatting

**Bold text** is wrapped in double asterisks: `**bold**`

*Italic text* uses single asterisks: `*italic*`

~~Strikethrough~~ uses double tildes: `~~strikethrough~~`

You can combine them: ***bold and italic***, ~~**bold strikethrough**~~

## Headings

```markdown
# Heading 1
## Heading 2
### Heading 3
#### Heading 4
```

Heading 1 gets a colored bottom border. Heading 2 gets a subtle separator line. Headings 3 and 4 are unstyled dividers.

## Links and Images

Standard Markdown links: `[text](url)`

Images: `![alt text](image-url)`

For linking between documentation pages, use [[writing/wiki-links|wiki-links]] instead — they resolve automatically and warn on broken references.

## Lists

### Unordered

- First item
- Second item
  - Nested item
  - Another nested item
- Third item

### Ordered

1. Step one
2. Step two
3. Step three
   1. Sub-step
   2. Another sub-step

### Task Lists

- [x] Write the documentation
- [x] Add code examples
- [ ] Review and publish
- [ ] Celebrate

Task lists render as checkboxes. Use `- [x]` for checked and `- [ ]` for unchecked items.

## Tables

| Feature | Syntax | Rendered |
|:--------|:------:|----------|
| Bold | `**text**` | **text** |
| Italic | `*text*` | *text* |
| Code | `` `code` `` | `code` |
| Strikethrough | `~~text~~` | ~~text~~ |

Tables support column alignment with colons in the separator row:

```markdown
| Left | Center | Right |
|:-----|:------:|------:|
| a    |   b    |     c |
```

## Code Blocks

Inline code uses single backticks: `let x = 42;`

Fenced code blocks use triple backticks with an optional language identifier:

```rust
fn main() {
    println!("Hello from DocAnvil!");
}
```

```javascript
const greet = (name) => {
  console.log(`Hello, ${name}!`);
};
```

## Blockquotes

> Blockquotes are rendered with a colored left border and a subtle background.
>
> They can span multiple paragraphs.

Use `>` at the start of each line.

## Footnotes

DocAnvil supports footnotes[^1] using the standard syntax. Reference them inline with `[^name]` and define them anywhere in the document.

[^1]: This is a footnote. It appears at the bottom of the page in a dedicated section.

Here's another example with a longer footnote[^details].

[^details]: Footnotes can contain multiple sentences. They're collected and rendered at the bottom of the page with a horizontal separator and back-references.

## Front Matter

Pages can include YAML front matter between `---` delimiters at the top of the file. Front matter lets you set custom page titles, descriptions, author info, and dates — which DocAnvil uses for navigation labels, search, and SEO meta tags.

```markdown
---
title: My Page Title
description: A brief summary for search engines
author: Jane Doe
date: 2024-01-15
---

# Page Content
```

See [[writing/front-matter|Front Matter]] for the full list of supported fields and examples.

## Horizontal Rules

Three or more hyphens, asterisks, or underscores create a horizontal rule:

---

```markdown
---
```

## Inline Attributes

DocAnvil supports a post-processing pass for inline attributes. Place `{.class #id}` on the line immediately after an element to inject HTML attributes:

```markdown
## My Section
{#custom-id .highlighted}
```

This renders as `<h2 id="custom-id" class="highlighted">My Section</h2>`.

Supported shorthand:

| Syntax | Result |
|--------|--------|
| `.classname` | `class="classname"` |
| `#idname` | `id="idname"` |
| `key="value"` | `key="value"` |

Multiple classes can be combined: `{.first .second #my-id}`

## Related Pages

- [[writing/front-matter|Front Matter]] — page metadata, titles, and SEO meta tags
- [[writing/wiki-links|Wiki-links]] — double-bracket links and inline popovers
- [[writing/components|Components]] — notes, warnings, tabs, and code groups

:::note{title="Powered by comrak"}
DocAnvil uses comrak for Markdown rendering with `unsafe` mode enabled, so raw HTML in your Markdown is passed through to the output.
:::
