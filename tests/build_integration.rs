mod integration_helpers;

use std::fs;

use integration_helpers::{
    DEFAULT_CONFIG, build_project, build_project_strict, create_project, output_exists, read_output,
};

const VERSION_CONFIG: &str = r#"
[project]
name = "Test Docs"

[version]
current = "v2"
enabled = ["v1", "v2"]

[version.display_names]
v1 = "v1.0"
v2 = "v2.0 (latest)"
"#;

const VERSION_I18N_CONFIG: &str = r#"
[project]
name = "Test Docs"

[version]
current = "v2"
enabled = ["v1", "v2"]

[locale]
default = "en"
enabled = ["en", "fr"]

[locale.display_names]
en = "English"
fr = "Français"
"#;

#[test]
fn test_basic_build() {
    let dir = create_project(DEFAULT_CONFIG, &[("index.md", "# Welcome\n\nHello world.")]);
    build_project(dir.path()).expect("build should succeed");

    assert!(output_exists(dir.path(), "index.html"));
    assert!(output_exists(dir.path(), "robots.txt"));
    assert!(output_exists(dir.path(), "sitemap.xml"));
    assert!(output_exists(dir.path(), "404.html"));
    assert!(output_exists(dir.path(), "js/docanvil.js"));

    let html = read_output(dir.path(), "index.html");
    assert!(
        html.contains("Hello world"),
        "page content should appear in output"
    );
    assert!(html.contains("<title>"), "output should have a title tag");
}

#[test]
fn test_multi_page_build() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[
            ("index.md", "# Home\n\nWelcome page."),
            ("guide.md", "# Guide\n\nA guide page."),
            ("reference.md", "# Reference\n\nAPI reference."),
        ],
    );
    build_project(dir.path()).expect("build should succeed");

    assert!(output_exists(dir.path(), "index.html"));
    assert!(output_exists(dir.path(), "guide.html"));
    assert!(output_exists(dir.path(), "reference.html"));

    // Nav should contain links to all pages
    let index_html = read_output(dir.path(), "index.html");
    assert!(
        index_html.contains("guide.html"),
        "nav should link to guide"
    );
    assert!(
        index_html.contains("reference.html"),
        "nav should link to reference"
    );
}

#[test]
fn test_nav_ordering() {
    let config = r#"
[project]
name = "Nav Test"
"#;

    let nav_toml = r#"
[[nav]]
page = "second"

[[nav]]
page = "first"

[[nav]]
page = "index"
"#;

    let dir = create_project(
        config,
        &[
            ("index.md", "# Home"),
            ("first.md", "# First Page"),
            ("second.md", "# Second Page"),
        ],
    );

    // Write nav.toml
    std::fs::write(dir.path().join("nav.toml"), nav_toml).expect("failed to write nav.toml");

    build_project(dir.path()).expect("build should succeed");

    // The nav HTML should list pages in the nav.json order: second, first, index
    let html = read_output(dir.path(), "second.html");

    // Find the nav section — look for href occurrences in the nav list
    let pos_second = html
        .find("href=\"/second.html\"")
        .expect("nav should contain second link");
    let pos_first = html
        .find("href=\"/first.html\"")
        .expect("nav should contain first link");
    assert!(
        pos_second < pos_first,
        "second should appear before first in nav (nav.toml ordering)\nhtml snippet: {}",
        &html[pos_second.saturating_sub(50)..pos_first + 80]
    );
}

#[test]
fn test_wikilinks_resolve() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[
            ("index.md", "# Home\n\nSee [[guide]] for details."),
            ("guide.md", "# Guide\n\nThe guide content."),
        ],
    );
    build_project(dir.path()).expect("build should succeed");

    let html = read_output(dir.path(), "index.html");
    assert!(
        html.contains("href=\"/guide.html\""),
        "wikilink should resolve to guide.html, got: {}",
        &html
            [html.find("guide").unwrap_or(0)..html.find("guide").unwrap_or(0) + 80.min(html.len())]
    );
}

#[test]
fn test_wikilink_syntax_in_code_block_does_not_warn_strict() {
    // TOML's `[[nav]]` table-array syntax looks like a wiki-link but lives inside
    // a fenced code block and an inline code span — neither should be rewritten
    // or trigger a broken-link warning, even under --strict (issue #57).
    // `site_url` is set so the only warning we could trip here is the one under test.
    let config =
        "[project]\nname = \"Test Docs\"\n\n[build]\nsite_url = \"https://docs.example.com\"\n";
    let dir = create_project(
        config,
        &[(
            "index.md",
            "# Home\n\n\
             ```toml\n\
             [[nav]]\n\
             page = \"x\"\n\
             ```\n\n\
             Inline `[[nav]]` too.\n",
        )],
    );

    build_project_strict(dir.path()).expect("strict build should succeed with no broken links");

    let html = read_output(dir.path(), "index.html");
    assert!(
        html.contains("[[nav]]"),
        "code block content should be left exactly as written, got: {html}"
    );
    assert!(
        !html.contains("broken-link popover-trigger"),
        "code block content should not trigger a broken-link popover, got: {html}"
    );
}

