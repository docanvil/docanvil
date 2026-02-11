# Markdown Features

DocAnvil supports GitHub Flavored Markdown (GFM) with several extensions.

## Tables

| Feature       | Supported |
|---------------|-----------|
| Tables        | Yes       |
| Task lists    | Yes       |
| Strikethrough | Yes       |
| Footnotes     | Yes       |
| Front matter  | Yes       |

## Task Lists

- [x] Set up project with `docanvil init`
- [x] Start dev server with `docanvil serve`
- [ ] Write your first page
- [ ] Customize the theme

## Strikethrough

This text has ~~strikethrough~~ formatting.

## Footnotes

DocAnvil is built on comrak[^1], which supports GFM footnotes[^2].

[^1]: comrak is a CommonMark + GFM compatible Markdown parser written in Rust.
[^2]: Footnotes render as numbered references at the bottom of the page.

## Wiki-Links

Link to other pages using double-bracket syntax:

- `[[page-name]]` — links using the page title (e.g. [[components]])
- `[[page-name|custom text]]` — links with custom display text (e.g. [[overview|API docs]])

## Code Blocks

Fenced code blocks support syntax highlighting:

```rust
fn greet(name: &str) {
    println!("Hello, {}!", name);
}
```

See [[components]] for the built-in directive components, or visit the
[[overview|API Reference]] for example API documentation.
