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
}

/// Emit a warning about a nav.toml entry referencing a page that doesn't exist.
pub fn warn_nav_missing_page(slug: &str) {
    increment();
    eprintln!(
        "{}: nav.toml references page '{}' which does not exist",
        "warning".yellow().bold(),
        slug
    );
}

/// Emit a warning that site_url is not configured (sitemap will use relative URLs).
pub fn warn_no_site_url() {
    increment();
    eprintln!(
        "{}: site_url not set in [build] — sitemap.xml will use relative URLs",
        "warning".yellow().bold()
    );
}

/// Emit a warning about an unclosed directive.
pub fn warn_unclosed_directive(source_file: &Path, directive_name: &str, line_number: usize) {
    increment();
    eprintln!(
        "{}: unclosed directive ':::{}' opened at line {} in {}",
        "warning".yellow().bold(),
        directive_name,
        line_number,
        source_file.display()
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
}