#[test]
fn test_strict_mode_broken_link() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[("index.md", "# Home\n\nSee [[nonexistent]] page.")],
    );

    let result = build_project_strict(dir.path());
    assert!(result.is_err(), "strict build with broken link should fail");

    let err = result.unwrap_err();
    assert!(
        matches!(err, docanvil::error::Error::StrictWarnings(_)),
        "error should be StrictWarnings, got: {err}"
    );
}

#[test]
fn test_search_index() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[("index.md", "# Welcome\n\nSearchable content here.")],
    );
    build_project(dir.path()).expect("build should succeed");

    assert!(output_exists(dir.path(), "search-index.json"));

    let json_str = read_output(dir.path(), "search-index.json");
    let index: serde_json::Value = serde_json::from_str(&json_str).expect("should be valid JSON");

    let arr = index.as_array().expect("search index should be an array");
    assert!(!arr.is_empty(), "search index should have entries");

    let entry = &arr[0];
    assert!(entry.get("title").is_some(), "entry should have title");
    assert!(entry.get("url").is_some(), "entry should have url");
    assert!(entry.get("body").is_some(), "entry should have body");
}

#[test]
fn test_front_matter() {
    let page = r#"---
{"title": "Custom Title"}
---
# Ignored Heading

Body text."#;

    let dir = create_project(DEFAULT_CONFIG, &[("index.md", page)]);
    build_project(dir.path()).expect("build should succeed");

    let html = read_output(dir.path(), "index.html");
    assert!(
        html.contains("Custom Title"),
        "custom front matter title should appear in output"
    );
}

#[test]
fn test_title_derived_slug() {
    let page = r#"---
{"title": "Setup Guide"}
---
# Setup Guide

How to set up."#;

    let dir = create_project(
        DEFAULT_CONFIG,
        &[("index.md", "# Home"), ("01-setup.md", page)],
    );
    build_project(dir.path()).expect("build should succeed");

    // Should produce setup-guide.html (not 01-setup.html)
    assert!(
        output_exists(dir.path(), "setup-guide.html"),
        "title-derived slug should produce setup-guide.html"
    );
    assert!(
        !output_exists(dir.path(), "01-setup.html"),
        "old filename-based output should not exist"
    );

    let html = read_output(dir.path(), "setup-guide.html");
    assert!(
        html.contains("Setup Guide"),
        "page should contain the title"
    );
}

#[test]
fn test_explicit_slug_field() {
    let page = r#"---
{"title": "My Page", "slug": "custom-url"}
---
# My Page

Content."#;

    let dir = create_project(
        DEFAULT_CONFIG,
        &[("index.md", "# Home"), ("boring-name.md", page)],
    );
    build_project(dir.path()).expect("build should succeed");

    // Should use the explicit slug, not the title-derived one
    assert!(
        output_exists(dir.path(), "custom-url.html"),
        "explicit slug should produce custom-url.html"
    );
    assert!(
        !output_exists(dir.path(), "boring-name.html"),
        "old filename-based output should not exist"
    );
    assert!(
        !output_exists(dir.path(), "my-page.html"),
        "title-derived slug should not be used when explicit slug is set"
    );
}

#[test]
fn test_wikilink_resolves_old_slug() {
    let setup_page = r#"---
{"title": "Setup Guide"}
---
# Setup Guide

How to set up."#;

    let index_page = "# Home\n\nSee [[01-setup]] for setup instructions.";

    let dir = create_project(
        DEFAULT_CONFIG,
        &[("index.md", index_page), ("01-setup.md", setup_page)],
    );
    build_project(dir.path()).expect("build should succeed");

    let html = read_output(dir.path(), "index.html");
    // Wiki-link using old filename slug should resolve to new slug's URL
    assert!(
        html.contains("href=\"/setup-guide.html\""),
        "wikilink using old slug should resolve to new slug URL, got nav section: {}",
        html
    );
}

#[test]
fn test_components_render() {
    let page = r#"# Notes

:::note
This is an important note.
:::
"#;

    let dir = create_project(DEFAULT_CONFIG, &[("index.md", page)]);
    build_project(dir.path()).expect("build should succeed");

    let html = read_output(dir.path(), "index.html");
    assert!(
        html.contains("important note"),
        "note content should appear in output"
    );
    // The note component should produce some admonition-style wrapper
    assert!(
        html.contains("note") && html.contains("class="),
        "note component should render with a CSS class"
    );
}

// ── i18n integration tests ──

const I18N_CONFIG: &str = r#"
[project]
name = "Test Docs"

[locale]
default = "en"
enabled = ["en", "fr"]

[locale.display_names]
en = "English"
fr = "Français"
"#;

