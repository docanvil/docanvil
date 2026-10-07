---
{
  "description": "Quick start guide for getting up and running with DocAnvil"
}
---
# Installation

Get DocAnvil running and create your first documentation site.

## Install DocAnvil

The install scripts download a prebuilt binary for your platform (about 5 MB), check it against the release's published SHA-256 checksums, and put it on your `PATH`. No Rust toolchain needed.

::::tabs
:::tab{title="macOS / Linux"}
```bash
curl -fsSL https://github.com/docanvil/docanvil/releases/latest/download/install.sh | sh
```

Installs to `~/.local/bin`. If that folder isn't on your `PATH` yet, the script tells you the line to add to your shell profile.
:::
:::tab{title="Windows"}
```powershell
irm https://github.com/docanvil/docanvil/releases/latest/download/install.ps1 | iex
```

Installs to `%LOCALAPPDATA%\docanvil\bin` and adds it to your user `PATH`. Open a new terminal afterwards.
:::
:::tab{title="From crates.io"}
```bash
# Compiles DocAnvil and its dependencies (needs a Rust toolchain)
cargo install docanvil
```
:::
:::tab{title="From source"}
```bash
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

### Install options

| Option | Environment variable | What it does |
|---|---|---|
| `--version 1.2.0` | `DOCANVIL_VERSION` | Install a specific version instead of the latest |
| `--install-dir DIR` | `DOCANVIL_INSTALL_DIR` | Install somewhere else, e.g. `/usr/local/bin` in CI |
| `--force` | | Reinstall even if that version is already there |
| `--quiet` | | Only print errors |

Pass options to the piped script with `sh -s --`:

```bash
curl -fsSL https://github.com/docanvil/docanvil/releases/latest/download/install.sh | sh -s -- --version 1.2.0
```

On Windows, set the environment variables before running the one-liner (for example `$env:DOCANVIL_VERSION = "1.2.0"`).

Re-running the script when that version is already installed does nothing and exits successfully, so it's safe to use in CI.

### Manual download

Prefer not to pipe a script into your shell, or working on a locked-down machine? Every [release](https://github.com/docanvil/docanvil/releases) has a prebuilt archive per platform:

| Platform | Archive |
|---|---|
| macOS (Apple Silicon) | `docanvil-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `docanvil-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| Linux x86_64 (any distro, including Alpine) | `docanvil-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 (any distro) | `docanvil-vX.Y.Z-aarch64-unknown-linux-musl.tar.gz` |
| Windows x86_64 (also runs on Windows on ARM) | `docanvil-vX.Y.Z-x86_64-pc-windows-msvc.zip` |

Download the archive and the release's `SHA256SUMS`, check the archive against it, then put the binary somewhere on your `PATH`:

::::tabs
:::tab{title="macOS / Linux"}
```bash
VERSION=1.1.4                 # the release you want
TARGET=aarch64-apple-darwin   # from the table above
BASE=https://github.com/docanvil/docanvil/releases/download/v$VERSION

curl -fLO "$BASE/docanvil-v$VERSION-$TARGET.tar.gz"
curl -fLO "$BASE/SHA256SUMS"
shasum -a 256 -c --ignore-missing SHA256SUMS   # or: sha256sum -c --ignore-missing SHA256SUMS

tar -xzf "docanvil-v$VERSION-$TARGET.tar.gz"
mkdir -p ~/.local/bin && mv docanvil ~/.local/bin/
```

The checksum line must print `OK`. If you downloaded the archive in a browser on macOS, clear the quarantine flag before running it: `xattr -d com.apple.quarantine ~/.local/bin/docanvil`.
:::
:::tab{title="Windows"}
```powershell
$Version = "1.1.4"
$Asset = "docanvil-v$Version-x86_64-pc-windows-msvc.zip"
$Base = "https://github.com/docanvil/docanvil/releases/download/v$Version"

Invoke-WebRequest "$Base/$Asset" -OutFile $Asset
Invoke-WebRequest "$Base/SHA256SUMS" -OutFile SHA256SUMS
(Get-FileHash $Asset -Algorithm SHA256).Hash.ToLower()
Select-String $Asset SHA256SUMS

Expand-Archive $Asset -DestinationPath "$env:LOCALAPPDATA\docanvil\bin" -Force
```

The two hashes must match. Then add `%LOCALAPPDATA%\docanvil\bin` to your user `PATH`.
:::
::::

Releases up to v1.1.3 have no `SHA256SUMS` file. For those, compare against the SHA-256 digest GitHub shows next to each asset on the release page.

## Updating

```bash
docanvil update          # check for a newer release and upgrade, with a confirmation prompt
docanvil update --check  # just report whether there's a newer release
docanvil update --yes    # upgrade without prompting
```

`docanvil serve` also prints a one-line notice when a new release is out. It checks at most once a day and never in CI. Set `DOCANVIL_NO_UPDATE_CHECK=1` to turn it off.

If you installed with `cargo install`, update with `cargo install docanvil --force` instead.

## Create a Project

Scaffold a new documentation project with `docanvil new`:

```bash
docanvil new my-docs
```

This creates the following structure:

```text
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

For CI/CD pipelines, use `--strict` to fail the build when there are any warnings:

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
