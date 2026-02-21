# Localisation

DocAnvil supports multi-language documentation sites out of the box. Each locale gets its own URL prefix, navigation, search index, and a language switcher in the header — all from a single content directory.

## Enabling i18n

Add a `[locale]` section to your `docanvil.toml`:

```toml
[locale]
default = "en"
enabled = ["en", "fr"]
auto_detect = true

[locale.display_names]
en = "English"
fr = "Français"
```

That's it. DocAnvil will now look for locale suffixes in your filenames and build a separate site tree for each language.

## File Naming Convention

Add the locale code as a suffix before the `.md` extension:

```
docs/
  index.en.md          # English home page
  index.fr.md          # French home page
  guides/
    getting-started.en.md
    getting-started.fr.md
    advanced.en.md      # No French translation (yet)
```

Files **without** a locale suffix are treated as the default locale. So if your default is `"en"`, then `index.md` and `index.en.md` are equivalent — both become the English version.

:::note{title="No false positives"}
Only suffixes that match your `enabled` list are recognized as locale codes. A file like `api.v2.md` won't be mistakenly treated as a locale — `v2` isn't in your enabled list.
:::

## Output Structure

Each locale gets its own directory in the build output:

```
dist/
  js/docanvil.js        # Shared assets (one copy)
  robots.txt
  sitemap.xml           # Contains all locales
  404.html              # Links to each locale's home page
  en/
    index.html
    guides/
      getting-started.html
    search-index.json   # English search index
  fr/
    index.html
    guides/
      getting-started.html
    search-index.json   # French search index
```

All locales get URL prefixes — even the default. This keeps URLs consistent and predictable.

## Navigation

### Auto-Discovered Nav

When you don't have a `nav.toml`, DocAnvil auto-discovers pages for each locale separately. The English nav only shows English pages, the French nav only shows French pages.

### Per-Locale nav.toml

You can create locale-specific navigation files:

- `nav.fr.toml` — used for the French build
- `nav.en.toml` — used for the English build
- `nav.toml` — fallback for any locale without its own file

Slugs in nav files reference **base slugs** without the locale suffix. A page at `docs/guides/setup.en.md` has the base slug `guides/setup`, so your nav file uses:

<pre><code class="language-toml">&#91;[nav]]
page = "guides/setup"
</code></pre>

The same slug works in both the English and French nav files — DocAnvil resolves it to the correct locale's page.

## Wiki-Links

Wiki-links resolve **within the same locale**. If you write `[[getting-started]]` in a French page, it links to the French version of that page. You don't need to add locale suffixes to your links.

This means your content files can be translated independently without updating any internal links.

## Search

Each locale gets its own search index (`en/search-index.json`, `fr/search-index.json`). The search UI automatically loads the right index for the current locale, so users only see results in their language.

## Language Switcher

When i18n is enabled, a language switcher appears in the header bar. It shows:

- A flag emoji with the current locale code (e.g. 🇬🇧 EN)
- A dropdown listing all enabled locales with flag emoji and display names
- The current locale highlighted
- Unavailable translations greyed out (when a page doesn't exist in that locale)

Clicking a locale navigates to the same page in the selected language. If the page doesn't exist in the target locale, the link goes to that locale's home page instead.

### Flag Emoji

DocAnvil auto-assigns flag emoji based on locale codes — `en` gets 🇬🇧, `fr` gets 🇫🇷, `de` gets 🇩🇪, and so on. Unknown locale codes get a 🌐 globe.

To override the default flag for a locale (e.g. using the US flag for English), add a `[locale.flags]` table:

```toml
[locale.flags]
en = "🇺🇸"
```

This is useful when a language maps to multiple countries and you want to match your audience.

## Browser Auto-Detection

When `auto_detect` is `true` (the default), DocAnvil checks the visitor's browser language on their first visit:

1. If the user has previously chosen a language (stored in `localStorage`), that choice is respected
2. If not, the browser's `navigator.language` is checked against the enabled locales
3. If it matches a different locale than the current page, the user is redirected
4. The detected language is saved to `localStorage` to prevent repeated redirects

Set `auto_detect = false` to disable this behavior entirely.

## Missing Translations

When a page exists in some locales but not all, DocAnvil emits a build warning:

```
warning: page 'guides/advanced' has no translation for locale 'fr'
  hint: Create a file with the '.fr.md' suffix to add a translation.
```

In `--strict` mode, these warnings become errors and the build fails. This is useful in CI to enforce complete translations before deploying.

The `docanvil doctor` command also checks translation coverage when i18n is enabled, reporting:

- **missing-translation** (Warning) — pages missing in some locales
- **orphaned-locale** (Warning) — files with locale suffixes not in the enabled list
- **missing-default-locale** (Error) — default locale has no pages at all

## SEO

DocAnvil generates comprehensive multilingual SEO signals when i18n is enabled:

### hreflang Tags

Each page includes `<link rel="alternate" hreflang="...">` tags pointing to every available translation. These tell search engines which pages are translations of each other, preventing duplicate content issues and ensuring users see results in their language.

An `hreflang="x-default"` tag is also emitted, pointing to the default locale's version of the page. This acts as a fallback for users whose language isn't in your enabled list.

### Canonical URLs and Open Graph

When `site_url` is configured, each page gets:

- `<link rel="canonical">` — the definitive URL for the page
- `<meta property="og:url">` — the URL used when the page is shared on social media
- `<meta property="og:locale">` — the current page's locale
- `<meta property="og:locale:alternate">` — tags for each translation

These tags use absolute URLs derived from `site_url`. Without `site_url`, hreflang tags still work with relative URLs, but canonical and og:url are omitted.

### Sitemap

The sitemap includes all pages across all locales with `xhtml:link` hreflang annotations:

```xml
<url>
  <loc>https://example.com/en/guides/setup.html</loc>
  <xhtml:link rel="alternate" hreflang="en" href="https://example.com/en/guides/setup.html"/>
  <xhtml:link rel="alternate" hreflang="fr" href="https://example.com/fr/guides/setup.html"/>
  <xhtml:link rel="alternate" hreflang="x-default" href="https://example.com/en/guides/setup.html"/>
</url>
```

Google recommends both in-page hreflang tags and sitemap hreflang annotations — DocAnvil does both automatically.

### HTML lang Attribute

The `<html>` tag includes a `lang` attribute matching the current locale, which helps search engines and screen readers.

:::note{title="Tip"}
Set `site_url` in your `docanvil.toml` to get the most out of multilingual SEO. Without it, canonical URLs and absolute hreflang links can't be generated.
:::

## Backward Compatibility

When no `[locale]` section exists in `docanvil.toml`:

- Pages build to the root output directory (no locale prefixes)
- No language switcher appears
- Wiki-links resolve globally as before
- Search index lives at `/search-index.json`
- Doctor skips translation checks

Existing single-language projects work without any changes.

## Related Pages

- [[guides/configuration|Configuration]] — `[locale]` config options
- [[writing/wiki-links|Links & Popovers]] — how wiki-links resolve within locales
- [[reference/project-structure|Project Structure]] — i18n directory layout
- [[reference/cli|CLI Commands]] — `--strict` mode and `docanvil doctor`
