use regex::Regex;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SearchIndexEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub body: String,
    pub headings: Vec<String>,
}

/// Remove HTML tags, decode common entities, and collapse whitespace.
pub fn strip_html_tags(html: &str) -> String {
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let stripped = tag_re.replace_all(html, " ");
    let result = stripped
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    let ws_re = Regex::new(r"\s+").unwrap();
    ws_re.replace_all(&result, " ").trim().to_string()
}

/// Extract text content from h2 and h3 tags.
pub fn extract_headings(html: &str) -> Vec<String> {
    let re = Regex::new(r"<h[23][^>]*>(.*?)</h[23]>").unwrap();
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    re.captures_iter(html)
        .map(|cap| tag_re.replace_all(&cap[1], "").trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Serialize search entries to JSON.
pub fn build_index(entries: &[SearchIndexEntry]) -> String {
    serde_json::to_string(entries).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_basic() {
        let html = "<p>Hello <strong>world</strong></p>";
        assert_eq!(strip_html_tags(html), "Hello world");
    }

    #[test]
    fn strip_html_entities() {
        let html = "<p>A &amp; B &lt; C</p>";
        assert_eq!(strip_html_tags(html), "A & B < C");
    }

    #[test]
    fn strip_html_collapses_whitespace() {
        let html = "<p>Hello</p>\n\n<p>World</p>";
        assert_eq!(strip_html_tags(html), "Hello World");
    }

    #[test]
    fn extract_headings_h2_h3() {
        let html = "<h1>Title</h1><h2>Section One</h2><p>text</p><h3>Sub <em>section</em></h3>";
        let headings = extract_headings(html);
        assert_eq!(headings, vec!["Section One", "Sub section"]);
    }

    #[test]
    fn extract_headings_empty() {
        let html = "<p>No headings here</p>";
        let headings = extract_headings(html);
        assert!(headings.is_empty());
    }

    #[test]
    fn build_index_json() {
        let entries = vec![SearchIndexEntry {
            id: "getting-started".to_string(),
            title: "Getting Started".to_string(),
            url: "/getting-started.html".to_string(),
            body: "Install and run".to_string(),
            headings: vec!["Installation".to_string()],
        }];
        let json = build_index(&entries);
        assert!(json.contains("\"id\":\"getting-started\""));
        assert!(json.contains("\"headings\":[\"Installation\"]"));
    }
}
