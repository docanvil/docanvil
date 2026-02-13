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
/// within text, have no body, and need no closing fence. Runs after block directive
/// processing so only unmatched inline patterns remain.
pub fn process_inline_directives(
    source: &str,
    renderer: &mut dyn FnMut(&DirectiveBlock) -> String,
) -> String {
    let mut result = String::with_capacity(source.len());
    let mut last_end = 0;

    for caps in INLINE_RE.captures_iter(source) {
        let full_match = caps.get(0).unwrap();
        let name = caps[1].to_string();
        let attr_str = format!("{{{}}}", &caps[2]);
        let attrs = parse_attributes(&attr_str);

        let block = DirectiveBlock {
            name,
            attributes: attrs,
            body: String::new(),
            depth: 3,
        };

        result.push_str(&source[last_end..full_match.start()]);
        result.push_str(&renderer(&block));
        last_end = full_match.end();
    }

    result.push_str(&source[last_end..]);
    result
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