#[test]
fn test_i18n_build_output_structure() {
    let dir = create_project(
        I18N_CONFIG,
        &[
            ("index.en.md", "# Welcome\n\nHello world."),
            ("index.fr.md", "# Bienvenue\n\nBonjour le monde."),
            ("guide.en.md", "# Guide\n\nA guide page."),
            ("guide.fr.md", "# Guide\n\nUne page de guide."),
        ],
    );
    build_project(dir.path()).expect("i18n build should succeed");

    // English pages
    assert!(output_exists(dir.path(), "en/index.html"));
    assert!(output_exists(dir.path(), "en/guide.html"));

    // French pages
    assert!(output_exists(dir.path(), "fr/index.html"));
    assert!(output_exists(dir.path(), "fr/guide.html"));

    // Shared assets at root
    assert!(output_exists(dir.path(), "js/docanvil.js"));
    assert!(output_exists(dir.path(), "robots.txt"));
    assert!(output_exists(dir.path(), "sitemap.xml"));
    assert!(output_exists(dir.path(), "404.html"));

    // Per-locale search indexes
    assert!(output_exists(dir.path(), "en/search-index.json"));
    assert!(output_exists(dir.path(), "fr/search-index.json"));

    // Root redirect to default locale
    assert!(
        output_exists(dir.path(), "index.html"),
        "root index.html redirect should be generated when i18n is enabled"
    );
    let root_html = read_output(dir.path(), "index.html");
    assert!(
        root_html.contains("http-equiv=\"refresh\""),
        "root index.html should be a meta-refresh redirect"
    );
    assert!(
        root_html.contains("en/index.html"),
        "root redirect should point to the default locale"
    );

    // Verify page content
    let en_html = read_output(dir.path(), "en/index.html");
    assert!(
        en_html.contains("Hello world"),
        "English content should appear"
    );

    let fr_html = read_output(dir.path(), "fr/index.html");
    assert!(
        fr_html.contains("Bonjour le monde"),
        "French content should appear"
    );
}

#[test]
fn test_i18n_locale_switcher() {
    let dir = create_project(
        I18N_CONFIG,
        &[("index.en.md", "# Welcome"), ("index.fr.md", "# Bienvenue")],
    );
    build_project(dir.path()).expect("build should succeed");

    let en_html = read_output(dir.path(), "en/index.html");
    assert!(
        en_html.contains("locale-switcher"),
        "language switcher should appear in output"
    );
    assert!(
        en_html.contains("English"),
        "English display name should appear"
    );
    assert!(
        en_html.contains("Français"),
        "French display name should appear"
    );
    assert!(
        en_html.contains("lang=\"en\""),
        "HTML lang attribute should be set to en"
    );

    let fr_html = read_output(dir.path(), "fr/index.html");
    assert!(
        fr_html.contains("lang=\"fr\""),
        "HTML lang attribute should be set to fr"
    );
}

#[test]
fn test_i18n_sitemap_includes_all_locales() {
    let dir = create_project(
        I18N_CONFIG,
        &[("index.en.md", "# Welcome"), ("index.fr.md", "# Bienvenue")],
    );
    build_project(dir.path()).expect("build should succeed");

    let sitemap = read_output(dir.path(), "sitemap.xml");
    assert!(
        sitemap.contains("en/index.html"),
        "sitemap should include English pages"
    );
    assert!(
        sitemap.contains("fr/index.html"),
        "sitemap should include French pages"
    );
}

#[test]
fn test_i18n_missing_translation_strict() {
    let config = r#"
[project]
name = "Test Docs"

[locale]
default = "en"
enabled = ["en", "fr"]
"#;

    let dir = create_project(
        config,
        &[
            ("index.en.md", "# Welcome"),
            ("index.fr.md", "# Bienvenue"),
            ("guide.en.md", "# Guide"),
            // guide.fr.md is missing — should produce a warning
        ],
    );

    // Non-strict build should succeed
    build_project(dir.path()).expect("non-strict build should succeed");

    // Strict build should fail due to missing translation warning
    let strict_result = build_project_strict(dir.path());
    assert!(
        strict_result.is_err(),
        "strict build should fail when translations are missing"
    );
}

#[test]
fn test_i18n_unsuffixed_files_get_default_locale() {
    let dir = create_project(
        I18N_CONFIG,
        &[("index.md", "# Welcome"), ("index.fr.md", "# Bienvenue")],
    );
    build_project(dir.path()).expect("build should succeed");

    // Unsuffixed file should be assigned to default locale (en)
    assert!(output_exists(dir.path(), "en/index.html"));
    assert!(output_exists(dir.path(), "fr/index.html"));

    let en_html = read_output(dir.path(), "en/index.html");
    assert!(
        en_html.contains("Welcome"),
        "unsuffixed file should become default locale page"
    );
}

#[test]
fn test_backward_compat_no_locale() {
    // Build without any locale config — should work exactly as before
    let dir = create_project(
        DEFAULT_CONFIG,
        &[
            ("index.md", "# Home\n\nWelcome."),
            ("guide.md", "# Guide\n\nA guide."),
        ],
    );
    build_project(dir.path()).expect("backward-compat build should succeed");

    // Pages at root, not under locale prefixes
    assert!(output_exists(dir.path(), "index.html"));
    assert!(output_exists(dir.path(), "guide.html"));
    assert!(!output_exists(dir.path(), "en/index.html"));

    // Search index at root
    assert!(output_exists(dir.path(), "search-index.json"));

    // No locale switcher dropdown in output (CSS classes may appear in <style>, but no actual element)
    let html = read_output(dir.path(), "index.html");
    assert!(
        !html.contains("data-locale="),
        "locale switcher elements should not appear without i18n config"
    );
}

