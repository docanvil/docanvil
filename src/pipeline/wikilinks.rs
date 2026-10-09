use std::path::Path;

use crate::config::DraftLinks;
use crate::diagnostics;
use crate::project::PageInventory;

/// Process wiki-links in rendered HTML.
/// Replaces `[[target]]` and `[[target|display text]]` with proper HTML links.
/// When `locale` is provided, links resolve within that locale only.
///
/// Code is left untouched: `<pre>…</pre>` blocks (highlighted blocks have no `<code>`
/// wrapper; unhighlighted ones are `<pre><code>…</code></pre>`) and inline `<code>…</code>`
/// spans are copied through verbatim, so `[[…]]` inside code (e.g. TOML's `[[nav]]`) is
/// never rewritten.
pub fn resolve(
    html: &str,
    inventory: &PageInventory,
    source_file: &Path,
    base_url: &str,
    locale: Option<&str>,
) -> String {
    let mut result = String::with_capacity(html.len());
    let mut remaining = html;

    while !remaining.is_empty() {
        match next_code_span(remaining) {
            Some((before, code, after)) => {
                result.push_str(&resolve_segment(
                    before,
                    inventory,
                    source_file,
                    base_url,
                    locale,
                ));
                result.push_str(code);
                remaining = after;
            }
            None => {
                result.push_str(&resolve_segment(
                    remaining,
                    inventory,
                    source_file,
                    base_url,
                    locale,
                ));
                remaining = "";
            }
        }
    }

    result
}

/// Find the next `<pre>…</pre>` or inline `<code>…</code>` span in `html`.
/// Returns `(before, span, after)` where `span` includes the tags themselves.
/// A `<pre>` without a matching `</pre>` (or `<code>` without `</code>`) is treated
/// as plain text from that point on, since there's nothing safe to skip past.
fn next_code_span(html: &str) -> Option<(&str, &str, &str)> {
    let pre_pos = html.find("<pre");
    let code_pos = html.find("<code");

    let start = match (pre_pos, code_pos) {
        (Some(p), Some(c)) => p.min(c),
        (Some(p), None) => p,
        (None, Some(c)) => c,
        (None, None) => return None,
    };

    let is_pre = pre_pos == Some(start);
    let close_tag = if is_pre { "</pre>" } else { "</code>" };

    let search_from = &html[start..];
    let close_rel = search_from.find(close_tag)?;
    let end = start + close_rel + close_tag.len();

    Some((&html[..start], &html[start..end], &html[end..]))
}

/// Resolve `[[…]]` wiki-links within a segment known to contain no code spans.
fn resolve_segment(
    html: &str,
    inventory: &PageInventory,
    source_file: &Path,
    base_url: &str,
    locale: Option<&str>,
) -> String {
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

            let resolved = match locale {
                Some(l) => inventory.resolve_link_in_locale(target, l),
                None => inventory.resolve_link(target),
            };
            if let Some(page) = resolved {
                let href = format!("{}{}", base_url, page.output_path.display());
                result.push_str(&format!("<a href=\"{href}\">{display}</a>"));
            } else if inventory.resolve_draft(target, locale).is_some() {
                // The page exists but this build leaves it out: keep the words, drop the link.
                if inventory.draft_links == DraftLinks::Warn {
                    diagnostics::warn_draft_link(source_file, target);
                }
                result.push_str(display);
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
        let inv = PageInventory::scan(&docs, None, None, None).unwrap();
        (dir, inv)
    }

    #[test]
    fn resolve_simple_link() {
        let (_dir, inv) = test_inventory();
        let html = "<p>See [[setup]] for details.</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert!(result.contains("<a href=\"/setup.html\">setup</a>"));
    }

    #[test]
    fn resolve_link_with_display_text() {
        let (_dir, inv) = test_inventory();
        let html = "<p>See [[setup|the setup guide]] for details.</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert!(result.contains("<a href=\"/setup.html\">the setup guide</a>"));
    }

    #[test]
    fn broken_link_gets_class() {
        let (_dir, inv) = test_inventory();
        let html = "<p>See [[nonexistent]] page.</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert!(result.contains("class=\"broken-link popover-trigger\""));
        assert!(result.contains("popover-error"));
        assert!(result.contains("<code>nonexistent</code>"));
        assert!(result.contains("Page not found"));
    }

    #[test]
    fn draft_link_renders_as_text() {
        let (_dir, mut inv) = test_inventory();
        inv.exclude_drafts(&["setup".to_string()], DraftLinks::Text);
        diagnostics::reset_warnings();
        let html = "<p>See [[setup|the setup guide]].</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert_eq!(result, "<p>See the setup guide.</p>");
        assert_eq!(diagnostics::warning_count(), 0);
    }

    #[test]
    fn draft_link_warns_when_configured() {
        let (_dir, mut inv) = test_inventory();
        inv.exclude_drafts(&["setup".to_string()], DraftLinks::Warn);
        diagnostics::reset_warnings();
        let result = resolve("<p>[[setup]]</p>", &inv, Path::new("test.md"), "/", None);
        assert_eq!(result, "<p>setup</p>");
        assert_eq!(diagnostics::warning_count(), 1);
    }

    #[test]
    fn unclosed_brackets_preserved() {
        let (_dir, inv) = test_inventory();
        let html = "<p>This [[ is unclosed.</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert!(result.contains("[["));
    }

    #[test]
    fn wikilink_syntax_in_highlighted_pre_block_untouched() {
        // Highlighted code blocks lose their <code> wrapper and gain a style attribute.
        let (_dir, inv) = test_inventory();
        let html = r#"<pre style="background-color:#fff;">[[nav]]
page = "x"
</pre>"#;
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert_eq!(result, html);
        assert!(!result.contains("broken-link"));
    }

    #[test]
    fn wikilink_syntax_in_plain_pre_code_block_untouched() {
        // Unhighlighted fenced blocks keep the <pre><code> wrapper.
        let (_dir, inv) = test_inventory();
        let html = "<pre><code>[[nav]]\npage = \"x\"\n</code></pre>";
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert_eq!(result, html);
        assert!(!result.contains("broken-link"));
    }

    #[test]
    fn wikilink_syntax_in_inline_code_untouched() {
        let (_dir, inv) = test_inventory();
        let html = "<p>Inline <code>[[nav]]</code> too.</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert_eq!(result, html);
        assert!(!result.contains("broken-link"));
    }

    #[test]
    fn wikilink_outside_code_still_resolves_next_to_code() {
        let (_dir, inv) = test_inventory();
        let html = "<p>See [[setup]] and <code>[[nav]]</code> too.</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert!(result.contains("<a href=\"/setup.html\">setup</a>"));
        assert!(result.contains("<code>[[nav]]</code>"));
        assert!(!result.contains("broken-link"));
    }

    #[test]
    fn wikilink_resolves_outside_pre_but_not_inside_same_document() {
        let (_dir, inv) = test_inventory();
        let html =
            "<p>Before [[setup]].</p>\n<pre><code>[[nav]]</code></pre>\n<p>After [[setup]].</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert_eq!(
            result.matches("<a href=\"/setup.html\">setup</a>").count(),
            2
        );
        assert!(result.contains("<pre><code>[[nav]]</code></pre>"));
        assert!(!result.contains("broken-link"));
    }
}
