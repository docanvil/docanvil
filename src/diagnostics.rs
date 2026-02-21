use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use owo_colors::OwoColorize;

static WARNING_COUNT: AtomicUsize = AtomicUsize::new(0);

fn increment() {
    WARNING_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Return the number of warnings emitted since the last reset.
pub fn warning_count() -> usize {
    WARNING_COUNT.load(Ordering::Relaxed)
}

/// Reset the warning counter to zero.
pub fn reset_warnings() {
    WARNING_COUNT.store(0, Ordering::Relaxed);
}

/// Emit a warning about a broken wiki-link.
pub fn warn_broken_link(source_file: &Path, link_target: &str) {
    increment();
    eprintln!(
        "{}: broken link [[{}]] in {}",
        "warning".yellow().bold(),
        link_target,
        source_file.display()
    );
    eprintln!(
        "  {}: Run 'docanvil doctor' to check all links.",
        "hint".dimmed()
    );
}

/// Emit a warning about a nav.toml entry referencing a page that doesn't exist.
pub fn warn_nav_missing_page(slug: &str) {
    increment();
    eprintln!(
        "{}: nav.toml references page '{}' which does not exist",
        "warning".yellow().bold(),
        slug
    );
    eprintln!(
        "  {}: Check nav.toml or run 'docanvil doctor' for details.",
        "hint".dimmed()
    );
}

/// Emit a warning that site_url is not configured (sitemap will use relative URLs).
pub fn warn_no_site_url() {
    increment();
    eprintln!(
        "{}: site_url not set in [build] — sitemap.xml will use relative URLs",
        "warning".yellow().bold()
    );
    eprintln!(
        "  {}: Add site_url to [build] in docanvil.toml for absolute URLs.",
        "hint".dimmed()
    );
}

/// Emit a warning that an autodiscover folder has no matching pages.
pub fn warn_nav_autodiscover_empty(folder: &str) {
    increment();
    eprintln!(
        "{}: nav.toml autodiscover folder '{}' matches no pages",
        "warning".yellow().bold(),
        folder
    );
    eprintln!(
        "  {}: Check the folder path in nav.toml or add pages to it.",
        "hint".dimmed()
    );
}

/// Emit a warning about a malformed HTML tag that prevented attribute injection.
pub fn warn_malformed_attribute_tag() {
    increment();
    eprintln!(
        "{}: malformed HTML tag — could not inject attribute block",
        "warning".yellow().bold(),
    );
    eprintln!(
        "  {}: Check your Markdown for unclosed HTML tags. Run 'docanvil doctor' for details.",
        "hint".dimmed()
    );
}

/// Emit a warning about an unexpected asset path (e.g. from symlinks).
pub fn warn_unexpected_asset_path(path: &Path) {
    increment();
    eprintln!(
        "{}: unexpected asset path {} — skipping file",
        "warning".yellow().bold(),
        path.display()
    );
    eprintln!(
        "  {}: This may be caused by symlinks. Check your assets directory for symlinks pointing outside the project.",
        "hint".dimmed()
    );
}

/// Emit a warning about an unexpected content path (e.g. from symlinks).
pub fn warn_unexpected_content_path(path: &Path) {
    increment();
    eprintln!(
        "{}: unexpected content path {} — skipping file",
        "warning".yellow().bold(),
        path.display()
    );
    eprintln!(
        "  {}: This may be caused by symlinks in your content directory. Run 'docanvil doctor' to check your project structure.",
        "hint".dimmed()
    );
}

/// Emit a warning about a missing translation for a page in a locale.
pub fn warn_missing_translation(slug: &str, locale: &str) {
    increment();
    eprintln!(
        "{}: page '{}' has no translation for locale '{}'",
        "warning".yellow().bold(),
        slug,
        locale
    );
    eprintln!(
        "  {}: Create a file with the '.{}.md' suffix to add a translation.",
        "hint".dimmed(),
        locale
    );
}

/// Emit a warning that a custom CSS file was not found.
pub fn warn_custom_css_not_found(path: &str) {
    increment();
    eprintln!(
        "{}: custom_css file not found: {}",
        "warning".yellow().bold(),
        path
    );
    eprintln!(
        "  {}: Check the path in docanvil.toml, or run 'docanvil doctor --fix' to create it.",
        "hint".dimmed()
    );
}