#[test]
fn test_versioned_build_output_structure() {
    let dir = create_project(
        VERSION_CONFIG,
        &[
            ("v1/index.md", "# Home v1\n\nWelcome to v1."),
            ("v1/guide.md", "# Guide v1\n\nA guide for v1."),
            ("v2/index.md", "# Home v2\n\nWelcome to v2."),
            ("v2/guide.md", "# Guide v2\n\nA guide for v2."),
            ("v2/new-feature.md", "# New Feature\n\nOnly in v2."),
        ],
    );
    build_project(dir.path()).expect("versioned build should succeed");

    // Per-version output directories exist
    assert!(output_exists(dir.path(), "v1/index.html"));
    assert!(output_exists(dir.path(), "v1/guide.html"));
    assert!(output_exists(dir.path(), "v2/index.html"));
    assert!(output_exists(dir.path(), "v2/guide.html"));
    assert!(output_exists(dir.path(), "v2/new-feature.html"));

    // Per-version search indexes
    assert!(output_exists(dir.path(), "v1/search-index.json"));
    assert!(output_exists(dir.path(), "v2/search-index.json"));

    // Root redirect to current version (v2)
    assert!(output_exists(dir.path(), "index.html"));
    let root_html = read_output(dir.path(), "index.html");
    assert!(
        root_html.contains("http-equiv=\"refresh\""),
        "root index.html should be a meta-refresh redirect"
    );
    assert!(
        root_html.contains("v2/index.html"),
        "root redirect should point to current version"
    );

    // Shared assets at root
    assert!(output_exists(dir.path(), "js/docanvil.js"));
    assert!(output_exists(dir.path(), "sitemap.xml"));

    // Version switcher present in current version output — check for the trigger
    // button element, not the CSS class which is always in the stylesheet.
    let v2_html = read_output(dir.path(), "v2/index.html");
    assert!(
        v2_html.contains("version-switcher-trigger"),
        "version switcher button should appear when multiple versions are configured"
    );

    // No older-version banner on current (latest) version — check for the unique
    // rendered text rather than the CSS class which is always present in the stylesheet.
    assert!(
        !v2_html.contains("viewing docs for"),
        "latest version should not show the outdated-version banner"
    );

    // Older-version banner shown on non-current version
    let v1_html = read_output(dir.path(), "v1/index.html");
    assert!(
        v1_html.contains("viewing docs for"),
        "older version should show the outdated-version banner"
    );

    // Page content correct
    assert!(
        v2_html.contains("Welcome to v2"),
        "v2 index should contain v2 content"
    );
    assert!(
        v1_html.contains("Welcome to v1"),
        "v1 index should contain v1 content"
    );
}

#[test]
fn test_versioned_i18n_build_output_structure() {
    let dir = create_project(
        VERSION_I18N_CONFIG,
        &[
            ("v1/index.en.md", "# Home v1 EN\n\nWelcome v1 English."),
            (
                "v1/index.fr.md",
                "# Accueil v1 FR\n\nBienvenue v1 Français.",
            ),
            ("v2/index.en.md", "# Home v2 EN\n\nWelcome v2 English."),
            (
                "v2/index.fr.md",
                "# Accueil v2 FR\n\nBienvenue v2 Français.",
            ),
        ],
    );
    build_project(dir.path()).expect("versioned + i18n build should succeed");

    // Pages exist under version/locale directories
    assert!(output_exists(dir.path(), "v1/en/index.html"));
    assert!(output_exists(dir.path(), "v1/fr/index.html"));
    assert!(output_exists(dir.path(), "v2/en/index.html"));
    assert!(output_exists(dir.path(), "v2/fr/index.html"));

    // Per-version per-locale search indexes
    assert!(output_exists(dir.path(), "v1/en/search-index.json"));
    assert!(output_exists(dir.path(), "v1/fr/search-index.json"));
    assert!(output_exists(dir.path(), "v2/en/search-index.json"));
    assert!(output_exists(dir.path(), "v2/fr/search-index.json"));

    // Root redirect to current version
    assert!(output_exists(dir.path(), "index.html"));
    let root_html = read_output(dir.path(), "index.html");
    assert!(
        root_html.contains("http-equiv=\"refresh\""),
        "root index.html should be a meta-refresh redirect"
    );

    // Content is correct per locale
    let v2_en_html = read_output(dir.path(), "v2/en/index.html");
    assert!(
        v2_en_html.contains("Welcome v2 English"),
        "v2/en should contain English content"
    );

    let v2_fr_html = read_output(dir.path(), "v2/fr/index.html");
    assert!(
        v2_fr_html.contains("Bienvenue v2"),
        "v2/fr should contain French content"
    );
}

