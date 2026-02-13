use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// A parsed directive block from the Markdown source.
#[derive(Debug, Clone)]
pub struct DirectiveBlock {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub body: String,
    /// Nesting depth (number of colons in the fence).
    pub depth: usize,
}

static OPEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(:{3,})\s*([\w][\w-]*)\s*(\{.*\})?\s*$").unwrap());

static ATTR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(\w[\w-]*)="([^"]*)""#).unwrap());

static INLINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r":::([\w][\w-]*)\{([^}]*)\}").unwrap());

/// Pre-comrak pass: parse `:::directive{attrs}` blocks and replace them with
/// HTML placeholder comments that will survive Markdown rendering.
/// The returned string has directives replaced with rendered component HTML.
pub fn process_directives(
    source: &str,
    renderer: &mut dyn FnMut(&DirectiveBlock) -> String,
) -> String {
    let mut output = String::with_capacity(source.len());
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if let Some(caps) = OPEN_RE.captures(lines[i]) {
            let colons = caps[1].len();
            let name = caps[2].to_string();
            let attrs = caps
                .get(3)
                .map(|m| parse_attributes(m.as_str()))
                .unwrap_or_default();

            // Find matching closing fence (same or more colons)
            let close_pattern = ":".repeat(colons);
            let mut body_lines = Vec::new();
            let mut j = i + 1;
            let mut found_close = false;

            while j < lines.len() {
                let trimmed = lines[j].trim();
                if trimmed.starts_with(&close_pattern)
                    && trimmed.len() == colons
                    && trimmed.chars().all(|c| c == ':')
                {
                    found_close = true;
                    break;
                }
                body_lines.push(lines[j]);
                j += 1;
            }

            if found_close {
                let block = DirectiveBlock {
                    name,
                    attributes: attrs,
                    body: body_lines.join("\n"),
                    depth: colons,
                };
                let rendered = renderer(&block);
                output.push_str(&rendered);
                output.push('\n');
                i = j + 1;
            } else {
                // Unclosed directive — pass through as-is
                output.push_str(lines[i]);
                output.push('\n');
                i += 1;
            }
        } else {
            output.push_str(lines[i]);
            output.push('\n');
            i += 1;
        }
    }

    output
}

/// Process inline (self-closing) directives: `:::name{attrs}` patterns that appear
/// within text, have no body, and need no closing fence. Skips matches inside fenced
/// code blocks and inline code spans.
pub fn process_inline_directives(
    source: &str,
    renderer: &mut dyn FnMut(&DirectiveBlock) -> String,
) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut result = String::with_capacity(source.len());
    let mut in_fence = false;
    let mut fence_char = '`';
    let mut fence_len = 0;

    for (idx, line) in lines.iter().enumerate() {
        if idx > 0 {
            result.push('\n');
        }

        let trimmed = line.trim();

        if !in_fence {
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                fence_char = trimmed.chars().next().unwrap();
                fence_len = trimmed.chars().take_while(|&c| c == fence_char).count();
                in_fence = true;
                result.push_str(line);
                continue;
            }
            result.push_str(&replace_inline_in_line(line, renderer));
        } else {
            let close_count = trimmed.chars().take_while(|&c| c == fence_char).count();
            if close_count >= fence_len
                && trimmed.chars().skip(close_count).all(|c| c.is_whitespace())
            {
                in_fence = false;
            }
            result.push_str(line);
        }
    }

    if source.ends_with('\n') {
        result.push('\n');
    }

    result
}

/// Replace inline directives in a single line, skipping matches inside backtick code spans.
fn replace_inline_in_line(
    line: &str,
    renderer: &mut dyn FnMut(&DirectiveBlock) -> String,
) -> String {
    let protected = inline_code_ranges(line);
    let mut out = String::with_capacity(line.len());
    let mut last = 0;

    for caps in INLINE_RE.captures_iter(line) {
        let m = caps.get(0).unwrap();

        if protected
            .iter()
            .any(|&(s, e)| m.start() >= s && m.end() <= e)
        {
            continue;
        }

        out.push_str(&line[last..m.start()]);

        let name = caps[1].to_string();
        let attr_str = format!("{{{}}}", &caps[2]);
        let attrs = parse_attributes(&attr_str);

        let block = DirectiveBlock {
            name,
            attributes: attrs,
            body: String::new(),
            depth: 3,
        };
        out.push_str(&renderer(&block));
        last = m.end();
    }

    out.push_str(&line[last..]);
    out
}

