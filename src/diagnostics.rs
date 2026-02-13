use std::path::Path;

use owo_colors::OwoColorize;

/// Emit a warning about a broken wiki-link.
pub fn warn_broken_link(source_file: &Path, link_target: &str) {
    eprintln!(
        "{}: broken link [[{}]] in {}",
        "warning".yellow().bold(),
        link_target,
        source_file.display()
    );
}

/// Emit a warning about a nav.toml entry referencing a page that doesn't exist.
pub fn warn_nav_missing_page(slug: &str) {
    eprintln!(
        "{}: nav.toml references page '{}' which does not exist",
        "warning".yellow().bold(),
        slug
    );
}

/// Emit a warning that site_url is not configured (sitemap will use relative URLs).
pub fn warn_no_site_url() {
    eprintln!(
        "{}: site_url not set in [build] — sitemap.xml will use relative URLs",
        "warning".yellow().bold()
    );
}

/// Emit a warning about an unclosed directive.
pub fn warn_unclosed_directive(source_file: &Path, directive_name: &str, line_number: usize) {
    eprintln!(
        "{}: unclosed directive ':::{}' opened at line {} in {}",
        "warning".yellow().bold(),
        directive_name,
        line_number,
        source_file.display()
    );
}
