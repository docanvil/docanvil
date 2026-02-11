use regex::Regex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;

static POPOVER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\^\[([^\]]+)\]").unwrap());

static POPOVER_ID: AtomicUsize = AtomicUsize::new(0);

/// Escape HTML entities in popover content.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Pre-comrak pass: convert `^[content]` to inline popover HTML spans.
/// Skips fenced code blocks (``` lines) and inline code spans (backtick-wrapped).
pub fn process_popovers(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut in_code_block = false;

    for line in source.lines() {
        if !output.is_empty() {
            output.push('\n');
        }

        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            output.push_str(line);
            continue;
        }

        if in_code_block {
            output.push_str(line);
            continue;
        }

        output.push_str(&replace_popovers_in_line(line));
    }

    // Preserve trailing newline if the source had one
    if source.ends_with('\n') {
        output.push('\n');
    }

    output
}

/// Replace `^[content]` in a single line, skipping anything inside backtick spans.
fn replace_popovers_in_line(line: &str) -> String {
    // Split the line into segments: inside backticks (skip) and outside (process).
    let mut result = String::with_capacity(line.len());
    let mut remaining = line;

    while !remaining.is_empty() {
        // Find the next backtick
        if let Some(tick_pos) = remaining.find('`') {
            // Process the part before the backtick
            let before = &remaining[..tick_pos];
            result.push_str(&replace_popovers_in_text(before));

            // Find the closing backtick
            let after_tick = &remaining[tick_pos + 1..];
            if let Some(close_pos) = after_tick.find('`') {
                // Output the entire inline code span verbatim
                result.push_str(&remaining[tick_pos..tick_pos + 1 + close_pos + 1]);
                remaining = &after_tick[close_pos + 1..];
            } else {
                // No closing backtick — output the rest verbatim
                result.push_str(&remaining[tick_pos..]);
                return result;
            }
        } else {
            // No backticks left — process the remainder
            result.push_str(&replace_popovers_in_text(remaining));
            return result;
        }
    }

    result
}

/// Replace all `^[content]` matches in a text segment (known to be outside code spans).
fn replace_popovers_in_text(text: &str) -> String {
    POPOVER_RE
        .replace_all(text, |caps: &regex::Captures| {
            let content = escape_html(&caps[1]);
            let id = POPOVER_ID.fetch_add(1, Ordering::Relaxed);
            format!(
                "<span class=\"popover-trigger\" tabindex=\"0\" aria-describedby=\"popover-{id}\">\
                 <span class=\"popover-indicator\"></span>\
                 <span class=\"popover-content\" id=\"popover-{id}\" role=\"tooltip\">{content}</span>\
                 </span>"
            )
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_popover() {
        let input = "Hello^[world] there";
        let result = process_popovers(input);
        assert!(result.contains("class=\"popover-trigger\""));
        assert!(result.contains("role=\"tooltip\""));
        assert!(result.contains("world"));
        assert!(result.contains("Hello"));
        assert!(result.contains("there"));
    }

    #[test]
    fn multiple_popovers_on_one_line() {
        let input = "A^[first] and B^[second]";
        let result = process_popovers(input);
        assert!(result.contains("first"));
        assert!(result.contains("second"));
        // Should have two popover-trigger spans
        assert_eq!(result.matches("popover-trigger").count(), 2);
    }

    #[test]
    fn skips_fenced_code_blocks() {
        let input = "before\n```\ncode^[nope]\n```\nafter^[yes]";
        let result = process_popovers(input);
        assert!(result.contains("code^[nope]")); // unchanged
        assert!(result.contains("popover-trigger")); // the after one is converted
        assert_eq!(result.matches("popover-trigger").count(), 1);
    }

    #[test]
    fn skips_inline_code() {
        let input = "text `code^[nope]` and^[yes]";
        let result = process_popovers(input);
        assert!(result.contains("`code^[nope]`")); // unchanged inside backticks
        assert_eq!(result.matches("popover-trigger").count(), 1);
    }

    #[test]
    fn escapes_html_in_content() {
        let input = "test^[<script>alert(1)</script>]";
        let result = process_popovers(input);
        assert!(result.contains("&lt;script&gt;"));
        assert!(!result.contains("<script>alert"));
    }

    #[test]
    fn no_popover_syntax_unchanged() {
        let input = "Just normal text with [brackets] and stuff.";
        let result = process_popovers(input);
        assert_eq!(result, input);
    }

    #[test]
    fn unique_ids() {
        let input = "A^[first] B^[second]";
        let result = process_popovers(input);
        // Each popover should have a unique id
        let id_count = result.matches("popover-").count();
        // 2 triggers * (aria-describedby + id) = 4 occurrences of "popover-<N>"
        assert!(id_count >= 4);
    }

    #[test]
    fn preserves_trailing_newline() {
        let input = "hello^[world]\n";
        let result = process_popovers(input);
        assert!(result.ends_with('\n'));
    }
}
