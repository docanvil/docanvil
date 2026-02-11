# DocAnvil — Forge beautiful static documentation from Markdown.

**DocAnvil** is a Rust-based static documentation generator that turns plain
Markdown into a fast, polished HTML site — with live reloading, customizable
components, and flexible styling.

Built for developers who want full control over how their documentation looks
and behaves, DocAnvil focuses on **clarity**, **performance**, and
**composability**. Write in Markdown, extend with custom components, and let
DocAnvil handle the rest.

Whether you're documenting a Rust crate, a library, or an entire product,
DocAnvil helps you forge documentation that feels solid, intentional, and
well-crafted.

## Why DocAnvil?

- ⚡ **Fast by default** — built in Rust for quick builds and snappy dev cycles
- 🔁 **Hot reloading** — see changes instantly as you edit Markdown
- 🧩 **Custom components** — extend beyond Markdown when you need to
- 🎨 **Configurable styling** — themes without fighting the tool
- 📦 **Static output** — deploy anywhere

## CLI Usage

DocAnvil is designed to be used from the command line during both development
and deployment.

### Install

```bash
cargo install docanvil
```

### Initialise a new project

```bash
docanvil init 
```

### Serve the current project (with hot reloading)

Simplest way to serve the current project is just to run:
```bash
docanvil serve
```

However you can also specify the host and listening ports:
```bash
docanvil serve --host 0.0.0.0 --port 3000
```

### Build the current project 

To output the current project as static html you can use the following command.

By default the built static site will be output to 'dist/'

```bash
docanvil build
```

You can pass an out flag and an alternative path instead

```bash
docanvil build --out dist/
```