use comrak::{markdown_to_html, Options};

/// Build comrak options with GFM extensions enabled.
pub fn comrak_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.front_matter_delimiter = Some("---".to_string());
    options.render.unsafe_ = true;
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
}
