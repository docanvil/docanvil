mod integration_helpers;

use integration_helpers::{
    DEFAULT_CONFIG, build_project, build_project_strict, create_project, output_exists, read_output,
};

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
        &html
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
