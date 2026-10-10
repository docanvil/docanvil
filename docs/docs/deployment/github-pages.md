---
{
  "title": "GitHub Pages",
  "description": "Deploy your DocAnvil docs to GitHub Pages with GitHub Actions"
}
---
# Deploying to GitHub Pages

This guide sets up a GitHub Actions workflow that builds your DocAnvil docs and publishes them to GitHub Pages every time you push to `main`. It's the same setup these docs use.

## Prerequisites

You'll need:

- A GitHub repository containing your DocAnvil project
- GitHub Pages set to deploy from **GitHub Actions**: go to **Settings → Pages → Build and deployment → Source** and choose **GitHub Actions**

No secrets or tokens are needed. The workflow uses GitHub's built-in Pages permissions.

## Set Your Base URL

Most repositories are published as a **project site** at `https://<user>.github.io/<repo>/`. Every page lives under `/<repo>/`, so tell DocAnvil about it in `docanvil.toml`:

```toml
[build]
base_url = "/my-repo/"
site_url = "https://my-user.github.io/my-repo/"
```

- **`base_url`** is the path prefix for every link and asset. Without it, CSS, JS and navigation links point at the domain root and the site loads unstyled.
- **`site_url`** is the full public address. DocAnvil uses it for `sitemap.xml`, `llms.txt`, canonical URLs and other SEO tags.

If you publish a **user or organisation site** (a repository named `<user>.github.io`) or use a custom domain, the site is served from the root, so keep `base_url = "/"` and set `site_url` to that address.

:::note
`docanvil serve` always uses `/` as the base URL and builds into its own folder, so local previews keep working whatever `base_url` you set.
:::

## Workflow Configuration

Create `.github/workflows/docs.yml`:

```yaml
name: Publish docs

on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Install the latest DocAnvil binary (checksum-verified)
      - name: Install DocAnvil
        run: curl -fsSL https://github.com/docanvil/docanvil/releases/latest/download/install.sh | DOCANVIL_INSTALL_DIR=/usr/local/bin sh

      - name: Build docs
        run: docanvil build --strict

      - uses: actions/upload-pages-artifact@v3
        with:
          path: dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

A few things worth noting:

- **`permissions`** grants the workflow just enough to publish: `pages: write` and `id-token: write` are what `deploy-pages` needs.
- **`concurrency`** makes sure only one deploy runs at a time. If you push twice in quick succession, the older run is cancelled.
- **`--strict`** fails the build on any warning, so broken wiki-links, missing images and similar problems stop the deploy before anything goes live.
- **`workflow_dispatch`** adds a **Run workflow** button in the Actions tab, which is handy after changing Pages settings.

## Adjusting for Your Project Layout

The workflow above assumes `docanvil.toml` is at the root of your repository. If your docs live in a subdirectory, point the build at it and upload that folder's output:

```yaml
      - name: Build docs
        run: docanvil build --strict --path docs

      - uses: actions/upload-pages-artifact@v3
        with:
          path: docs/dist
```

To publish only when the docs change, add a path filter to the trigger:

```yaml
on:
  push:
    branches: [main]
    paths:
      - "docs/**"
      - ".github/workflows/docs.yml"
```

## Last Updated Dates

`actions/checkout` fetches only the latest commit by default. If you've turned on [[guides/configuration|`[last_updated]`]], that would give every page the same date, so DocAnvil warns and `--strict` fails the build. Fetch the full history instead:

```yaml
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
```

## Custom Domains

Set the domain under **Settings → Pages → Custom domain**. Because the workflow deploys an artifact rather than a branch, you don't need a `CNAME` file in your output. Then update `docanvil.toml` so links are built for the root of your domain:

```toml
[build]
base_url = "/"
site_url = "https://docs.example.com/"
```

## What Happens on Each Push

When you push to `main`:

1. GitHub Actions checks out your repository
2. The install script fetches the latest DocAnvil binary from GitHub Releases and verifies its checksum
3. `docanvil build --strict` builds the site into `dist/`, and fails if anything needs fixing
4. The output is uploaded as a Pages artifact and deployed
5. The new version is live, usually within a minute, at the URL shown on the workflow run

:::note
The install script fetches the latest stable release every time. If you need reproducible builds with a pinned version, set `DOCANVIL_VERSION` as well, e.g. `DOCANVIL_INSTALL_DIR=/usr/local/bin DOCANVIL_VERSION=1.2.0 sh`. See [[guides/getting-started|Installation]] for all the install options.
:::
