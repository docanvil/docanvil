---
{
  "title": "Links & Popovers"
}
---

# Links & Popovers

DocAnvil provides two special inline syntaxes beyond standard Markdown: **wiki-links** for linking between pages and **popovers** for inline tooltips.

## Wiki-Links

### Basic Syntax

Link to another page using double brackets:

<pre><code>See &#91;[guides/getting-started]] for installation steps.</code></pre>

This resolves to an HTML link pointing at the target page. The display text defaults to the link target.

### Custom Display Text

Use a pipe to set custom link text:

<pre><code>Check the &#91;[guides/getting-started|installation guide]] to get started.</code></pre>

### Live Examples

Here are working wiki-links to pages in this documentation:

- [[index]] — the home page
- [[guides/configuration|Configuration guide]] — customizing your project
- [[reference/cli|CLI reference]] — all commands and flags
- [[writing/markdown|Markdown features]] — supported syntax

### Resolution Rules

DocAnvil resolves wiki-links against the page inventory in two steps:

1. **Exact match** — the target is compared to page slugs directly (`guides/getting-started` matches `guides/getting-started`)
2. **Basename match** — if no exact match, the last path component is tried (`getting-started` matches `guides/getting-started`)

Basename matching means you can use short names when the page name is unique:

<pre><code>&#91;[getting-started]]     resolves to → guides/getting-started
&#91;[configuration]]       resolves to → guides/configuration</code></pre>

### Slug Derivation

Slugs are derived from the file path relative to the content directory, with the `.md` extension removed:

| File Path | Slug |
|-----------|------|
| `docs/index.md` | `index` |
| `docs/guides/getting-started.md` | `guides/getting-started` |
| `docs/reference/cli.md` | `reference/cli` |

### Broken Links

When a wiki-link target doesn't match any page, it renders as a red highlighted span with an error popover. Here is a deliberate example:

<p>
    <span class="broken-link popover-trigger" tabindex="0">nonexistent-page
        <span class="popover-content popover-error" role="tooltip">
        <strong>Page not found</strong><br>
        The linked page doesn't exist: <code>nonexistent-page</code>
        </span>
    </span>
</p>

The build process also logs a warning for each broken link, so you can find and fix them.

:::warning{title="Broken links are visible"}
Broken wiki-links are styled in red with a dashed underline and an error tooltip. They're easy to spot both in the browser and in build output.
:::

## Popovers

Popovers add inline tooltip content that appears on hover or focus.

### Syntax

Use `^[content]` to create a popover:

```markdown
DocAnvil uses comrak^[A fast, GFM-compatible Markdown parser written in Rust] for rendering.
```

### Live Examples

DocAnvil uses comrak^[A fast, GFM-compatible Markdown parser written in Rust] for rendering Markdown.

The default theme uses an indigo^[Specifically #6366f1, a balanced purple-blue] accent color.

Popovers appear above the trigger text by default, but flip below^[This automatic repositioning prevents popovers from being clipped at the top of the viewport] when near the top of the viewport.

### Behavior

- Popovers appear on **hover** and **keyboard focus**
- They automatically reposition to avoid overflowing viewport edges
- Content inside backticks (`` ` ``) and fenced code blocks is not processed
- HTML in popover content is escaped for safety

:::note{title="Accessibility"}
Each popover uses `role="tooltip"` and `aria-describedby` to connect the trigger to its content, making them accessible to screen readers. The trigger element has `tabindex="0"` for keyboard navigation.
:::

### Skipped in Code

Popover syntax inside inline code (`^[like this]`) and fenced code blocks is left as-is:

```
This ^[popover syntax] is not processed inside code blocks.
```

## Related Pages

- [[writing/markdown|Markdown]] — all supported Markdown and GFM features
- [[writing/components|Components]] — directive blocks for richer content