#[test]
fn test_rebuild_removes_stale_pages() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[
            ("index.md", "# Home"),
            ("old-page.md", "# Old"),
            ("guides/legacy.md", "# Legacy"),
        ],
    );
    build_project(dir.path()).expect("first build should succeed");
    assert!(output_exists(dir.path(), "old-page.html"));
    assert!(output_exists(dir.path(), "guides/legacy.html"));

    fs::remove_file(dir.path().join("docs/old-page.md")).unwrap();
    fs::remove_dir_all(dir.path().join("docs/guides")).unwrap();
    build_project(dir.path()).expect("rebuild should succeed");

    assert!(output_exists(dir.path(), "index.html"));
    assert!(!output_exists(dir.path(), "old-page.html"));
    assert!(!output_exists(dir.path(), "guides/legacy.html"));
    assert!(!output_exists(dir.path(), "guides"));
    assert!(!dir.path().join(".dist.docanvil-staging").exists());
}

#[test]
fn test_rebuild_keeps_hidden_top_level_entries() {
    let dir = create_project(DEFAULT_CONFIG, &[("index.md", "# Home")]);
    let dist = dir.path().join("dist");
    fs::create_dir_all(dist.join(".git")).unwrap();
    fs::write(dist.join(".git/HEAD"), "ref: refs/heads/gh-pages").unwrap();
    fs::write(dist.join("stray.html"), "stale").unwrap();

    build_project(dir.path()).expect("build should succeed");

    assert!(dist.join(".git/HEAD").exists());
    assert!(!dist.join("stray.html").exists());
    assert!(output_exists(dir.path(), "index.html"));
}

#[test]
fn test_failed_build_leaves_previous_output() {
    let dir = create_project(DEFAULT_CONFIG, &[("index.md", "# Home")]);
    build_project(dir.path()).expect("first build should succeed");

    fs::remove_dir_all(dir.path().join("docs")).unwrap();
    assert!(build_project(dir.path()).is_err());

    assert!(output_exists(dir.path(), "index.html"));
    assert!(!dir.path().join(".dist.docanvil-staging").exists());
}

#[test]
fn test_dev_build_does_not_touch_output_dir() {
    let dir = create_project(DEFAULT_CONFIG, &[("index.md", "# Home")]);
    let dev_out = tempfile::tempdir().unwrap();

    docanvil::cli::build::run_with_options(dir.path(), dev_out.path(), true)
        .expect("dev build should succeed");

    let html = fs::read_to_string(dev_out.path().join("index.html")).unwrap();
    assert!(html.contains("__docanvil_ws"));
    assert!(!dir.path().join("dist").exists());
}

#[test]
fn test_serve_output_dir_is_not_build_output_dir() {
    let dir = create_project(DEFAULT_CONFIG, &[("index.md", "# Home")]);
    let dev_out = docanvil::cli::serve::dev_output_dir(dir.path());
    assert!(!dev_out.starts_with(dir.path()));
}

#[test]
fn test_nav_labels_are_escaped() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[
            ("index.md", "# Home"),
            (
                "generics.md",
                "---\n{\"title\": \"Vec<T> & friends\"}\n---\n# Generics",
            ),
        ],
    );
    build_project(dir.path()).expect("build should succeed");

    let html = read_output(dir.path(), "index.html");
    assert!(html.contains("Vec&lt;T&gt; &amp; friends</a>"));
}

const EDIT_CONFIG: &str = r#"
[project]
name = "Test Docs"

[edit]
repo = "https://github.com/org/repo"
root = "site"
"#;

/// Read an output page with Tera's `&#x2F;` escaping of URLs undone.
fn read_page(dir: &std::path::Path, path: &str) -> String {
    read_output(dir, path).replace("&#x2F;", "/")
}

