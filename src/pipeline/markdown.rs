use std::sync::LazyLock;

use comrak::nodes::NodeValue;
use comrak::{Arena, Options, markdown_to_html, parse_document};
use regex::Regex;

/// Build comrak options with GFM extensions enabled.
pub fn comrak_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.superscript = true;
    options.extension.subscript = true;
    options.extension.highlight = true;
    options.extension.shortcodes = true;
    options.extension.description_lists = true;
    options.extension.front_matter_delimiter = Some("---".to_string());
    options.render.r#unsafe = true;
    options
}

/// Render Markdown source to HTML using comrak with GFM extensions.
pub fn render(source: &str) -> String {
    let options = comrak_options();
    markdown_to_html(source, &options)
}

/// Plain text of the page's first top-level `# H1`, used as its title.
///
/// Inline formatting is dropped (`` # The `build` command `` → "The build command")
/// and a trailing custom `{#id}` is ignored. Headings inside fenced code are skipped.
pub fn first_h1_text(source: &str) -> Option<String> {
    static CUSTOM_ID_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\s*\{#[\w-]+\}\s*$").unwrap());

    let arena = Arena::new();
    let root = parse_document(&arena, source, &comrak_options());
    let heading = root
        .children()
        .find(|node| matches!(&node.data.borrow().value, NodeValue::Heading(h) if h.level == 1))?;

    let mut text = String::new();
    for node in heading.descendants() {
        match &node.data.borrow().value {
            NodeValue::Text(t) => text.push_str(t),
            NodeValue::Code(code) => text.push_str(&code.literal),
            NodeValue::SoftBreak | NodeValue::LineBreak => text.push(' '),
            _ => {}
        }
    }
    let text = CUSTOM_ID_RE.replace(text.trim(), "").trim().to_string();
    (!text.is_empty()).then_some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_h1_plain() {
        assert_eq!(
            first_h1_text("Intro\n\n# Getting Started\n\n# Second").as_deref(),
            Some("Getting Started")
        );
    }

    #[test]
    fn first_h1_strips_formatting_and_custom_id() {
        assert_eq!(
            first_h1_text("# The `build` **command** {#build}").as_deref(),
            Some("The build command")
        );
    }

    #[test]
    fn first_h1_setext_and_front_matter() {
        let src = "---\n{\"description\": \"x\"}\n---\n\nVersionnement\n=============\n";
        assert_eq!(first_h1_text(src).as_deref(), Some("Versionnement"));
    }

    #[test]
    fn first_h1_ignores_code_and_lower_levels() {
        assert_eq!(first_h1_text("## Sub\n\n```md\n# Not a title\n```\n"), None);
        assert_eq!(first_h1_text("#   "), None);
    }

    #[test]
    fn basic_paragraph() {
        let html = render("Hello, world!");
        assert_eq!(html.trim(), "<p>Hello, world!</p>");
    }

    #[test]
    fn headings() {
        let html = render("# Title\n\n## Subtitle");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<h2>Subtitle</h2>"));
    }

    #[test]
    fn gfm_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = render(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<td>1</td>"));
    }

    #[test]
    fn gfm_strikethrough() {
        let html = render("~~deleted~~");
        assert!(html.contains("<del>deleted</del>"));
    }

    #[test]
    fn gfm_tasklist() {
        let md = "- [x] done\n- [ ] todo";
        let html = render(md);
        assert!(html.contains("checked=\"\""));
        assert!(html.contains("type=\"checkbox\""));
    }

    #[test]
    fn footnotes() {
        let md = "Text[^1]\n\n[^1]: Footnote content";
        let html = render(md);
        assert!(html.contains("footnote"));
    }

    #[test]
    fn front_matter_stripped() {
        let md = "---\ntitle: Test\n---\n\nContent here";
        let html = render(md);
        assert!(!html.contains("title: Test"));
        assert!(html.contains("Content here"));
    }

    #[test]
    fn superscript() {
        let html = render("X^2^");
        assert!(html.contains("<sup>2</sup>"));
    }

    #[test]
    fn subscript() {
        let html = render("H~2~O");
        assert!(html.contains("<sub>2</sub>"));
    }

    #[test]
    fn highlight() {
        let html = render("==highlighted==");
        assert!(html.contains("<mark>highlighted</mark>"));
    }

    #[test]
    fn emoji_shortcodes() {
        let html = render("Hello :smile:");
        assert!(!html.contains(":smile:"));
        // Should be converted to an actual emoji character
        assert!(html.contains('\u{1F604}') || html.contains("😄"));
    }

    #[test]
    fn description_lists() {
        let md = "Term\n: Definition";
        let html = render(md);
        assert!(html.contains("<dl>"));
        assert!(html.contains("<dt>"));
        assert!(html.contains("<dd>"));
    }
}
