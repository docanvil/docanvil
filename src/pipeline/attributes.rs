use regex::Regex;
use std::sync::LazyLock;

/// Post-comrak pass: process inline `{.class #id key="val"}` attribute blocks
/// that appear after HTML elements and inject them into the preceding tag.
///
/// This handles patterns like:
/// `<p>text</p>\n{.highlight}` → `<p class="highlight">text</p>`
/// `<h2>Title</h2>\n{#section-id .special}` → `<h2 id="section-id" class="special">Title</h2>`
pub fn inject_attributes(html: &str) -> String {
    static ATTR_LINE_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?m)^<p>\{([^}]+)\}</p>$").unwrap());

    let mut result = html.to_string();

    // Find all attribute blocks that are standalone paragraphs
    let matches: Vec<_> = ATTR_LINE_RE
        .captures_iter(html)
        .map(|caps| {
            let full = caps.get(0).unwrap();
            let inner = caps[1].to_string();
            (full.start(), full.end(), inner)
        })
        .collect();

    // Process in reverse order to preserve positions
    for (start, end, inner) in matches.into_iter().rev() {
        // Find the preceding closing tag
        let before = &result[..start].trim_end();
        if let Some(tag_end) = before.rfind('>')
            && let Some(tag_start) = before[..tag_end].rfind("</")
        {
            // Found closing tag like </h2>, now find the opening tag
            let tag_name = &before[tag_start + 2..tag_end];
            let open_pattern = format!("<{}", tag_name);

            if let Some(open_pos) = before[..tag_start].rfind(&open_pattern) {
                let Some(offset) = before[open_pos..].find('>') else {
                    crate::diagnostics::warn_malformed_attribute_tag();
                    continue;
                };
                let open_tag_end = offset + open_pos;

                // Build attribute string
                let attrs = build_attr_string(&inner);

                // Insert attributes into opening tag
                let new_result = format!(
                    "{}{}>{}\n",
                    &result[..open_tag_end],
                    attrs,
                    &result[open_tag_end + 1..start].trim_end(),
                );
                result = format!("{}{}", new_result, &result[end..]);
            }
        }
    }

    result
}

/// Build an HTML attribute string from shorthand notation.
fn build_attr_string(inner: &str) -> String {
    let mut classes = Vec::new();
    let mut id = None;
    let mut other_attrs = Vec::new();

    for part in inner.split_whitespace() {
        if let Some(class) = part.strip_prefix('.') {
            classes.push(class.to_string());
        } else if let Some(id_val) = part.strip_prefix('#') {
            id = Some(id_val.to_string());
        } else if part.contains('=') {
            other_attrs.push(part.to_string());
        }
    }

    let mut result = String::new();
    if let Some(id) = id {
        result.push_str(&format!(" id=\"{id}\""));
    }
    if !classes.is_empty() {
        result.push_str(&format!(" class=\"{}\"", classes.join(" ")));
    }
    for attr in other_attrs {
        result.push_str(&format!(" {attr}"));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_class_on_heading() {
        let html = "<h2>Title</h2>\n<p>{.highlight}</p>\n";
        let result = inject_attributes(html);
        assert!(result.contains("class=\"highlight\""));
        assert!(!result.contains("{.highlight}"));
    }

    #[test]
    fn inject_id_and_class() {
        let html = "<h2>Title</h2>\n<p>{#my-id .special}</p>\n";
        let result = inject_attributes(html);
        assert!(result.contains("id=\"my-id\""));
        assert!(result.contains("class=\"special\""));
    }

    #[test]
    fn no_attribute_block_unchanged() {
        let html = "<p>Normal paragraph</p>\n";
        let result = inject_attributes(html);
        assert_eq!(result, html);
    }
}
