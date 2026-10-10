---
{
  "description": "Upgrade between DocAnvil versions, or move an existing docs site over from another tool."
}
---

# Migration Guide

DocAnvil follows [Semantic Versioning](https://semver.org) and is stable at 1.x, so upgrading within 1.x never requires changes to your project. This page covers anything that ever does.

## Migrating Between Versions

Nothing to migrate yet! Every 1.x release works with your existing config, content and theme overrides.

If a future major version needs changes, you'll find step-by-step migration instructions here — covering config changes, renamed options, and anything else that might affect your project.

## Coming From Another Tool

If you're moving an existing documentation site to DocAnvil, the process is straightforward:

::::steps
:::step{title="Create a project"}
Run `docanvil new my-docs` to scaffold a fresh project
:::
:::step{title="Copy your content"}
Copy your Markdown files into the `docs/` directory
:::
:::step{title="Set up navigation"}
Set up your navigation in `nav.toml` (or let auto-discovery handle it)
:::
:::step{title="Adapt the syntax"}
Adjust any tool-specific syntax (front matter format, component directives, etc.)
:::
::::

DocAnvil uses JSON front matter (not YAML), so you may need to convert front matter blocks. See [[writing/front-matter|Front Matter]] for the format.

:::note{title="Need help migrating?"}
If you run into issues moving from another documentation tool, open a discussion on [GitHub](https://github.com/docanvil/docanvil/discussions) — we'd love to hear what tripped you up so we can make the process smoother.
:::
