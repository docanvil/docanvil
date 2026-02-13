use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

#[derive(Debug, Serialize)]
pub struct SearchSection {
    pub id: String,
    pub title: String,
    pub heading: String,
    pub anchor: String,
    pub url: String,
    pub body: String,
}

/// Remove HTML tags, decode common entities, and collapse whitespace.
pub fn strip_html_tags(html: &str) -> String {
    static TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());
    static WS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

    let stripped = TAG_RE.replace_all(html, " ");
    let result = stripped
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    WS_RE.replace_all(&result, " ").trim().to_string()
}

/// Split HTML into sections based on headings with IDs.
/// Each heading starts a new section; content before the first heading is the intro.
pub fn extract_sections(html: &str, slug: &str, title: &str, base_url: &str) -> Vec<SearchSection> {
    // Split on heading tags that have an id attribute
    static HEADING_SPLIT_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"<h[1-6][^>]*\bid="([^"]*)"[^>]*>(.*?)</h[1-6]>"#).unwrap());

    let page_url = format!("{}{}.html", base_url, slug);
    let mut sections = Vec::new();

    // Collect all heading positions
    let matches: Vec<_> = HEADING_SPLIT_RE
        .captures_iter(html)
        .map(|caps| {
            let m = caps.get(0).unwrap();
            let anchor = caps[1].to_string();
            let heading_html = caps[2].to_string();
            let heading_text = strip_html_tags(&heading_html);
            (m.start(), m.end(), anchor, heading_text)
        })
        .collect();

    if matches.is_empty() {
        // No headings — single intro section for the whole page
        let body = strip_html_tags(html);
        if !body.is_empty() {
            sections.push(SearchSection {
                id: slug.to_string(),
                title: title.to_string(),
                heading: String::new(),
                anchor: String::new(),
                url: page_url,
                body,
            });
        }
        return sections;
    }

    // Intro section: content before the first heading
    let intro_html = &html[..matches[0].0];
    let intro_body = strip_html_tags(intro_html);
    if !intro_body.is_empty() {
        sections.push(SearchSection {
            id: slug.to_string(),
            title: title.to_string(),
            heading: String::new(),
            anchor: String::new(),
            url: page_url.clone(),
            body: intro_body,
        });
    }

    // Each heading starts a section that runs until the next heading
    for (i, (_, end, anchor, heading_text)) in matches.iter().enumerate() {
        let section_end = if i + 1 < matches.len() {
            matches[i + 1].0
        } else {
            html.len()
        };

        let section_html = &html[*end..section_end];
        let body = strip_html_tags(section_html);

        sections.push(SearchSection {
            id: format!("{}#{}", slug, anchor),
            title: title.to_string(),
            heading: heading_text.clone(),
            anchor: anchor.clone(),
            url: format!("{}#{}", page_url, anchor),
            body,
        });
    }

    sections
}

/// Serialize search sections to JSON.
pub fn build_index(entries: &[SearchSection]) -> String {
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
    fn extract_sections_no_headings() {
        let html = "<p>Just some text</p>";
        let sections = extract_sections(html, "intro", "Intro Page", "/");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].heading, "");
        assert_eq!(sections[0].anchor, "");
        assert_eq!(sections[0].url, "/intro.html");
        assert!(sections[0].body.contains("Just some text"));
    }

    #[test]
    fn extract_sections_with_headings() {
        let html = r#"<p>Intro text</p><h2 id="install">Installation</h2><p>Install steps</p><h2 id="usage">Usage</h2><p>Usage info</p>"#;
        let sections = extract_sections(html, "guide", "Guide", "/docs/");

        assert_eq!(sections.len(), 3);

        // Intro
        assert_eq!(sections[0].heading, "");
        assert_eq!(sections[0].url, "/docs/guide.html");
        assert!(sections[0].body.contains("Intro text"));

        // Installation section
        assert_eq!(sections[1].heading, "Installation");
        assert_eq!(sections[1].anchor, "install");
        assert_eq!(sections[1].url, "/docs/guide.html#install");
        assert!(sections[1].body.contains("Install steps"));

        // Usage section
        assert_eq!(sections[2].heading, "Usage");
        assert_eq!(sections[2].anchor, "usage");
        assert_eq!(sections[2].url, "/docs/guide.html#usage");
        assert!(sections[2].body.contains("Usage info"));
    }

    #[test]
    fn extract_sections_heading_with_inner_tags() {
        let html = r#"<h2 id="api">The <code>API</code> Reference</h2><p>Details here</p>"#;
        let sections = extract_sections(html, "ref", "Reference", "/");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].heading, "The API Reference");
        assert_eq!(sections[0].anchor, "api");
    }

    #[test]
    fn extract_sections_no_intro_when_heading_first() {
        let html = r#"<h2 id="first">First</h2><p>Content</p>"#;
        let sections = extract_sections(html, "page", "Page", "/");
        // No intro section since there's no content before the heading
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].heading, "First");
    }

    #[test]
    fn build_index_json() {
        let entries = vec![SearchSection {
            id: "getting-started".to_string(),
            title: "Getting Started".to_string(),
            heading: String::new(),
            anchor: String::new(),
            url: "/getting-started.html".to_string(),
            body: "Install and run".to_string(),
        }];
        let json = build_index(&entries);
        assert!(json.contains("\"id\":\"getting-started\""));
        assert!(json.contains("\"heading\":\"\""));
        assert!(json.contains("\"anchor\":\"\""));
    }
}
