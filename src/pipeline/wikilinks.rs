use std::path::Path;

use crate::config::DraftLinks;
use crate::diagnostics;
use crate::project::PageInventory;

/// Process wiki-links in rendered HTML.
/// Replaces `[[target]]` and `[[target|display text]]` with proper HTML links.
/// When `locale` is provided, links resolve within that locale only.
///
/// Code is left untouched: `<pre>…</pre>` blocks (highlighted blocks have no `<code>`
/// wrapper; unhighlighted ones are `<pre><code>…</code></pre>`) are copied through
/// verbatim, and a link can't start or end inside inline `<code>…</code>`, so `[[…]]`
/// inside code (e.g. TOML's `[[nav]]`) is never rewritten. Inline code can still sit
/// in a link's display text (``[[page|the `x` page]]``).
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
        match next_pre_block(remaining) {
            Some((before, pre, after)) => {
                result.push_str(&resolve_segment(
                    before,
                    inventory,
                    source_file,
                    base_url,
                    locale,
                ));
                result.push_str(pre);
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

/// Find the next `<pre>…</pre>` block in `html`.
/// Returns `(before, block, after)` where `block` includes the tags themselves.
/// A `<pre>` without a matching `</pre>` is treated as plain text from that point
/// on, since there's nothing safe to skip past.
fn next_pre_block(html: &str) -> Option<(&str, &str, &str)> {
    let start = html.find("<pre")?;
    let end = start + html[start..].find("</pre>")? + "</pre>".len();
    Some((&html[..start], &html[start..end], &html[end..]))
}

/// Byte ranges of the inline `<code>…</code>` spans in `html` (an unclosed
/// `<code>` runs to the end).
fn inline_code_ranges(html: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut from = 0;
    while let Some(rel) = html[from..].find("<code") {
        let start = from + rel;
        let end = html[start..]
            .find("</code>")
            .map_or(html.len(), |close| start + close + "</code>".len());
        ranges.push((start, end));
        from = end;
    }
    ranges
}

/// Byte ranges of the `[[…]]` wiki-links in `text`, brackets included. `code`
/// holds the byte ranges of inline code spans: a link can't start or end inside
/// one (so `` `[[nav]]` `` stays code), but one can sit in a link's display text.
pub(crate) fn find_links(text: &str, code: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut masked = text.as_bytes().to_vec();
    for &(start, end) in code {
        masked[start..end].fill(b' ');
    }
    let find = |from: usize, pattern: &[u8]| {
        masked[from..]
            .windows(pattern.len())
            .position(|w| w == pattern)
            .map(|pos| from + pos)
    };

    let mut links = Vec::new();
    let mut pos = 0;
    while let Some(open) = find(pos, b"[[") {
        // No closing ]]: the rest stays as written.
        let Some(close) = find(open + 2, b"]]") else {
            break;
        };
        links.push((open, close + 2));
        pos = close + 2;
    }
    links
}

/// Resolve `[[…]]` wiki-links within a segment known to contain no `<pre>` blocks.
fn resolve_segment(
    html: &str,
    inventory: &PageInventory,
    source_file: &Path,
    base_url: &str,
    locale: Option<&str>,
) -> String {
    let mut result = String::with_capacity(html.len());
    let mut last = 0;

    for (start, end) in find_links(html, &inline_code_ranges(html)) {
        result.push_str(&html[last..start]);
        let inner = &html[start + 2..end - 2];
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
        last = end;
    }

    result.push_str(&html[last..]);
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
    fn display_text_can_contain_inline_code() {
        let (_dir, inv) = test_inventory();
        let html = "<p>See [[setup|the <code>setup</code> page]] and [[setup|plain]].</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert_eq!(
            result,
            "<p>See <a href=\"/setup.html\">the <code>setup</code> page</a> and <a href=\"/setup.html\">plain</a>.</p>"
        );
    }

    #[test]
    fn closing_brackets_inside_code_in_display_text_dont_end_the_link() {
        let (_dir, inv) = test_inventory();
        let html = "<p>[[setup|<code>a]]b</code> c]]</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert_eq!(
            result,
            "<p><a href=\"/setup.html\"><code>a]]b</code> c</a></p>"
        );
    }

    #[test]
    fn broken_link_with_code_in_display_text_warns() {
        let (_dir, inv) = test_inventory();
        diagnostics::reset_warnings();
        let html = "<p>[[missing|the <code>missing</code> page]]</p>";
        let result = resolve(html, &inv, Path::new("test.md"), "/", None);
        assert!(
            result.contains("class=\"broken-link popover-trigger\""),
            "{result}"
        );
        assert!(result.contains("the <code>missing</code> page"), "{result}");
        assert_eq!(diagnostics::warning_count(), 1);
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
