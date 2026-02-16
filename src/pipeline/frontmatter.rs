use serde::Deserialize;

/// Parsed front matter metadata from a Markdown file.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
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
        let source =
            "---\n{\"title\": \"My Page\", \"custom_field\": \"some value\", \"tags\": [\"a\", \"b\", \"c\"]}\n---\n\nContent";
        let fm = extract(source);
        assert_eq!(fm.title.as_deref(), Some("My Page"));
    }

    #[test]
    fn no_closing_delimiter() {
        let source = "---\n{\"title\": \"Broken\"}\n\nContent without closing delimiter";
        let fm = extract(source);
        // No closing `---`, so no valid front matter
        assert!(fm.title.is_none());
    }
}
