use std::cell::Cell;
use std::path::Path;

use owo_colors::OwoColorize;

use crate::pipeline::includes::display_path;
use crate::redirects::{Origin, RedirectProblem};

// Thread-local so each build counts only its own warnings. A build runs start to
// finish on one thread, so concurrent builds (parallel tests, the dev server's
// rebuilds) can't reset or inflate each other's counts.
thread_local! {
    static WARNING_COUNT: Cell<usize> = const { Cell::new(0) };
}

fn increment() {
    WARNING_COUNT.with(|c| c.set(c.get() + 1));
}

/// Return the number of warnings emitted on this thread since the last reset.
pub fn warning_count() -> usize {
    WARNING_COUNT.with(Cell::get)
}

/// Reset this thread's warning counter to zero.
pub fn reset_warnings() {
    WARNING_COUNT.with(|c| c.set(0));
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

/// Emit a warning about a wiki-link to a draft page that this build leaves out
/// (only when `[build] draft_links = "warn"`).
pub fn warn_draft_link(source_file: &Path, link_target: &str) {
    increment();
    eprintln!(
        "{}: [[{}]] in {} links to a draft page, so it's shown as plain text",
        "warning".yellow().bold(),
        link_target,
        source_file.display()
    );
    eprintln!(
        "  {}: Publish the page (remove \"draft\": true) or remove the link.",
        "hint".dimmed()
    );
}

/// Emit a warning about an `:::include` or `file="…"` code block that couldn't be filled in.
/// `file` is canonical; it's shown relative to `project_root` when it's inside it.
pub fn warn_include(
    project_root: &Path,
    file: &Path,
    line: usize,
    message: &str,
    hint: Option<&str>,
) {
    increment();
    eprintln!(
        "{}: {} ({}:{})",
        "warning".yellow().bold(),
        message,
        display_path(project_root, file).display(),
        line
    );
    eprintln!(
        "  {}: {}",
        "hint".dimmed(),
        hint.unwrap_or("Run 'docanvil doctor' to check every include and code block file.")
    );
}

/// Emit a warning about a redirect that can't be written as declared.
pub fn warn_redirect(project_root: &Path, problem: &RedirectProblem) {
    increment();
    let place = match &problem.origin {
        Origin::FrontMatter(path) => display_path(project_root, path).display().to_string(),
        Origin::Table(_) | Origin::Unprefixed => "docanvil.toml".to_string(),
    };
    eprintln!(
        "{}: {} ({})",
        "warning".yellow().bold(),
        problem.message,
        place
    );
    eprintln!(
        "  {}: Run 'docanvil doctor' to check every redirect.",
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

/// Emit a warning that the `[edit]` config can't produce "Edit this page" links.
pub fn warn_edit_link_config(message: &str) {
    increment();
    eprintln!("{}: {message}", "warning".yellow().bold());
    eprintln!(
        "  {}: Edit links are turned off for this build. Run `docanvil doctor` for details.",
        "hint".dimmed()
    );
}

/// Emit a warning that `[project] repo` can't be linked from the site header.
pub fn warn_repo_link_config(message: &str) {
    increment();
    eprintln!("{}: {message}", "warning".yellow().bold());
    eprintln!(
        "  {}: The header repository link is left out of this build.",
        "hint".dimmed()
    );
}

/// Emit a warning about `serve --editor` / `DOCANVIL_EDITOR`. Shown once when the
/// dev server starts, so it isn't counted as a build warning.
pub fn warn_editor(message: &str) {
    eprintln!("{}: {message}", "warning".yellow().bold());
}

/// Emit a warning that Git history can't be read for "last updated" dates.
pub fn warn_last_updated_no_git(message: &str) {
    increment();
    eprintln!("{}: {message}", "warning".yellow().bold());
    eprintln!(
        "  {}: Pages only show front matter `last_updated` dates. Set source = \"front-matter\" under [last_updated] to skip Git.",
        "hint".dimmed()
    );
}

/// Emit a warning that a shallow clone makes every page look last changed in the newest commit.
pub fn warn_last_updated_shallow() {
    increment();
    eprintln!(
        "{}: [last_updated] this is a shallow Git clone, so every page shows the date of the latest commit",
        "warning".yellow().bold()
    );
    eprintln!(
        "  {}: Fetch the full history, e.g. `fetch-depth: 0` on GitHub's actions/checkout.",
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

/// Emit a warning that a component template failed to render on a page.
pub fn warn_component_render(source_file: &Path, name: &str, message: &str) {
    increment();
    eprintln!(
        "{}: couldn't render :::{} in {}: {}",
        "warning".yellow().bold(),
        name,
        source_file.display(),
        message
    );
    if message.contains("not found in context") {
        eprintln!(
            "  {}: Optional attributes need a fallback, e.g. {{{{ attrs.title | default(value=\"…\") }}}}",
            "hint".dimmed()
        );
    } else {
        eprintln!(
            "  {}: Check the template, or run 'docanvil doctor' to validate component templates.",
            "hint".dimmed()
        );
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warning_count_is_isolated_per_thread() {
        reset_warnings();
        warn_no_site_url();

        // A concurrent build on another thread must not reset or inflate our count.
        std::thread::spawn(|| {
            reset_warnings();
            warn_no_site_url();
            warn_no_site_url();
        })
        .join()
        .unwrap();

        assert_eq!(warning_count(), 1);
    }
}
