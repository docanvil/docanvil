use regex::Regex;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// Holds preloaded syntax definitions and themes for reuse across pages.
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: String,
}

impl SyntaxHighlighter {
    /// Create a new highlighter. Loads bundled syntaxes and themes once.
    pub fn new(theme_name: &str) -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
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

/// Find `<pre><code class="language-X">…</code></pre>` blocks and replace them
/// with syntect-highlighted output. Unknown languages or an invalid theme cause
/// the block to pass through unchanged.
pub fn highlight_code_blocks(html: &str, highlighter: &SyntaxHighlighter) -> String {
    if !highlighter.theme_valid() {
        return html.to_string();
    }

    let re = Regex::new(
        r#"<pre><code class="language-([\w+\-\.#]+)">([\s\S]*?)</code></pre>"#,
    )
    .expect("syntax regex is valid");

    let theme = &highlighter.theme_set.themes[&highlighter.theme_name];

    re.replace_all(html, |caps: &regex::Captures| {
        let lang = &caps[1];
        let code = html_unescape(&caps[2]);

        match highlighter.syntax_set.find_syntax_by_token(lang) {
            Some(syntax) => {
                match highlighted_html_for_string(&code, &highlighter.syntax_set, syntax, theme) {
                    Ok(highlighted) => highlighted,
                    Err(_) => caps[0].to_string(),
                }
            }
            None => caps[0].to_string(),
        }
    })
    .into_owned()
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
    fn leaves_unknown_language_unchanged() {
        let h = highlighter();
        let input =
            r#"<pre><code class="language-nosuchlang">hello world</code></pre>"#;
        let result = highlight_code_blocks(input, &h);
        assert_eq!(result, input);
    }

    #[test]
    fn leaves_no_language_blocks_unchanged() {
        let h = highlighter();
        let input = r#"<pre><code>plain code</code></pre>"#;
        let result = highlight_code_blocks(input, &h);
        assert_eq!(result, input);
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
        assert!(!result.contains("language-rust"));
        assert!(!result.contains("language-python"));
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
}
