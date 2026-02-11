# Components

DocAnvil includes several built-in components you can use in your Markdown files
with the `:::name{attrs}` directive syntax.

## Note

:::note{title="Information"}
This is a note component. Use it to highlight important information.
:::

## Warning

:::warning{title="Caution"}
This is a warning component. Use it to call out potential issues.
:::

## Tabs

:::tabs
```rust tab="Rust"
fn main() {
    println!("Hello from Rust!");
}
```

```python tab="Python"
print("Hello from Python!")
```

```javascript tab="JavaScript"
console.log("Hello from JavaScript!");
```
:::

## Usage

Directives use fenced syntax:

````markdown
:::note{title="My Title"}
Content goes here.
:::
````

See [[markdown|Markdown Features]] for more syntax, or check the
[[configuration]] page for theme customization options.
