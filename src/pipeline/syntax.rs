use std::sync::LazyLock;

use regex::Regex;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::{SyntaxDefinition, SyntaxSet};

/// Holds preloaded syntax definitions and themes for reuse across pages.
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: String,
}

impl SyntaxHighlighter {
    /// Create a new highlighter. Loads bundled syntaxes and themes once.
    pub fn new(theme_name: &str) -> Self {
        let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
        builder.add(
            SyntaxDefinition::load_from_str(
                include_str!("syntaxes/TOML.sublime-syntax"),
                true,
                None,
            )
            .expect("bundled TOML syntax is valid"),
        );
        Self {
            syntax_set: builder.build(),
            theme_set: ThemeSet::load_defaults(),
            theme_name: theme_name.to_string(),
        }
    }

    /// Check whether the configured theme exists.
    fn theme_valid(&self) -> bool {
        self.theme_set.themes.contains_key(&self.theme_name)
    }
}

/// Reverse comrak's HTML entity escaping so syntect receives raw source code.
fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

static BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"<pre><code(?: class="language-([\w+\-\.#]+)")?(?: data-meta="([^"]*)")?>([\s\S]*?)</code></pre>"#,
    )
    .expect("syntax regex is valid")
});

/// Render fenced code blocks using the configured syntax theme, keeping fence
/// metadata on the new `<pre>`. Unknown or absent languages use plain text syntax
/// so their background and foreground match highlighted blocks. An invalid theme
/// leaves blocks unchanged.
pub fn highlight_code_blocks(html: &str, highlighter: &SyntaxHighlighter) -> String {
    if !highlighter.theme_valid() {
        return html.to_string();
    }

    let theme = &highlighter.theme_set.themes[&highlighter.theme_name];

    BLOCK_RE
        .replace_all(html, |caps: &regex::Captures| {
            let lang = caps.get(1).map_or("", |m| m.as_str());
            let code = html_unescape(&caps[3]);
            let syntax = highlighter
                .syntax_set
                .find_syntax_by_token(lang)
                .unwrap_or_else(|| highlighter.syntax_set.find_syntax_plain_text());

            match highlighted_html_for_string(&code, &highlighter.syntax_set, syntax, theme) {
                Ok(highlighted) => with_meta(highlighted, lang, caps.get(2).map(|m| m.as_str())),
                Err(_) => caps[0].to_string(),
            }
        })
        .into_owned()
}