/// Find byte ranges of inline code spans (backtick-delimited) in a line.
fn inline_code_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'`' {
            let start = i;
            let mut ticks = 0;
            while i < len && bytes[i] == b'`' {
                ticks += 1;
                i += 1;
            }
            // Search for matching closing run of exactly `ticks` backticks
            loop {
                if i >= len {
                    break;
                }
                if bytes[i] == b'`' {
                    let mut cticks = 0;
                    while i < len && bytes[i] == b'`' {
                        cticks += 1;
                        i += 1;
                    }
                    if cticks == ticks {
                        ranges.push((start, i));
                        break;
                    }
                } else {
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }

    ranges
}

/// Parse `{key="val" key2="val2"}` attribute strings.
pub fn parse_attributes(attr_str: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    // Strip braces
    let inner = attr_str
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}');

    // Parse class shorthand .class and id shorthand #id
    for part in inner.split_whitespace() {
        if let Some(class) = part.strip_prefix('.') {
            map.entry("class".to_string())
                .and_modify(|v: &mut String| {
                    v.push(' ');
                    v.push_str(class);
                })
                .or_insert_with(|| class.to_string());
        } else if let Some(id) = part.strip_prefix('#') {
            map.insert("id".to_string(), id.to_string());
        }
    }

    // Parse key="value" pairs
    for caps in ATTR_RE.captures_iter(inner) {
        map.insert(caps[1].to_string(), caps[2].to_string());
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_directive() {
        let input = "Before\n\n:::note\nThis is a note.\n:::\n\nAfter\n";
        let output = process_directives(input, &mut |block| {
            format!("<div class=\"{}\"><p>{}</p></div>", block.name, block.body)
        });
        assert!(output.contains("<div class=\"note\"><p>This is a note.</p></div>"));
        assert!(output.contains("Before"));
        assert!(output.contains("After"));
    }

    #[test]
    fn parse_directive_with_attributes() {
        let input = ":::note{title=\"Important\"}\nContent\n:::\n";
        let output = process_directives(input, &mut |block| {
            format!(
                "name={} title={}",
                block.name,
                block.attributes.get("title").unwrap()
            )
        });
        assert!(output.contains("name=note title=Important"));
    }

    #[test]
    fn nested_directives() {
        let input = "::::tabs\n:::tab{title=\"Rust\"}\nRust code\n:::\n:::tab{title=\"Python\"}\nPython code\n:::\n::::\n";
        let mut blocks = Vec::new();
        let _ = process_directives(input, &mut |block| {
            blocks.push(block.clone());
            format!("<{}>", block.name)
        });
        // The outer block (tabs) should capture everything
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "tabs");
        assert!(blocks[0].body.contains(":::tab"));
    }

    #[test]
    fn unclosed_directive_preserved() {
        let input = ":::note\nNo closing fence\n";
        let output = process_directives(input, &mut |_| "RENDERED".to_string());
        assert!(!output.contains("RENDERED"));
        assert!(output.contains(":::note"));
    }

    #[test]
    fn inline_directive() {
        let input = "Status: :::lozenge{type=\"success\",text=\"Done\"} end";
        let output = process_inline_directives(input, &mut |block| {
            format!(
                "<span class=\"lozenge {}\">{}</span>",
                block.attributes.get("type").unwrap_or(&String::new()),
                block.attributes.get("text").unwrap_or(&String::new()),
            )
        });
        assert!(output.contains("<span class=\"lozenge success\">Done</span>"));
        assert!(output.starts_with("Status: "));
        assert!(output.ends_with(" end"));
    }

    #[test]
    fn inline_directive_skipped_in_fenced_code_block() {
        let input = "Before\n\n```\n:::lozenge{type=\"success\",text=\"Done\"}\n```\n\nAfter\n";
        let output = process_inline_directives(input, &mut |_| "RENDERED".to_string());
        assert!(!output.contains("RENDERED"));
        assert!(output.contains(":::lozenge"));
    }

    #[test]
    fn inline_directive_skipped_in_inline_code() {
        let input = "Use `:::lozenge{type=\"success\",text=\"Done\"}` for badges";
        let output = process_inline_directives(input, &mut |_| "RENDERED".to_string());
        assert!(!output.contains("RENDERED"));
        assert!(output.contains(":::lozenge"));
    }

    #[test]
    fn inline_directive_not_triggered_by_block() {
        // A block directive on its own line should NOT be matched by inline processing
        // (it will already have been handled by process_directives)
        let input = ":::note{title=\"Important\"}\nContent\n:::\n";
        let block_output = process_directives(input, &mut |block| {
            format!("<div class=\"{}\">RENDERED</div>", block.name)
        });
        // After block processing, the :::note line is gone
        assert!(!block_output.contains(":::note"));
    }

    #[test]
    fn parse_attributes_shorthand() {
        let attrs = parse_attributes("{.info #my-id title=\"Hello\"}");
        assert_eq!(attrs.get("class").unwrap(), "info");
        assert_eq!(attrs.get("id").unwrap(), "my-id");
        assert_eq!(attrs.get("title").unwrap(), "Hello");
    }
}
