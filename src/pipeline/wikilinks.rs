use std::path::Path;

use crate::diagnostics;
use crate::project::PageInventory;

/// Process wiki-links in rendered HTML.
/// Replaces `[[target]]` and `[[target|display text]]` with proper HTML links.
pub fn resolve(html: &str, inventory: &PageInventory, source_file: &Path, base_url: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut remaining = html;

    while let Some(start) = remaining.find("[[") {
        result.push_str(&remaining[..start]);
        let after_open = &remaining[start + 2..];

        if let Some(end) = after_open.find("]]") {
            let inner = &after_open[..end];
            let (target, display) = if let Some(pipe_pos) = inner.find('|') {
                (&inner[..pipe_pos], &inner[pipe_pos + 1..])
            } else {
                (inner, inner)
            };

            let target = target.trim();
            let display = display.trim();

            if let Some(page) = inventory.resolve_link(target) {
                let href = format!("{}{}", base_url, page.output_path.display());
                result.push_str(&format!("<a href=\"{href}\">{display}</a>"));
            } else {
                diagnostics::warn_broken_link(source_file, target);
                result.push_str(&format!(
                    "<span class=\"broken-link popover-trigger\" tabindex=\"0\">\
                     {display}\
                     <span class=\"popover-content popover-error\" role=\"tooltip\">\
                     <strong>Page not found</strong><br />
                     The linked page doesn't exist: <code>{target}</code></span>\
                     </span>"
                ));
            }

            remaining = &after_open[end + 2..];
        } else {
            // No closing ]], output as-is
            result.push_str("[[");
            remaining = after_open;
        }
    }

    result.push_str(remaining);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::PageInventory;
    use std::fs;

    fn test_inventory() -> (tempfile::TempDir, PageInventory) {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.md"), "# Home").unwrap();
        fs::write(docs.join("setup.md"), "# Setup").unwrap();
        let inv = PageInventory::scan(&docs).unwrap();
        (dir, inv)
    }

    #[test]
    fn resolve_simple_link() {
        let (_dir, inv) = test_inventory();
        let html = "<p>See [[setup]] for details.</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/");
        assert!(result.contains("<a href=\"/setup.html\">setup</a>"));
    }

    #[test]
    fn resolve_link_with_display_text() {
        let (_dir, inv) = test_inventory();
        let html = "<p>See [[setup|the setup guide]] for details.</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/");
        assert!(result.contains("<a href=\"/setup.html\">the setup guide</a>"));
    }

    #[test]
    fn broken_link_gets_class() {
        let (_dir, inv) = test_inventory();
        let html = "<p>See [[nonexistent]] page.</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/");
        assert!(result.contains("class=\"broken-link popover-trigger\""));
        assert!(result.contains("popover-error"));
        assert!(result.contains("<code>nonexistent</code>"));
        assert!(result.contains("Page not found"));
    }

    #[test]
    fn unclosed_brackets_preserved() {
        let (_dir, inv) = test_inventory();
        let html = "<p>This [[ is unclosed.</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/");
        assert!(result.contains("[["));
    }
}