#[test]
fn test_edit_link_points_at_source() {
    let dir = create_project(
        EDIT_CONFIG,
        &[("index.md", "# Home"), ("guide/setup.md", "# Setup")],
    );
    build_project(dir.path()).unwrap();

    let html = read_page(dir.path(), "guide/setup.html");
    assert!(html.contains("Edit this page"));
    assert!(
        html.contains(r#"href="https://github.com/org/repo/edit/main/site/docs/guide/setup.md""#)
    );
    assert!(html.contains(r#"target="_blank" rel="noopener""#));

    let not_found = read_output(dir.path(), "404.html");
    assert!(!not_found.contains("Edit this page"));
}

#[test]
fn test_edit_link_off_by_default() {
    let dir = create_project(DEFAULT_CONFIG, &[("index.md", "# Home")]);
    build_project(dir.path()).unwrap();
    assert!(!read_output(dir.path(), "index.html").contains("Edit this page"));
}

#[test]
fn test_edit_link_front_matter_opt_out() {
    let dir = create_project(
        EDIT_CONFIG,
        &[
            ("index.md", "# Home"),
            ("private.md", "---\n{\"edit_link\": false}\n---\n# Private"),
        ],
    );
    build_project(dir.path()).unwrap();
    assert!(read_output(dir.path(), "index.html").contains("Edit this page"));
    assert!(!read_output(dir.path(), "private.html").contains("Edit this page"));
}

#[test]
fn test_edit_link_uses_locale_source_file() {
    let config = format!("{EDIT_CONFIG}\n[locale]\ndefault = \"en\"\nenabled = [\"en\", \"fr\"]\n");
    let dir = create_project(
        &config,
        &[("index.en.md", "# Welcome"), ("index.fr.md", "# Bienvenue")],
    );
    build_project(dir.path()).unwrap();
    assert!(
        read_page(dir.path(), "fr/index.html")
            .contains("https://github.com/org/repo/edit/main/site/docs/index.fr.md")
    );
    assert!(
        read_page(dir.path(), "en/index.html")
            .contains("https://github.com/org/repo/edit/main/site/docs/index.en.md")
    );
}

#[test]
fn test_edit_link_uses_version_source_file() {
    let config =
        format!("{EDIT_CONFIG}\n[version]\ncurrent = \"v2\"\nenabled = [\"v1\", \"v2\"]\n");
    let dir = create_project(
        &config,
        &[("v1/guide.md", "# Old guide"), ("v2/guide.md", "# Guide")],
    );
    build_project(dir.path()).unwrap();
    assert!(
        read_page(dir.path(), "v1/guide.html")
            .contains("https://github.com/org/repo/edit/main/site/docs/v1/guide.md")
    );
}

#[test]
fn test_edit_link_unknown_host_fails_strict() {
    let config = |repo: &str| {
        format!(
            "[project]\nname = \"Test Docs\"\n\n[build]\nsite_url = \"https://docs.example.com\"\n\n[edit]\nrepo = \"{repo}\"\nroot = \"\"\n"
        )
    };

    // Control: a recognised host builds cleanly in strict mode.
    let ok = create_project(
        &config("https://github.com/org/repo"),
        &[("index.md", "# Home")],
    );
    build_project_strict(ok.path()).unwrap();

    let dir = create_project(
        &config("https://git.example.com/team/docs"),
        &[("index.md", "# Home")],
    );
    assert!(build_project_strict(dir.path()).is_err());
    // A normal build still succeeds, just without links.
    build_project(dir.path()).unwrap();
    assert!(!read_output(dir.path(), "index.html").contains("Edit this page"));
}

#[test]
fn test_template_components_custom_and_override() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[
            (
                "index.md",
                "# Home\n\n::::card{title=\"Start here\"}\n:::note\nSee [[other]].\n:::\n\n```rust\nfn main() {}\n```\n::::\n",
            ),
            ("other.md", "# Other\n"),
        ],
    );
    let comps = dir.path().join("theme/components");
    std::fs::create_dir_all(&comps).unwrap();
    std::fs::write(
        comps.join("card.html"),
        "<section class=\"card\">\n\n  <h3>{{ attrs.title }}</h3>\n\n  {{ body | safe }}\n\n</section>\n",
    )
    .unwrap();
    std::fs::write(
        comps.join("note.html"),
        "<aside class=\"my-note\">{{ body | safe }}</aside>",
    )
    .unwrap();

    build_project(dir.path()).unwrap();
    let html = fs::read_to_string(dir.path().join("dist/index.html")).unwrap();

    assert!(html.contains("<section class=\"card\">"), "card rendered");
    // The pipeline's auto heading-ID stage (pipeline/headings.rs) runs over the whole
    // page, including component-rendered markup, so the `<h3>` picks up an id.
    assert!(
        html.contains("<h3 id=\"start-here\">Start here</h3>"),
        "title rendered, not a code block"
    );
    assert!(
        html.contains("<aside class=\"my-note\">"),
        "note override used"
    );
    let note_start = html.find("<aside class=\"my-note\">").unwrap();
    let note_body = &html[note_start..];
    let note_end = note_body.find("</aside>").unwrap();
    let note_body = &note_body[..note_end];
    assert!(
        note_body.contains("href="),
        "wiki-link inside nested component resolved to a link"
    );
    assert!(
        note_body.contains("other.html"),
        "wiki-link inside nested component resolved to other.html"
    );
    assert!(!html.contains("[[other]]"), "no unresolved wiki-link");
    assert!(!html.contains("da-component"), "no leftover placeholders");
}

#[test]
fn test_template_component_syntax_error_fails_build() {
    let dir = create_project(DEFAULT_CONFIG, &[("index.md", "# Home\n")]);
    let comps = dir.path().join("theme/components");
    std::fs::create_dir_all(&comps).unwrap();
    std::fs::write(comps.join("card.html"), "{% if %}").unwrap();
    let err = build_project(dir.path()).unwrap_err();
    assert!(
        matches!(err, docanvil::error::Error::ComponentTemplate { .. }),
        "{err}"
    );
}

#[test]
fn test_template_component_render_error_fails_strict() {
    let dir = create_project(DEFAULT_CONFIG, &[("index.md", "# Home\n\n:::card\n:::\n")]);
    let comps = dir.path().join("theme/components");
    std::fs::create_dir_all(&comps).unwrap();
    std::fs::write(comps.join("card.html"), "<h3>{{ attrs.title }}</h3>").unwrap();
    assert!(
        build_project(dir.path()).is_ok(),
        "non-strict build succeeds"
    );
    assert!(
        build_project_strict(dir.path()).is_err(),
        "strict build fails on the render warning"
    );
}

#[test]
fn test_directive_examples_in_code_fences_stay_literal() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[(
            "index.md",
            "# Home\n\n```markdown\n:::note{title=\"Hi\"}\nBody\n:::\n```\n",
        )],
    );
    build_project(dir.path()).unwrap();
    let html = fs::read_to_string(dir.path().join("dist/index.html")).unwrap();
    assert!(
        !html.contains("admonition-title\">Hi"),
        "example was rendered as a component"
    );
    assert!(html.contains(":::note"), "example source shown");
}

