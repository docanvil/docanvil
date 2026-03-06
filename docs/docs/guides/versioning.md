# Versioning

DocAnvil supports multi-version documentation sites out of the box. Each version gets its own URL prefix, navigation, and search index — and a version switcher in the header lets readers jump between versions. An automatic banner reminds them when they're reading an older version.

## Enabling Versioning

Add a `[version]` section to your `docanvil.toml`:

```toml
[version]
current = "v2"
enabled = ["v1", "v2"]

[version.display_names]
v1 = "v1.0"
v2 = "v2.0 (latest)"
```

That's all the config needed. DocAnvil will now look for version subdirectories in your content directory and build a separate site tree for each version.

## Directory Structure

Versions live as subdirectories inside your content directory — not as file suffixes. This keeps old versions self-contained and easy to freeze:

```text
docs/
  v1/
    index.md
    getting-started.md
    api/overview.md
  v2/
    index.md
    getting-started.md
    new-feature.md
    api/overview.md
```

Each version directory is an independent content tree. You can add, remove, or reorganize pages between versions without affecting other versions.

:::note{title="Why directories, not suffixes?"}
A suffix approach (`page.v1.md`) leads to hundreds of interleaved files in a single directory, makes freezing old versions messy, and combines badly with locale suffixes. Directory-based versioning keeps each version clean and self-contained.
:::

## Output Structure

Each version gets its own directory in the build output:

```text
dist/
  index.html            # Meta-refresh redirect to /v2/index.html
  js/docanvil.js        # Shared assets (one copy)
  robots.txt
  sitemap.xml           # All versions included
  404.html
  v1/
    index.html
    getting-started.html
    api/overview.html
    search-index.json   # v1 search index
  v2/
    index.html
    getting-started.html
    new-feature.html
    api/overview.html
    search-index.json   # v2 search index
```

The root `index.html` is a simple meta-refresh redirect — no JavaScript required, and it works on any static host.

## Navigation

DocAnvil builds a separate navigation tree for each version, so each version's sidebar only shows pages in that version.

### Auto-Discovered Nav

When you don't have a `nav.toml`, DocAnvil auto-discovers pages for each version separately. The v1 nav only shows v1 pages, the v2 nav only shows v2 pages.

### Per-Version nav.toml

You can create version-specific navigation files:

- `nav.v2.toml` — used for the v2 build
- `nav.v1.toml` — used for the v1 build
- `nav.toml` — fallback for any version without its own file

Slugs in nav files reference **base slugs** within the version directory — not the full version-prefixed path. A page at `docs/v2/guides/setup.md` has the base slug `guides/setup`, so your nav file uses:

<pre><code class="language-toml">&#91;[nav]]
page = "guides/setup"
</code></pre>

The same slug works across version nav files — DocAnvil resolves it to the correct version's page.

## Version Switcher

When multiple versions are enabled, a version switcher appears in the header bar. It shows:

- The current version code (e.g. "v2")
- A dropdown listing all enabled versions with display names
- The current version highlighted
- When the current page doesn't exist in a target version, the link falls back to that version's home page

The switcher is suppressed when only one version is configured (same behaviour as the locale switcher with a single locale).

## Older-Version Banner

When a reader is viewing a non-current version, a banner appears at the top of the page:

> ⚠️ You're viewing docs for **v1**. [Switch to latest (v2)](#)

This uses the `version.current` setting to determine what "latest" means. If you haven't set `current`, it defaults to the last entry in `enabled`.

The banner links directly to the same page in the latest version when that page exists, or to the latest version's home page if not.

## Wiki-Links

Wiki-links resolve within the current version. `[[getting-started]]` written in a v1 page links to the v1 version of that page. You don't need to add version prefixes to your links.

## Search

Each version gets its own search index (`v1/search-index.json`, `v2/search-index.json`). The search UI automatically loads the right index for the current version, so readers only see results from their version.

## Composing with i18n

Versioning and localisation compose cleanly. Enable both features in your config:

```toml
[version]
current = "v2"
enabled = ["v1", "v2"]

[locale]
default = "en"
enabled = ["en", "fr"]
```

Then use locale suffixes within version directories:

```text
docs/
  v2/
    index.en.md
    index.fr.md
    guides/
      setup.en.md
      setup.fr.md
```

The output nests locales inside versions:

```text
dist/
  v2/
    en/
      index.html
      guides/setup.html
      search-index.json
    fr/
      index.html
      guides/setup.html
      search-index.json
```

Per-version nav files can also be locale-specific. DocAnvil resolves them with the following priority:

1. `nav.{version}.{locale}.toml` — version + locale specific
2. `nav.{version}.toml` — version specific
3. `nav.{locale}.toml` — locale specific
4. `nav.toml` — global fallback

## Doctor Checks

The `docanvil doctor` command runs version-specific checks when `version.enabled` is non-empty:

| Check | Severity | What it catches |
|-------|----------|-----------------|
| `current-not-in-enabled` | Error | `version.current` specifies a version not in the `enabled` list |
| `version-dir-missing` | Error | An enabled version has no matching subdirectory in the content directory |
| `empty-version` | Warning | A version directory exists but contains no `.md` files |

Run `docanvil doctor --fix` to automatically create missing version directories.

## Backward Compatibility

When no `[version]` section exists in `docanvil.toml` (or `enabled` is empty):

- Pages build to the root output directory (no version prefixes)
- No version switcher appears
- No older-version banner is rendered
- Doctor skips version checks

Existing projects work without any changes.

## Related Pages

- [[guides/configuration|Configuration]] — `[version]` config options
- [[guides/localisation|Localisation]] — composing versioning with multi-language docs
- [[reference/project-structure|Project Structure]] — versioned directory layout
- [[reference/cli|CLI Commands]] — `docanvil doctor` version checks
