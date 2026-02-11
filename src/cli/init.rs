use std::path::Path;

use owo_colors::OwoColorize;

use crate::error::Result;

pub fn run(name: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        return Err(crate::error::Error::Render(format!(
            "directory '{}' already exists",
            name
        )));
    }

    // Create project structure
    std::fs::create_dir_all(project_dir.join("docs"))?;
    std::fs::create_dir_all(project_dir.join("theme"))?;

    // Write docanvil.toml
    let config = format!(
        r##"[project]
name = "{name}"
content_dir = "docs"

[build]
output_dir = "dist"

[theme]
custom_css = "theme/custom.css"
# Set CSS variables to customize the theme:
# [theme.variables]
# color-primary = "#6366f1"
# font-body = "Georgia, serif"
"##
    );
    std::fs::write(project_dir.join("docanvil.toml"), config)?;

    // Write initial index.md
    let index = format!(
        r##"# Welcome to {name}

This is your new documentation site, powered by [DocAnvil](https://github.com/docanvil/docanvil).

## Getting Started

Edit this file at `docs/index.md` to start writing your documentation.

### Features

- **Markdown** with GFM extensions (tables, task lists, footnotes)
- **Wiki-style links**: Link to other pages with `[[page-name]]`
- **Custom components**: Use `:::note`, `:::warning`, `:::tabs` directives
- **Theming**: Customize with CSS variables in `docanvil.toml`
- **Live reload**: Run `docanvil serve` for instant preview

:::note{{title="Tip"}}
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
"##
    );
    std::fs::write(project_dir.join("docs/index.md"), index)?;

    // Write custom.css starter template
    let custom_css = include_str!("../theme/starter_custom.css");
    std::fs::write(project_dir.join("theme/custom.css"), custom_css)?;

    eprintln!(
        "{} Created project '{}' at {}",
        "✓".green().bold(),
        name.bold(),
        project_dir.display()
    );
    eprintln!();
    eprintln!("  {} {}", "cd".dimmed(), name);
    eprintln!("  {} serve", "docanvil".dimmed());
    eprintln!();

    Ok(())
}
