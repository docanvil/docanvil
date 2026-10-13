---
{
  "description": "Notes, warnings, tabs, code groups, diagrams and your own template components, all from ::: directives."
}
---

# Components

DocAnvil provides built-in components rendered via fenced directives. These are processed before Markdown rendering, so you can use Markdown inside them.

## Directive Syntax

Components use a fenced block syntax with triple colons:

```markdown
:::name{attributes}
Content goes here. **Markdown** is supported.
:::
```

The opening fence is `:::` followed by the component name and optional attributes in curly braces. The closing fence is `:::` on its own line.

### Attributes

Attributes are specified inside `{...}` after the component name:

| Syntax | Result |
|--------|--------|
| `key="value"` | Named attribute |
| `.classname` | Adds a CSS class |
| `#idname` | Sets the element ID |

Multiple attributes can be combined: `:::note{title="Important" .custom-class #my-note}`

## Note

Display informational callouts with a blue/indigo theme.

:::note
This is a note with the default title.
:::

:::note{title="Custom Title"}
Notes accept a `title` attribute. The default title is "Note".
:::

Raw syntax:

```markdown
:::note{title="Custom Title"}
Your content here. Supports **Markdown**.
:::
```

## Warning

Display cautionary messages with an orange theme.

:::warning
This is a warning with the default title.
:::

:::warning{title="Danger Zone"}
Warnings accept a `title` attribute. The default title is "Warning".
:::

Raw syntax:

```markdown
:::warning{title="Danger Zone"}
Your warning content here.
:::
```

## Lozenge

Display a quick visual status with a lozenge.

Syntax is as follow: `:::lozenge{type="default",text="Default"}`

| Syntax | Result |
|--------|--------|
| :::lozenge{type="default",text="Default"} | Default |
| :::lozenge{type="warning",text="Warning"} | Warning |
| :::lozenge{type="in-progress",text="In Progress"} | In Progress |
| :::lozenge{type="error",text="Error"} | Error |
| :::lozenge{type="success",text="Success"} | Success |

## Tabs

Group content into switchable tabs. Each tab is defined with a nested `:::tab` directive. The outer `::::tabs` uses four colons so the inner `:::tab` closings (three colons) don't end the container prematurely:

::::tabs
:::tab{title="JavaScript"}
```javascript
console.log("Hello!");
```
:::
:::tab{title="Python"}
```python
print("Hello!")
```
:::
:::tab{title="Rust"}
```rust
fn main() {
    println!("Hello!");
}
```
:::
::::

Raw syntax:

```markdown
::::tabs
:::tab{title="JavaScript"}
Content for the JavaScript tab.
:::
:::tab{title="Python"}
Content for the Python tab.
:::
::::
```

If no `title` is provided, tabs are labeled "Tab 1", "Tab 2", etc.

## Code Blocks and Groups

Use fenced code blocks to show syntax-highlighted examples:

```rust
println!("Hello, DocAnvil!");
```

Wrap several blocks in `:::code-group` to give each language its own tab. See [[writing/components/code-blocks|Code Blocks]] for the full syntax, rendered code groups, captions, line numbers, highlighting, diffs and local or remote sources.

## Mermaid Diagrams

Render diagrams and charts using Mermaid.js. The content inside a `:::mermaid` block is passed directly to Mermaid — it is not processed as Markdown.

:::mermaid
graph TD
    A[Write Markdown] --> B[Run docanvil build]
    B --> C[Static HTML site]
    C --> D[Deploy anywhere]
:::

Raw syntax:

````markdown
:::mermaid
graph TD
    A[Write Markdown] --> B[Run docanvil build]
    B --> C[Static HTML site]
    C --> D[Deploy anywhere]
:::
````

