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

## Code Group

A specialized tab component for comparing code blocks across languages. Each fenced code block becomes a tab, with the language name as the tab label:

:::code-group
```rust
fn greet(name: &str) {
    println!("Hello, {name}!");
}
```

```python
def greet(name):
    print(f"Hello, {name}!")
```

```javascript
function greet(name) {
  console.log(`Hello, ${name}!`);
}
```
:::

Raw syntax:

````markdown
:::code-group
```rust
fn greet(name: &str) {
    println!("Hello, {name}!");
}
```

```python
def greet(name):
    print(f"Hello, {name}!")
```
:::
````

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

## Nesting Directives

When nesting directives, use more colons on the outer fence to distinguish it from inner closings. The `::::tabs` (four colons) and `:::tab` (three colons) pattern is the primary example of this:

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

:::note
Components are processed before Markdown rendering. This means you can use bold, italic, links, code, and other Markdown formatting inside any component.
:::

## Related Pages

- [[writing/markdown|Markdown]] — text formatting, tables, and other Markdown features
- [[writing/wiki-links|Wiki-links]] — double-bracket links and inline popovers
