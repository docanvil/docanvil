/// Escape HTML entities in user-supplied content.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_special_chars() {
        assert_eq!(html_escape("hello world"), "hello world");
    }

    #[test]
    fn escapes_ampersand() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn escapes_angle_brackets() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn escapes_double_quotes() {
        assert_eq!(html_escape(r#"say "hi""#), "say &quot;hi&quot;");
    }

    #[test]
    fn escapes_all_entities_together() {
        assert_eq!(
            html_escape(r#"<a href="x&y">"#),
            "&lt;a href=&quot;x&amp;y&quot;&gt;"
        );
    }

    #[test]
    fn empty_string() {
        assert_eq!(html_escape(""), "");
    }

    #[test]
    fn already_escaped_entities() {
        assert_eq!(html_escape("&amp;"), "&amp;amp;");
    }
}