/// syntect's output starts `<pre style="…">`; put the block's meta on that tag.
fn with_meta(highlighted: String, lang: &str, meta: Option<&str>) -> String {
    let Some(end) = highlighted.find('>') else {
        return highlighted;
    };
    let language = if lang.is_empty() {
        String::new()
    } else {
        format!(" data-language=\"{lang}\"")
    };
    let meta = meta.map_or(String::new(), |meta| format!(" data-meta=\"{meta}\""));
    format!(
        "{}{language}{meta}{}",
        &highlighted[..end],
        &highlighted[end..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn highlighter() -> SyntaxHighlighter {
        SyntaxHighlighter::new("base16-ocean.dark")
    }

    #[test]
    fn highlights_known_language() {
        let h = highlighter();
        let input = r#"<pre><code class="language-rust">fn main() {}</code></pre>"#;
        let result = highlight_code_blocks(input, &h);
        // Should produce a <pre style="..."> with spans, not the original block
        assert!(result.contains("<pre style=\""));
        assert!(result.contains("<span"));
        assert!(!result.contains("<code class=\"language-rust\">"));
    }

    #[test]
    fn highlights_toml_tokens_and_multiline_strings() {
        use syntect::easy::HighlightLines;

        let h = highlighter();
        let syntax = h.syntax_set.find_syntax_by_token("toml").unwrap();
        assert_eq!(syntax.scope.to_string(), "source.toml");
        assert_eq!(
            h.syntax_set.find_syntax_by_extension("toml").unwrap().scope,
            syntax.scope
        );
        let theme = &h.theme_set.themes[&h.theme_name];
        let mut lines = HighlightLines::new(syntax, theme);
        for line in [
            "[project]\n",
            "[[nav]]\n",
            "name = \"DocAnvil\" # comment\n",
            "enabled = true\n",
            "count = 42\n",
            "ratio = 1.5\n",
            "hex = 0xFF\n",
            "date = 2026-10-11T12:30:00Z\n",
            "items = [1, 2, 3]\n",
            "options = { enabled = false }\n",
            "'quoted.key' = 'literal'\n",
            "message = \"\"\"first\n",
            "# still a string\n",
            "last\"\"\"\n",
            "count = 99\n",
            "literal = '''first\n",
            "# still literal\n",
            "last'''\n",
        ] {
            let spans = lines.highlight_line(line, &h.syntax_set).unwrap();
            assert_eq!(
                spans.iter().map(|(_, text)| *text).collect::<String>(),
                line
            );
            assert!(
                spans
                    .iter()
                    .any(|(style, _)| Some(style.foreground) != theme.settings.foreground),
                "{line}"
            );
        }
    }

    #[test]
    fn toml_values_receive_token_colours() {
        use syntect::easy::HighlightLines;

        let h = highlighter();
        let syntax = h.syntax_set.find_syntax_by_token("toml").unwrap();
        let theme = &h.theme_set.themes[&h.theme_name];
        let mut lines = HighlightLines::new(syntax, theme);
        for (line, token) in [
            ("enabled = true\n", "true"),
            ("count = 42\n", "42"),
            ("ratio = 1.5\n", "1"),
            ("name = \"DocAnvil\"\n", "DocAnvil"),
            ("# config comment\n", "config comment"),
        ] {
            let spans = lines.highlight_line(line, &h.syntax_set).unwrap();
            assert!(
                spans.iter().any(|(style, text)| text.contains(token)
                    && Some(style.foreground) != theme.settings.foreground),
                "{line}"
            );
        }
    }

    #[test]
    fn themes_unknown_language_as_plain_text() {
        let h = highlighter();
        let input = r#"<pre><code class="language-nosuchlang">hello world</code></pre>"#;
        let result = highlight_code_blocks(input, &h);
        assert!(
            result.starts_with("<pre style=\"background-color:#2b303b;\""),
            "{result}"
        );
        assert!(result.contains("data-language=\"nosuchlang\""));
        assert!(result.contains("color:#c0c5ce;"));
        assert!(result.contains("hello world"));
    }

    #[test]
    fn themes_no_language_blocks_as_plain_text() {
        let h = highlighter();
        let input = r#"<pre><code>plain code</code></pre>"#;
        let result = highlight_code_blocks(input, &h);
        assert!(
            result.starts_with("<pre style=\"background-color:#2b303b;\""),
            "{result}"
        );
        assert!(result.contains("color:#c0c5ce;"));
        assert!(!result.contains("data-language"));
    }

    #[test]
    fn fallback_uses_configured_theme_and_preserves_file_block_features() {
        for theme in ["base16-ocean.dark", "InspiredGitHub"] {
            let h = SyntaxHighlighter::new(theme);
            let input = "<pre><code class=\"language-nosuchlang\" data-meta=\"docanvil ranges=1-1,4-4 file=config.toml\">[project]\nname = &quot;Docs&quot;\n</code></pre>";
            let highlighted = highlight_code_blocks(input, &h);
            let reference = highlight_code_blocks(
                "<pre><code class=\"language-rust\">fn main() {}\n</code></pre>",
                &h,
            );
            let style_end = reference.find(" data-language").unwrap();
            assert!(highlighted.starts_with(&reference[..style_end]), "{theme}");
            let out = crate::pipeline::code_blocks::process(&highlighted, false);
            assert!(out.contains("class=\"line-numbers\""));
            assert!(out.contains("data-line=\"4\""));
            assert!(out.contains("data-hidden=\"2\""));
            assert!(out.contains("config.toml</span>"));
            assert!(out.contains("data-language=\"nosuchlang\""));
            assert!(!out.contains("data-meta"));
            assert!(out.contains("name = &quot;Docs&quot;"));
        }
    }

    #[test]
    fn handles_multiple_blocks() {
        let h = highlighter();
        let input = r#"<p>text</p>
<pre><code class="language-rust">let x = 1;</code></pre>
<p>middle</p>
<pre><code class="language-python">x = 1</code></pre>"#;
        let result = highlight_code_blocks(input, &h);
        assert!(result.contains("<p>text</p>"));
        assert!(result.contains("<p>middle</p>"));
        // Both blocks should be highlighted
        assert!(result.contains("data-language=\"rust\""));
        assert!(result.contains("data-language=\"python\""));
        // Count pre tags with style — should be 2
        assert_eq!(result.matches("<pre style=\"").count(), 2);
    }

    #[test]
    fn unknown_theme_leaves_all_unchanged() {
        let h = SyntaxHighlighter::new("no-such-theme");
        let input = r#"<pre><code class="language-rust">fn main() {}</code></pre>"#;
        let result = highlight_code_blocks(input, &h);
        assert_eq!(result, input);
    }

    #[test]
    fn unescapes_html_entities() {
        let h = highlighter();
        let input =
            r#"<pre><code class="language-rust">if x &lt; 5 &amp;&amp; y &gt; 3 {}</code></pre>"#;
        let result = highlight_code_blocks(input, &h);
        // Should be highlighted (syntect re-escapes for its own HTML output)
        assert!(result.contains("<pre style=\""));
        assert!(!result.contains("<code class=\"language-rust\">"));
    }

    #[test]
    fn keeps_meta_on_highlighted_blocks() {
        let h = highlighter();
        let input = r#"<pre><code class="language-rust" data-meta="docanvil numbers=on">fn main() {}
</code></pre>"#;
        let result = highlight_code_blocks(input, &h);
        assert!(result.starts_with("<pre style=\""), "{result}");
        // Both language and fence metadata survive highlighting.
        assert!(
            result.contains(r#"data-language="rust" data-meta="docanvil numbers=on">"#),
            "{result}"
        );
    }

    #[test]
    fn unknown_language_preserves_meta() {
        let h = highlighter();
        let input = r#"<pre><code class="language-nosuchlang" data-meta="docanvil numbers=on">x</code></pre>"#;
        let result = highlight_code_blocks(input, &h);
        assert!(result.contains("data-language=\"nosuchlang\" data-meta=\"docanvil numbers=on\""));
    }

    #[test]
    fn syntect_closes_every_span_at_line_end() {
        // code_blocks.rs wraps each line in its own span. That's only safe
        // because syntect closes its spans at the end of every line — the
        // newline sits just inside the line's last span (`…\n</span>`).
        let h = highlighter();
        let input = "<pre><code class=\"language-rust\">fn a() {\n    let s = &quot;x\ny&quot;;\n}\n</code></pre>";
        let result = highlight_code_blocks(input, &h);
        let body = &result[result.find(">\n").unwrap() + 2..result.rfind("</pre>").unwrap()];
        let normalised = body.replace("\n</span>", "</span>\n");
        for line in normalised.lines() {
            assert_eq!(
                line.matches("<span").count(),
                line.matches("</span>").count(),
                "{line}"
            );
        }
    }
}