Mermaid supports many diagram types including flowcharts, sequence diagrams, class diagrams, state diagrams, Gantt charts, and more. See the [Mermaid documentation](https://mermaid.js.org/) for the full syntax reference.

:::note{title="Configuration"}
Mermaid is enabled by default. Disable it by setting `enabled = false` under `[charts]` in `docanvil.toml`. When disabled, `:::mermaid` blocks render as preformatted text. See [[guides/configuration|Configuration]] for details.
:::

## Landing Pages {#landing-pages}

Four components turn a home page into a proper landing page: a hero, a row of call-to-action buttons and a grid of feature tiles. They're ordinary built-ins, so you can restyle or replace them like any other (see [Restyling Built-ins](#restyling-built-ins)).

### Hero

`::::hero` opens the page with a large title, an optional label above it (`eyebrow`) and a lead paragraph from its body. Put a `:::buttons` row inside it for your calls to action — use four colons on the hero so the inner `:::` doesn't close it early:

````markdown
::::hero{eyebrow="Markdown-powered documentation" title="Beautiful docs. Built fast."}
Turn Markdown into fast, searchable documentation sites.

:::buttons
[[guides/getting-started|Get started →]] [View on GitHub ↗](https://github.com/docanvil/docanvil)
:::
::::
````

The hero's title is the page's `<h1>`, so leave out a `# Heading` of your own. The browser tab and search then use the [[writing/front-matter|front matter]] `title`, or the file name (`index` becomes "Home"). A page that opens with a hero doesn't repeat its front matter `description` under it; the description still goes into the page's meta tags.

### Buttons

`:::buttons` turns every link in its body into a button. The first one is the primary call to action; the rest are secondary. Use [[writing/links-popovers|wiki-links]] for your own pages, so the buttons follow the reader's language and version and broken links are still reported:

:::buttons
[[guides/getting-started|Get started →]] [[writing/components|Components]]
:::

````markdown
:::buttons
[[guides/getting-started|Get started →]] [[writing/components|Components]]
:::
````

Buttons work anywhere, not just inside a hero.

### Feature Tiles

`::::features` lays out `:::feature` tiles in a grid that adapts to the screen width. Each tile takes an optional `icon` (a Lucide name such as `lucide:zap`, or an emoji) and `title`, and its body is Markdown, so it can link onwards:

::::features
:::feature{icon="lucide:zap" title="Fast static builds"}
Plain HTML you can deploy anywhere.
:::
:::feature{icon="lucide:search" title="Full-text search"}
Ready out of the box.
:::
:::feature{icon="lucide:palette" title="Custom themes"}
See [[guides/theming|Theming]].
:::
::::

````markdown
::::features
:::feature{icon="lucide:zap" title="Fast static builds"}
Plain HTML you can deploy anywhere.
:::
:::feature{icon="lucide:search" title="Full-text search"}
Ready out of the box.
:::
:::feature{icon="lucide:palette" title="Custom themes"}
See [[guides/theming|Theming]].
:::
::::
````

:::note{title="Hiding the table of contents"}
A landing page rarely needs the on-page table of contents. Turn it off for one page with `"toc": false` in its [[writing/front-matter|front matter]].
:::

## More Layout Components

Use these components to guide readers through tutorials, highlight related pages, and arrange supporting content. They work with Markdown and need no additional JavaScript.

### Link Cards

Use `::::cards` to group clickable `:::link-card` tiles. Each card takes `title`,
`href`, and an optional `icon`. Its Markdown body is the description. Body links
remain independently clickable. Use ordinary output URLs for `href`, relative
to the current page (including `.html`); wiki-links remain available in the body.

::::cards
:::link-card{title="Getting started" href="../guides/getting-started.html" icon="lucide:rocket"}
Install DocAnvil and create your first project.
:::
:::link-card{title="Theming" href="../guides/theming.html" icon="lucide:palette"}
Make your documentation your own.
:::
::::

````markdown
::::cards
:::link-card{title="Getting started" href="../guides/getting-started.html" icon="lucide:rocket"}
Install DocAnvil and create your first project.
:::
:::link-card{title="Theming" href="../guides/theming.html" icon="lucide:palette"}
Make your documentation your own.
:::
::::
````

Cards default to two columns. `columns="3"` selects three; both stack on mobile.
A standalone `:::link-card` works without a grid.

### Steps

Use `::::steps` with one `:::step{title="…"}` per stage of a tutorial. Step bodies
support Markdown, code blocks, and nested components. Numbering is automatic.

::::steps
:::step{title="Install"}
```bash
cargo install docanvil
```
:::
:::step{title="Create a project"}
```bash
docanvil new my-docs
```
:::
:::step{title="Preview"}
```bash
cd my-docs && docanvil serve
```
:::
::::

````markdown
::::steps
:::step{title="Install"}
```bash
cargo install docanvil
```
:::
:::step{title="Create a project"}
```bash
docanvil new my-docs
```
:::
:::step{title="Preview"}
```bash
cd my-docs && docanvil serve
```
:::
::::
````

### Details

Native expandable sections keep supplementary information close by. Set
`open="true"` to start expanded; the default is closed. They work without JavaScript.

:::details{title="Do I need JavaScript for these components?"}
No. Cards, steps, figures, columns, and details use HTML and CSS.
:::

```markdown
:::details{title="Do I need JavaScript for these components?"}
No. Cards, steps, figures, columns, and details use HTML and CSS.
:::
```

### Figures

Frame an image with optional `caption` text. `browser="true"` adds a decorative
browser bar. `alt` supplies the image's accessible description. Image paths use
the same resolution rules as Markdown images.

```markdown
:::figure{src="assets/screenshot.png" alt="Project settings screen" caption="Configure your project" browser="true"}
:::
```

Omit `src` to wrap Markdown images or another component in the figure body.
Without `caption`, no caption space is reserved.

### Columns

Use `::::columns` with `:::column` children for side-by-side explanations, code,
or screenshots. The default is two equal columns; `columns="3"` selects three.
All columns stack on mobile. Use longer outer fences when nesting directives.

::::columns
:::column
### Write
Use Markdown for your content and directives for richer layouts.
:::
:::column
### Publish
Build a static site and deploy it to your preferred host.
:::
::::

```markdown
::::columns
:::column
### Write
Use Markdown for your content and directives for richer layouts.
:::
:::column
### Publish
Build a static site and deploy it to your preferred host.
:::
::::
```

### More Callouts

`:::tip`, `:::success`, and `:::danger` share the note/warning layout and accept an
optional `title`. Each has its own accent and Lucide icon.

:::tip{title="A useful shortcut"}
Keep frequently used commands in a titled code block.
:::

:::success
Your site is ready to publish.
:::

:::danger{title="Check before deleting"}
Make sure you have a backup before deleting your content.
:::

```markdown
:::tip{title="A useful shortcut"}
Keep frequently used commands in a titled code block.
:::

:::success
Your site is ready to publish.
:::

:::danger{title="Check before deleting"}
Make sure you have a backup before deleting your content.
:::
```

## Custom Components {#custom-components}

Built-ins cover the common cases, but sometimes you want your own. Drop a Tera template into `theme/components/<name>.html` and use it with `:::name{...}` — same directive syntax as any built-in.

### Your First Component

Create `theme/components/card.html`:

```html
<div class="card">
  {% if attrs.title %}<h3>{{ attrs.title }}</h3>{% endif %}
  {{ body | safe }}
</div>
```

Then use it in your content:

```markdown
:::card{title="Quick start"}
Install DocAnvil and run `docanvil new my-docs`.
:::
```

Style it in your custom CSS (`.card { ... }`) and you've got a reusable block with none of the copy-paste.

### Template Variables

Every component template has access to:

| Variable | Type | Description |
|----------|------|-------------|
| `attrs` | map | The directive's attributes, e.g. `attrs.title` for `{title="..."}` |
| `body` | string (HTML) | The body, rendered through the same pipeline as a page — print it with `\| safe` |
| `body_raw` | string | The original, unrendered body text |
| `name` | string | The component's name |
| `inline` | bool | `true` when used inline (see below) |

Built-in components that need structured data pass extra variables — `tabs` (a list of `{title, body}`) for the tabs component, and `blocks` (a list of `{lang, code}`) for code-group. You'll see these if you eject either one.

### Icons

All bundled [Lucide icons](https://lucide.dev/icons/) are available by kebab-case
name. Feature tiles accept `icon="lucide:zap"` (or `icon="zap"`); emoji values
such as `icon="⚡"` still work. An unknown name with the explicit `lucide:` prefix
produces a component rendering warning and a visible error box so typos are easy to spot.

Custom component and layout templates can render icons with:

```html
{{ icon(name="lucide:rocket") | safe }}
```

Icons render as decorative inline SVGs at build time, inherit the surrounding
text colour, and use the `.docanvil-icon` class (default size: `1em`). Supply an
accessible label on the containing button or link when the icon is its only
content. Feature icons use the tile's existing `1.5rem` size.

The full library is bundled in DocAnvil, but only selected icons appear in page
HTML. No browser icon library, font, or CDN is needed. The ISC and Feather MIT
licence notices ship at `js/icons/lucide/LICENSE.txt`.

### Escaping

DocAnvil autoescapes template output, same as it would any untrusted string: `&`, `<`, `>`, `"`, and `'` are escaped. `body` is already-rendered HTML, so print it with `| safe` — exactly like `{{ content | safe }}` in `layout.html`. `attrs` and `body_raw` are plain text and should usually be printed without `| safe`.

### Optional Attributes

If a template references an attribute that wasn't passed (and has no fallback), the page shows an error box in its place and the build emits a warning — which fails under `--strict`. Give every optional attribute a default:

```html
{{ attrs.title | default(value="Note") }}
```

or guard it with an `{% if %}`:

```html
{% if attrs.title %}<h3>{{ attrs.title }}</h3>{% endif %}
```

### Nesting

Components nest inside each other, not just tabs inside tabs. Use more colons on the outer fence so its closing `:::` isn't mistaken for an inner one:

```markdown
::::card{title="Heads up"}
:::note
Nested components render just like top-level ones.
:::

Inline too: :::lozenge{type="warning" text="Beta"}
::::
```

### Inline Components

The same template renders both block and inline use — `:::name{attrs}` with no body and no closing fence. Inside the template, `inline` is `true` and `body` is empty, so check `attrs` instead of `body` for what to render. The built-in `lozenge.html` is a good example: it only ever uses `attrs`.

Keep an inline component inside a line of text, like `Status: :::lozenge{text="Done"}`. One written alone on its own line is read as the start of a block, and it'll scan all the way down to the next bare `:::` looking for a closing fence — swallowing everything in between.

### Restyling Built-ins

Want to change how a built-in looks or behaves? Eject its template and edit it:

```bash
docanvil component eject note
```

This copies the built-in's template to `theme/components/note.html`. A user template with a built-in's name takes over rendering for that component while keeping its data — editing `theme/components/tabs.html` still gets the `tabs` list, and `theme/components/code-group.html` still gets `blocks`.

If you restyle `tabs` or `code-group`, keep the `tab-header` / `tab-content` / `data-tab` / `active` classes and attributes — `docanvil.js` uses them to switch tabs. For `mermaid`, print the diagram source with `{{ body_raw | safe }}`: Mermaid reads the element's raw text, so escaped entities would break it.

### Checking Templates

`docanvil doctor` checks every file in `theme/components/` for Tera syntax errors, names that can't be used as a directive (letters, numbers, `_` and `-` only), and a `{{ body }}` printed without `| safe`. Run `docanvil component list` any time to see which components are built in, which are overridden, and which are your own.

## Nesting Directives

When nesting directives, use more colons on the outer fence to distinguish it from inner closings — this works for every component, built-in or custom, not just tabs. The `::::tabs` (four colons) and `:::tab` (three colons) pattern is the clearest example of it:

```markdown
::::tabs
:::tab{title="First"}
Content here.
:::
:::tab{title="Second"}
Content here.
:::
::::
```

The outer directive uses 4 colons (`::::`) while the inner ones use 3 (`:::`). The closing fence must match the exact number of colons used in the opening fence.

## Unknown Directives

If you use a directive name that doesn't match a built-in component, the content is wrapped in a `<div>` with the directive name as the class:

```markdown
:::custom-block{.extra}
This becomes a `<div class="custom-block extra">`.
:::
```

This lets you create custom styled blocks using your own CSS.

## Summary

| Component | Directive | Key Attribute | Default |
|-----------|-----------|---------------|---------|
| Note | `:::note` | `title` | `"Note"` |
| Warning | `:::warning` | `title` | `"Warning"` |
| Tabs | `::::tabs` + `:::tab` | `title` (on tab) | `"Tab 1"`, `"Tab 2"`, ... |
| Code Group | `:::code-group` | *(none)* | Language name from code fence |
| Mermaid | `:::mermaid` | *(none)* | Renders diagram via Mermaid.js |
| Hero | `::::hero` | `eyebrow`, `title` | *(none)* |
| Buttons | `:::buttons` | *(none)* | First link is the primary button |
| Feature tiles | `::::features` + `:::feature` | `icon`, `title` (on feature) | *(none)* |
| Link cards | `::::cards` + `:::link-card` | `title`, `href`, `icon`, `columns` (on grid) | Two columns |
| Steps | `::::steps` + `:::step` | `title` (on step) | Automatic numbering |
| Details | `:::details` | `title`, `open` | Closed |
| Figure | `:::figure` | `src`, `alt`, `caption`, `browser` | Plain frame |
| Columns | `::::columns` + `:::column` | `columns` (on container) | Two columns |
| Callouts | `:::tip`, `:::success`, `:::danger` | `title` | Type name |

:::note
Components are processed before Markdown rendering. This means you can use bold, italic, links, code, and other Markdown formatting inside any component.
:::

## Related Pages

- [[writing/markdown|Markdown]] — text formatting, tables, and other Markdown features
- [[writing/links-popovers|Wiki-links]] — double-bracket links and inline popovers
