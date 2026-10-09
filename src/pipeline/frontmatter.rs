use serde::Deserialize;

/// Parsed front matter metadata from a Markdown file.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub slug: Option<String>,
    /// Set to `false` to hide the "Edit this page" link on this page.
    pub edit_link: Option<bool>,
    /// `"YYYY-MM-DD"` to override the page's "last updated" date, or `false` to hide it.
    /// Kept as raw JSON so a bad value can't discard the rest of the front matter;
    /// `last_updated::parse_override` interprets it.
    pub last_updated: Option<serde_json::Value>,
    /// `true` keeps the page out of `docanvil build` (it still shows in `docanvil serve`).
    pub draft: bool,
    /// Old paths that should redirect to this page. Kept as raw JSON so a bad value
    /// can't discard the rest of the front matter; `redirects` interprets it.
    pub redirect_from: Option<serde_json::Value>,
    /// Set to `false` to leave this page out of `llms.txt` and `llms-full.txt`.
    pub llms: Option<bool>,
}

/// Extract JSON front matter from a Markdown source string.
///
/// Expects the standard `---` delimiters at the start of the file.
/// Returns `FrontMatter::default()` if no front matter is found or parsing fails.
pub fn extract(source: &str) -> FrontMatter {
    let trimmed = source.trim_start();
    if !trimmed.starts_with("---") {
        return FrontMatter::default();
    }

    // Find the closing delimiter after the opening `---`
    let after_open = &trimmed[3..];
    let rest = after_open
        .strip_prefix('\n')
        .or_else(|| after_open.strip_prefix("\r\n"));
    let Some(rest) = rest else {
        return FrontMatter::default();
    };

    let Some(end) = rest.find("\n---") else {
        return FrontMatter::default();
    };

    let content = &rest[..end];
    serde_json::from_str(content).unwrap_or_default()
}