#[test]
fn test_breadcrumbs_on_nested_pages_only() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[
            ("index.md", "# Home"),
            ("guide/index.md", "# Guide"),
            ("guide/setup.md", "# Setup"),
        ],
    );
    build_project(dir.path()).unwrap();

    let setup = read_page(dir.path(), "guide/setup.html");
    assert!(setup.contains(r#"<nav class="breadcrumbs" aria-label="Breadcrumb">"#));
    assert!(setup.contains(r#"<li aria-current="page">Setup</li>"#));
    assert!(setup.contains(r#"<a href="/guide/index.html">"#));

    let home = read_page(dir.path(), "index.html");
    assert!(!home.contains(r#"aria-label="Breadcrumb""#));
}

#[test]
fn test_breadcrumbs_use_locale_urls() {
    let config =
        format!("{DEFAULT_CONFIG}\n[locale]\ndefault = \"en\"\nenabled = [\"en\", \"fr\"]\n");
    let dir = create_project(
        &config,
        &[
            ("guide/index.en.md", "# Guide"),
            ("guide/setup.en.md", "# Setup"),
            ("guide/index.fr.md", "# Guide"),
            ("guide/setup.fr.md", "# Installation"),
        ],
    );
    build_project(dir.path()).unwrap();
    let fr = read_page(dir.path(), "fr/guide/setup.html");
    assert!(fr.contains(r#"<a href="/fr/guide/index.html">"#));
    assert!(fr.contains(r#"<li aria-current="page">Installation</li>"#));
}

#[test]
fn test_description_shown_under_title() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[
            (
                "index.md",
                "---\n{\"description\": \"Docs that <ship> fast\"}\n---\n# Home\n\nWelcome.",
            ),
            ("plain.md", "# Plain"),
        ],
    );
    build_project(dir.path()).unwrap();

    let home = read_output(dir.path(), "index.html");
    let h1_end = home.find("</h1>").unwrap();
    let desc = home
        .find(r#"<p class="page-description">Docs that &lt;ship&gt; fast</p>"#)
        .expect("description subtitle missing");
    assert!(desc > h1_end);
    assert!(desc < home.find("Welcome.").unwrap());

    assert!(!read_output(dir.path(), "plain.html").contains(r#"<p class="page-description">"#));
}

#[test]
fn test_title_from_h1_keeps_filename_slug() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[
            ("index.md", "# Home"),
            ("guide/setup.md", "# Installing `docanvil`\n\nBody."),
            (
                "guide/short.md",
                "---\n{\"title\": \"Short\", \"slug\": \"short\"}\n---\n# A Much Longer Heading",
            ),
        ],
    );
    build_project(dir.path()).unwrap();

    // The H1 names the page everywhere, but the URL still comes from the filename.
    let setup = read_page(dir.path(), "guide/setup.html");
    assert!(setup.contains("<title>Installing docanvil — Test Docs</title>"));
    assert!(setup.contains(r#"<li aria-current="page">Installing docanvil</li>"#));

    // A front matter title still wins over the H1.
    let short = read_page(dir.path(), "guide/short.html");
    assert!(short.contains("<title>Short — Test Docs</title>"));
}

#[test]
fn test_home_title_not_repeated_when_h1_is_project_name() {
    let dir = create_project(DEFAULT_CONFIG, &[("index.md", "# Test Docs\n\nWelcome.")]);
    build_project(dir.path()).unwrap();
    assert!(read_output(dir.path(), "index.html").contains("<title>Test Docs</title>"));
}

const STRICT_CONFIG: &str =
    "[project]\nname = \"Test Docs\"\n\n[build]\nsite_url = \"https://docs.example.com\"\n";

fn write_file(root: &std::path::Path, path: &str, content: &str) {
    let full = root.join(path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, content).unwrap();
}

fn strip_tags(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

#[test]
fn test_includes_and_file_blocks() {
    let dir = create_project(
        STRICT_CONFIG,
        &[
            (
                "index.md",
                "# Home\n\n:::include{file=\"_shared/install.md\"}\n\n```rust file=\"/examples/server.rs\" lines=\"1-2,5-6\"\n```\n\n:::code-group\n```rust file=\"/examples/server.rs\" lines=\"5-6\"\n```\n```text\nplain\n```\n:::\n",
            ),
            (
                "_shared/install.md",
                "## Install\n\nRun the installer, then restart.\n",
            ),
        ],
    );
    write_file(
        dir.path(),
        "examples/server.rs",
        "fn start() {\n    listen();\n}\n\nfn stop() {\n    close();\n}\n",
    );
    build_project_strict(dir.path()).expect("strict build should succeed");

    assert!(!output_exists(dir.path(), "_shared/install.html"));
    let html = read_output(dir.path(), "index.html");
    assert!(
        html.contains("id=\"install\""),
        "fragment heading gets an id for the TOC"
    );
    assert!(html.contains("Run the installer"));
    assert!(html.contains("data-hidden=\"2\""), "lines 3–4 are hidden");
    assert!(html.contains("<span class=\"code-block-lines\">lines 1–2, 5–6</span>"));
    assert!(!html.contains("data-meta"));
    assert!(
        html.contains(">rust</button>"),
        "code group tab shows the language"
    );
    assert!(html.contains("<span class=\"line\" data-line=\"5\">"));

    // What the copy button copies (textContent) is only the code.
    let figure = html.find("<figure class=\"code-block\">").unwrap();
    let pre_start = figure + html[figure..].find("<pre").unwrap();
    let pre_end = pre_start + html[pre_start..].find("</pre>").unwrap();
    assert_eq!(
        strip_tags(&html[pre_start..pre_end]).trim_start_matches('\n'),
        "fn start() {\n    listen();\nfn stop() {\n    close();\n"
    );

    let index = read_output(dir.path(), "search-index.json");
    assert!(
        index.contains("Run the installer"),
        "search indexes fragment text"
    );
}

#[test]
fn test_includes_with_locales_and_versions() {
    let page = |title: &str| {
        format!(
            "# {title}\n\n:::include{{file=\"_shared/note.md\"}}\n\n:::include{{file=\"/_shared/footer.md\"}}\n"
        )
    };
    let dir = create_project(
        VERSION_I18N_CONFIG,
        &[
            ("v1/index.md", page("Home v1").as_str()),
            ("v1/_shared/note.md", "Version one note.\n"),
            ("v2/index.md", page("Home v2").as_str()),
            ("v2/index.fr.md", page("Accueil v2").as_str()),
            ("v2/_shared/note.md", "Version two note.\n"),
            ("v2/_shared/note.fr.md", "Note de la version deux.\n"),
        ],
    );
    write_file(dir.path(), "_shared/footer.md", "Shared footer.\n");
    build_project(dir.path()).expect("build should succeed");

    let v1 = read_output(dir.path(), "v1/en/index.html");
    assert!(v1.contains("Version one note.") && v1.contains("Shared footer."));
    let v2 = read_output(dir.path(), "v2/en/index.html");
    assert!(v2.contains("Version two note.") && v2.contains("Shared footer."));
    let fr = read_output(dir.path(), "v2/fr/index.html");
    assert!(fr.contains("Note de la version deux.") && fr.contains("Shared footer."));
    assert!(!fr.contains("Version two note."));
    assert!(!output_exists(dir.path(), "v2/en/_shared/note.html"));
}

#[test]
fn test_strict_fails_on_include_problems() {
    let missing = create_project(
        STRICT_CONFIG,
        &[("index.md", "# Home\n\n:::include{file=\"_nope.md\"}\n")],
    );
    assert!(matches!(
        build_project_strict(missing.path()),
        Err(docanvil::error::Error::StrictWarnings(_))
    ));
    build_project(missing.path()).expect("a normal build still succeeds");
    assert!(read_output(missing.path(), "index.html").contains("<div class=\"include-error\">"));

    let stale = create_project(
        STRICT_CONFIG,
        &[(
            "index.md",
            "# Home\n\n```rust file=\"/a.rs\" lines=\"5-9\"\n```\n",
        )],
    );
    write_file(stale.path(), "a.rs", "fn a() {}\n");
    assert!(matches!(
        build_project_strict(stale.path()),
        Err(docanvil::error::Error::StrictWarnings(_))
    ));
}

#[test]
fn test_include_errors_never_show_absolute_paths() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[(
            "index.md",
            "# Home\n\n:::include{file=\"_nope.md\"}\n\n```rust file=\"/missing.rs\"\n```\n",
        )],
    );
    build_project(dir.path()).expect("build should succeed");
    let html = read_output(dir.path(), "index.html");
    assert_eq!(html.matches("<div class=\"include-error\">").count(), 2);
    let root = dir.path().canonicalize().unwrap();
    for path in [dir.path(), root.as_path()] {
        let path = path.to_string_lossy();
        assert!(!html.contains(&*path), "page shows {path}");
    }
}

#[test]
fn test_underscore_files_are_not_pages() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[
            ("index.md", "# Home"),
            ("_draft.md", "# Draft"),
            ("_notes/idea.md", "# Idea"),
        ],
    );
    build_project(dir.path()).expect("build should succeed");
    assert!(!output_exists(dir.path(), "_draft.html"));
    assert!(!output_exists(dir.path(), "_notes/idea.html"));
    assert!(!read_output(dir.path(), "index.html").contains("_draft.html"));
}

#[test]
fn test_dev_build_reports_include_dependencies() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[(
            "index.md",
            "# Home\n\n```rust file=\"/examples/a.rs\"\n```\n",
        )],
    );
    write_file(dir.path(), "examples/a.rs", "fn a() {}\n");
    let dev_out = tempfile::tempdir().unwrap();
    let deps = docanvil::cli::build::run_with_options(dir.path(), dev_out.path(), true).unwrap();
    assert!(deps.contains(&dir.path().join("examples/a.rs").canonicalize().unwrap()));
}
