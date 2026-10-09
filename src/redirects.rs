//! Redirects: small HTML pages at old URLs that send readers on to where a page
//! lives now.
//!
//! Declared in front matter (`"redirect_from": ["old/path"]` on the page that
//! moved) and in `[redirects]` in `docanvil.toml`. [`plan`] turns both into one
//! flat list of stub pages, plus the problems the build warns about and
//! `docanvil doctor` reports.

use crate::util::html_escape;

/// One side of a redirect, as written.
#[allow(dead_code)] // used by plan() (Task 4)
#[derive(Debug, Clone, PartialEq, Eq)]
enum Spec {
    /// `help/faq`: a page slug, expanded for every version and language.
    Slug(String),
    /// `/old/page.html`: a path from the site root (stored without the leading
    /// `/`), never expanded.
    Literal(String),
    /// `https://…`: another site. Only valid as a target.
    External(String),
}

/// Read one side of a redirect. `target` allows external URLs.
#[allow(dead_code)] // used by plan() (Task 4)
fn parse_spec(raw: &str, target: bool) -> std::result::Result<Spec, String> {
    let s = raw.trim().replace('\\', "/");
    if s.is_empty() {
        return Err("a redirect path is empty".to_string());
    }
    if s.contains("://") {
        return if target {
            Ok(Spec::External(s))
        } else {
            Err(format!(
                "\"{raw}\" is a URL, but an old path must be a page on this site"
            ))
        };
    }
    let bytes = s.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return Err(format!(
            "\"{raw}\" is a filesystem path. Use a page slug like \"guides/setup\" or a site path like \"/old.html\""
        ));
    }
    if s.contains(['?', '#']) {
        return Err(format!("\"{raw}\" can't contain ? or #"));
    }
    if s.split('/').any(|segment| segment == "..") {
        return Err(format!("\"{raw}\" can't contain .."));
    }

    if let Some(rest) = s.strip_prefix('/') {
        let rest = rest.trim_start_matches('/');
        let path = if rest.is_empty() || rest.ends_with('/') {
            format!("{rest}index.html")
        } else if rest.ends_with(".html") {
            rest.to_string()
        } else if rest
            .rsplit('/')
            .next()
            .is_some_and(|last| last.contains('.'))
        {
            return Err(format!(
                "\"{raw}\" isn't a page. A site path must end in .html or /"
            ));
        } else {
            format!("{rest}.html")
        };
        return Ok(Spec::Literal(path));
    }

    let slug = s.trim_end_matches('/');
    let slug = slug.strip_suffix(".html").unwrap_or(slug);
    if slug.is_empty() {
        return Err(format!("\"{raw}\" isn't a page slug"));
    }
    Ok(Spec::Slug(slug.to_string()))
}

/// A page that sends the reader to `href`: meta refresh for no-JS, a script that
/// keeps the old URL's `?query` and `#anchor`, `noindex` + canonical so search
/// engines move their ranking to the target, and a plain link as a last resort.
pub fn stub_html(href: &str, canonical: &str, lang: Option<&str>) -> String {
    let href_attr = html_escape(href);
    let canonical_attr = html_escape(canonical);
    // A JSON string is a valid JS string literal; escaping `<` stops a target
    // from closing the script element.
    let href_js = serde_json::to_string(href)
        .unwrap_or_default()
        .replace('<', "\\u003c");
    let lang_attr = lang
        .map(|l| format!(" lang=\"{}\"", html_escape(l)))
        .unwrap_or_default();
    format!(
        "<!DOCTYPE html>\n\
         <html{lang_attr}>\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <title>Redirecting…</title>\n\
         <meta name=\"robots\" content=\"noindex\">\n\
         <link rel=\"canonical\" href=\"{canonical_attr}\">\n\
         <meta http-equiv=\"refresh\" content=\"0; url={href_attr}\">\n\
         <script>\n\
         var u = new URL({href_js}, location.href);\n\
         if (!u.search) u.search = location.search;\n\
         if (!u.hash) u.hash = location.hash;\n\
         location.replace(u.href);\n\
         </script>\n\
         </head>\n\
         <body>\n\
         <p>Redirecting to <a href=\"{href_attr}\">{href_attr}</a>…</p>\n\
         </body>\n\
         </html>\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_ignore_trailing_slash_and_html() {
        for raw in ["setup", "setup/", "setup.html", " setup "] {
            assert_eq!(
                parse_spec(raw, false),
                Ok(Spec::Slug("setup".into())),
                "{raw}"
            );
        }
        assert_eq!(
            parse_spec("guides\\setup", false),
            Ok(Spec::Slug("guides/setup".into()))
        );
    }

    #[test]
    fn leading_slash_is_a_literal_site_path() {
        assert_eq!(
            parse_spec("/old.html", false),
            Ok(Spec::Literal("old.html".into()))
        );
        assert_eq!(
            parse_spec("/old/page", false),
            Ok(Spec::Literal("old/page.html".into()))
        );
        assert_eq!(
            parse_spec("/old/", false),
            Ok(Spec::Literal("old/index.html".into()))
        );
        assert_eq!(
            parse_spec("/", false),
            Ok(Spec::Literal("index.html".into()))
        );
    }

    #[test]
    fn urls_are_only_allowed_as_targets() {
        assert_eq!(
            parse_spec("https://blog.example.com", true),
            Ok(Spec::External("https://blog.example.com".into()))
        );
        assert!(parse_spec("https://blog.example.com", false).is_err());
    }

    #[test]
    fn bad_paths_are_rejected() {
        for raw in [
            "",
            "  ",
            "../secret",
            "a/../b",
            "C:/docs/x.md",
            "old#part",
            "page?x=1",
            "/report.pdf",
            ".html",
        ] {
            assert!(parse_spec(raw, true).is_err(), "{raw:?} should be invalid");
        }
    }

    #[test]
    fn stub_redirects_three_ways() {
        let html = stub_html(
            "/docs/guides/install.html",
            "https://x.dev/docs/guides/install.html",
            Some("fr"),
        );
        assert!(html.contains("<html lang=\"fr\">"));
        assert!(html.contains(
            "<meta http-equiv=\"refresh\" content=\"0; url=/docs/guides/install.html\">"
        ));
        assert!(
            html.contains(
                "<link rel=\"canonical\" href=\"https://x.dev/docs/guides/install.html\">"
            )
        );
        assert!(html.contains("<meta name=\"robots\" content=\"noindex\">"));
        assert!(html.contains("location.replace"));
        assert!(html.contains("<a href=\"/docs/guides/install.html\">"));
    }

    #[test]
    fn stub_keeps_query_and_hash() {
        // The script copies the old URL's ?query and #anchor onto the target unless
        // the target sets its own.
        let html = stub_html("/new.html", "/new.html", None);
        assert!(html.contains("if (!u.search) u.search = location.search;"));
        assert!(html.contains("if (!u.hash) u.hash = location.hash;"));
        assert!(html.contains("<html>"));
    }

    #[test]
    fn stub_escapes_its_target() {
        let html = stub_html("/a\"b<c>.html", "/a\"b<c>.html", None);
        assert!(html.contains("url=/a&quot;b&lt;c&gt;.html"));
        // A target can't close the script element early.
        assert!(!html.contains("</script></script>"));
        assert!(html.contains("\\u003c"));
    }
}
