# Installation

Get DocAnvil running and create your first documentation site.

## Install DocAnvil

::::tabs
:::tab{title="From crates.io"}
```bash
# Install from crates.io (when published)
cargo install docanvil
```
:::
:::tab{title="From GitHub"}
```bash
# Build from source
git clone https://github.com/docanvil/docanvil.git
cd docanvil
cargo install --path .
```
:::
::::

Verify the installation:

```bash
docanvil --help
```

## Create a Project

Scaffold a new documentation project with `docanvil new`:

```bash
docanvil new my-docs
```

This creates the following structure:

```
my-docs/
  docanvil.toml        # Project configuration
  nav.toml             # Navigation structure
  docs/                # Your Markdown content
    index.md           # Home page
    guides/
      getting-started.md
      configuration.md
  theme/
    custom.css         # Your CSS overrides
```

## Start the Dev Server

```bash
cd my-docs
docanvil serve
```

The dev server starts at [http://localhost:3000](http://localhost:3000) by default. You can change the host and port:

```bash
docanvil serve --host 0.0.0.0 --port 8080
```

## Write Your First Page

Create a new Markdown file anywhere in the `docs/` directory:

```markdown
# My New Page

Welcome to my documentation!

- Supports **bold**, *italic*, and ~~strikethrough~~
- Add [[index|links to other pages]] with wiki-link syntax
```

Save the file and your browser will reload automatically. The page is discovered and added to the navigation.

## Build for Production

When you're ready to deploy, generate the static site:

```bash
docanvil build
```

The output goes to the `dist/` directory by default. Upload it to any static host — GitHub Pages, Netlify, Vercel, S3, or just a plain web server.

Use `--clean` to remove the output directory before building:

```bash
docanvil build --clean
```

For integration with CI/CD pipelines, use `--strict` to return an error and non-zero exit code when there is any warnings during the build:

```bash
docanvil build --strict
```

## Checklist

- [x] Install DocAnvil
- [x] Run `docanvil new` to scaffold a project
- [x] Start the dev server with `docanvil serve`
- [ ] Write your pages in Markdown
- [ ] Customize the theme
- [ ] Build and deploy with `docanvil build`

## Next Steps

- [[guides/configuration|Configure]] your project and navigation
- Learn about [[writing/markdown|Markdown features]] and [[writing/components|components]]
- [[guides/theming|Customize the theme]] to match your brand

:::note
DocAnvil watches all files in your project directory. Changes to Markdown, config files, CSS, and templates all trigger a live reload.
:::
