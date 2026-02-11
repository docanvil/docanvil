# Welcome to docs

This is your new documentation site, powered by [DocAnvil](https://github.com/docanvil/docanvil).

## Getting Started

Edit this file at `docs/index.md` to start writing your documentation.
Check out the guides to learn more:

- [[getting-started]] — install and run your first site
- [[configuration]] — customize your project settings

## Explore

- [[overview|API Reference]] — browse the API docs
- [[components]] — see the built-in components in action
- [[markdown]] — learn about supported Markdown features

### Features

- **Markdown** with GFM extensions (tables, task lists, footnotes)
- **Wiki-style links**: Link to other pages with `[[page-name]]`
- **Custom components**: Use `:::note`, `:::warning`, `:::tabs` directives
- **Theming**: Customize with CSS variables in `docanvil.toml`
- **Live reload**: Run `docanvil serve` for instant preview

:::note{title="Tip"}
Run `docanvil serve` in this directory to see your docs with live reloading!
:::

## Customizing the Theme

Edit `theme/custom.css` to override any CSS variable or add your own styles.
You can also set variables directly in `docanvil.toml`:

```toml
[theme.variables]
color-primary = "#10b981"
font-body = "Georgia, serif"
```

## Next Steps

- Add more `.md` files to the `docs/` directory
- Customize your theme in `docanvil.toml` or `theme/custom.css`
- Run `docanvil build` to generate a static site
