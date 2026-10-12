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

                let opening_tag = merge_attributes(&result[open_pos..open_tag_end], &attrs);

                // Update the opening tag, preserving the element's content.
                let new_result = format!(
                    "{}{}>{}\n",
                    &result[..open_pos],
                    opening_tag,
                    result[open_tag_end + 1..start].trim_end(),
                );
                result = format!("{}{}", new_result, &result[end..]);
            }
        }
    }

    result
}

/// Merge class tokens and replace IDs without mistaking attribute-like text
/// inside quoted values (or `data-class`) for actual class/id attributes.
fn merge_attributes(opening_tag: &str, added: &str) -> String {
    static ATTRIBUTE_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r#"(\s+)([a-zA-Z_:][a-zA-Z0-9_:.-]*)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+)))?"#,
        )
        .unwrap()
    });
    let value = |caps: &regex::Captures<'_>| {
        (3..=5)
            .find_map(|i| caps.get(i))
            .map_or("", |m| m.as_str())
            .to_string()
    };
    let mut classes = Vec::new();
    let mut id = None;
    let mut other = String::new();
    let mut has_new_class = false;
    for caps in ATTRIBUTE_RE.captures_iter(added) {
        if caps[2].eq_ignore_ascii_case("class") {
            has_new_class = true;
        } else if caps[2].eq_ignore_ascii_case("id") {
            id = Some(value(&caps));
        } else {
            other.push_str(&caps[0]);
        }
    }
    if has_new_class {
        for attrs in [opening_tag, added] {
            for caps in ATTRIBUTE_RE.captures_iter(attrs) {
                if caps[2].eq_ignore_ascii_case("class") {
                    for class in value(&caps).split_ascii_whitespace() {
                        if !classes.iter().any(|existing| existing == class) {
                            classes.push(class.to_string());
                        }
                    }
                }
            }
        }
    }
    let mut class_written = false;
    let mut id_written = false;
    // Keep existing entities intact while safely converting single-quoted values.
    let merged_classes = classes.join(" ").replace('"', "&quot;");
    let mut result = ATTRIBUTE_RE
        .replace_all(opening_tag, |caps: &regex::Captures| {
            if caps[2].eq_ignore_ascii_case("class") && has_new_class {
                if class_written {
                    return String::new();
                }
                class_written = true;
                return format!("{}class=\"{merged_classes}\"", &caps[1]);
            }
            if caps[2].eq_ignore_ascii_case("id")
                && let Some(id) = &id
            {
                if id_written {
                    return String::new();
                }
                id_written = true;
                return format!("{}id=\"{id}\"", &caps[1]);
            }
            caps[0].to_string()
        })
        .into_owned();
    if let Some(id) = id
        && !id_written
    {
        result.push_str(&format!(" id=\"{id}\""));
    }
    if has_new_class && !class_written {
        result.push_str(&format!(" class=\"{merged_classes}\""));
    }
    result.push_str(&other);
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

    #[test]
    fn merges_classes_and_replaces_id_on_component_outer_tag() {
        let html = "<div class=\"card rounded\" id=\"old\" data-kind=\"card\"><p>Body.</p></div>\n<p>{.extra .card #new}</p>\n";
        let result = inject_attributes(html);
        assert!(
            result.contains("<div class=\"card rounded extra\" id=\"new\" data-kind=\"card\">"),
            "{result}"
        );
        assert_eq!(result.matches(" class=").count(), 1);
        assert_eq!(result.matches(" id=").count(), 1);
    }

    #[test]
    fn recognizes_single_quoted_unquoted_and_case_insensitive_attributes() {
        for tag in ["<div CLASS='card' ID='old'>", "<div class=card id=old>"] {
            let result = inject_attributes(&format!("{tag}Body.</div>\n<p>{{.extra #new}}</p>\n"));
            assert!(result.contains("class=\"card extra\""), "{result}");
            assert!(result.contains("id=\"new\""), "{result}");
            assert!(!result.contains("old"));
        }
    }

    #[test]
    fn preserves_class_entities_and_quotes_when_merging() {
        let result =
            inject_attributes("<div class='say\"hi a&amp;b'>Body.</div>\n<p>{.extra}</p>\n");
        assert!(
            result.contains("class=\"say&quot;hi a&amp;b extra\""),
            "{result}"
        );
    }

    #[test]
    fn preserves_other_attributes_and_untouched_class_or_id() {
        let tag = "<div data-class=\"unrelated\" title=\"class='fake' id='fake'\" class=\"card\" id=\"old\">";
        let result = inject_attributes(&format!("{tag}Body.</div>\n<p>{{.extra}}</p>\n"));
        assert!(result.contains("data-class=\"unrelated\" title=\"class='fake' id='fake'\" class=\"card extra\" id=\"old\""), "{result}");
        let result = inject_attributes(&format!("{tag}Body.</div>\n<p>{{#new}}</p>\n"));
        assert!(result.contains("class=\"card\" id=\"new\""), "{result}");
    }
}