/// `source` without its front matter block, whatever the JSON inside it holds.
/// Sources without a complete `---` … `---` block come back unchanged.
pub fn strip(source: &str) -> &str {
    let Some(after_open) = source.trim_start().strip_prefix("---") else {
        return source;
    };
    let Some(rest) = after_open
        .strip_prefix('\n')
        .or_else(|| after_open.strip_prefix("\r\n"))
    else {
        return source;
    };
    let Some(end) = rest.find("\n---") else {
        return source;
    };
    // Skip the rest of the closing `---` line.
    let after_close = &rest[end + 4..];
    match after_close.find('\n') {
        Some(newline) => &after_close[newline + 1..],
        None => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_front_matter() {
        let source = "---\n{\"title\": \"Getting Started\", \"description\": \"Learn how to set up DocAnvil\", \"author\": \"Jane Doe\", \"date\": \"2024-01-15\"}\n---\n\n# Hello";
        let fm = extract(source);
        assert_eq!(fm.title.as_deref(), Some("Getting Started"));
        assert_eq!(
            fm.description.as_deref(),
            Some("Learn how to set up DocAnvil")
        );
        assert_eq!(fm.author.as_deref(), Some("Jane Doe"));
        assert_eq!(fm.date.as_deref(), Some("2024-01-15"));
    }

    #[test]
    fn partial_front_matter() {
        let source = "---\n{\"title\": \"My Page\"}\n---\n\nContent here";
        let fm = extract(source);
        assert_eq!(fm.title.as_deref(), Some("My Page"));
        assert!(fm.description.is_none());
        assert!(fm.author.is_none());
        assert!(fm.date.is_none());
    }

    #[test]
    fn no_front_matter() {
        let source = "# Just a heading\n\nSome content.";
        let fm = extract(source);
        assert!(fm.title.is_none());
        assert!(fm.description.is_none());
    }

    #[test]
    fn empty_front_matter() {
        let source = "---\n{}\n---\n\nContent";
        let fm = extract(source);
        assert!(fm.title.is_none());
        assert!(fm.description.is_none());
    }

    #[test]
    fn invalid_json() {
        let source = "---\n{not valid json\n---\n\nContent";
        let fm = extract(source);
        assert!(fm.title.is_none());
    }

    #[test]
    fn unknown_fields_ignored() {
        let source = "---\n{\"title\": \"My Page\", \"custom_field\": \"some value\", \"tags\": [\"a\", \"b\", \"c\"]}\n---\n\nContent";
        let fm = extract(source);
        assert_eq!(fm.title.as_deref(), Some("My Page"));
    }

    #[test]
    fn explicit_slug_field() {
        let source =
            "---\n{\"title\": \"My Page\", \"slug\": \"custom-slug\"}\n---\n\nContent here";
        let fm = extract(source);
        assert_eq!(fm.title.as_deref(), Some("My Page"));
        assert_eq!(fm.slug.as_deref(), Some("custom-slug"));
    }

    #[test]
    fn slug_without_title() {
        let source = "---\n{\"slug\": \"override-slug\"}\n---\n\nContent";
        let fm = extract(source);
        assert!(fm.title.is_none());
        assert_eq!(fm.slug.as_deref(), Some("override-slug"));
    }

    #[test]
    fn no_closing_delimiter() {
        let source = "---\n{\"title\": \"Broken\"}\n\nContent without closing delimiter";
        let fm = extract(source);
        // No closing `---`, so no valid front matter
        assert!(fm.title.is_none());
    }

    #[test]
    fn last_updated_date_and_false() {
        let fm = extract("---\n{\"last_updated\": \"2026-09-30\"}\n---\n# Hi");
        assert_eq!(fm.last_updated, Some(serde_json::json!("2026-09-30")));
        let fm = extract("---\n{\"last_updated\": false}\n---\n# Hi");
        assert_eq!(fm.last_updated, Some(serde_json::json!(false)));
    }

    #[test]
    fn invalid_last_updated_keeps_other_fields() {
        for value in ["\"2026-9-1\"", "true", "42", "{\"a\": 1}"] {
            let source = format!(
                "---\n{{\"title\": \"Kept\", \"slug\": \"kept\", \"last_updated\": {value}}}\n---\n# Hi"
            );
            let fm = extract(&source);
            assert_eq!(fm.title.as_deref(), Some("Kept"), "value {value}");
            assert_eq!(fm.slug.as_deref(), Some("kept"), "value {value}");
        }
    }

    #[test]
    fn draft_flag() {
        assert!(extract("---\n{\"draft\": true}\n---\n# Hi").draft);
        assert!(!extract("---\n{\"title\": \"Hi\"}\n---\n# Hi").draft);
        assert!(!extract("# Hi").draft);
    }

    #[test]
    fn redirect_from_kept_as_raw_json() {
        let fm = extract("---\n{\"redirect_from\": [\"setup\", \"old/install\"]}\n---\n# Hi");
        assert_eq!(
            fm.redirect_from,
            Some(serde_json::json!(["setup", "old/install"]))
        );
        // A bad value doesn't discard the rest of the front matter.
        let fm = extract("---\n{\"title\": \"Kept\", \"redirect_from\": 5}\n---\n# Hi");
        assert_eq!(fm.title.as_deref(), Some("Kept"));
        assert_eq!(fm.redirect_from, Some(serde_json::json!(5)));
    }

    #[test]
    fn llms_opt_out() {
        assert_eq!(extract("---\n{\"llms\": false}\n---\n# A").llms, Some(false));
        assert_eq!(extract("# A").llms, None);
    }

    #[test]
    fn strip_removes_front_matter() {
        assert_eq!(strip("---\n{\"title\": \"X\"}\n---\n# X\n"), "# X\n");
        assert_eq!(strip("---\r\n{}\r\n---\r\nBody\r\n"), "Body\r\n");
        assert_eq!(strip("---\n{not json}\n---\nBody"), "Body");
        assert_eq!(strip("---\n{}\n---"), "");
    }

    #[test]
    fn strip_leaves_other_sources_alone() {
        assert_eq!(strip("# No front matter\n"), "# No front matter\n");
        assert_eq!(strip("---\nnever closed\n"), "---\nnever closed\n");
        assert_eq!(strip("--- not a block\n"), "--- not a block\n");
    }
}
