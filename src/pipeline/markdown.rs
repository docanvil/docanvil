use comrak::{Options, markdown_to_html};

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

#[cfg(test)]
mod tests {
    use super::*;

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
