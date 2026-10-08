//! Code block numbering, hidden-line gaps and captions.

use std::sync::LazyLock;

use regex::{Captures, Regex};

use crate::util::html_escape;

/// First token of a DocAnvil fence meta string.
pub const META_MARKER: &str = "docanvil";

/// What the include pass tells this stage about one code block. Carried from
/// the fence's info string through comrak (`data-meta`) and syntax highlighting.
///
/// Encoded as `docanvil key=value …` with keys in this order, each only when
/// set: `numbers=on|off`, `start=N`, `ranges=a-b,c-d`, `file=…`, `title=…`.
/// `file` and `title` are percent-encoded, so the string never holds spaces,
/// quotes or backticks.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BlockMeta {
    /// `Some(true)` for `numbers` / `numbers="N"`, `Some(false)` for `numbers="false"`.
    pub numbers: Option<bool>,
    /// First line number from `numbers="N"`.
    pub start: Option<usize>,
    /// Resolved `lines` ranges of a file block (empty = whole file or not a file block).
    pub ranges: Vec<(usize, usize)>,
    /// The `file` attribute as written.
    pub file: Option<String>,
    /// The `title` attribute; `Some("")` hides the caption.
    pub title: Option<String>,
}

impl BlockMeta {
    pub fn encode(&self) -> String {
        let mut out = String::from(META_MARKER);
        if let Some(numbers) = self.numbers {
            out.push_str(if numbers {
                " numbers=on"
            } else {
                " numbers=off"
            });
        }
        if let Some(start) = self.start {
            out.push_str(&format!(" start={start}"));
        }
        if !self.ranges.is_empty() {
            let ranges: Vec<String> = self
                .ranges
                .iter()
                .map(|(start, end)| format!("{start}-{end}"))
                .collect();
            out.push_str(&format!(" ranges={}", ranges.join(",")));
        }
        if let Some(file) = &self.file {
            out.push_str(&format!(" file={}", pct_encode(file)));
        }
        if let Some(title) = &self.title {
            out.push_str(&format!(" title={}", pct_encode(title)));
        }
        out
    }

    /// `None` unless `meta` starts with [`META_MARKER`] (other tools' info
    /// strings, like `{1,3}`, are ignored).
    pub fn parse(meta: &str) -> Option<Self> {
        let mut tokens = meta.split_whitespace();
        if tokens.next() != Some(META_MARKER) {
            return None;
        }
        let mut parsed = Self::default();
        for token in tokens {
            let Some((key, value)) = token.split_once('=') else {
                continue;
            };
            let value = pct_decode(value);
            match key {
                "numbers" => parsed.numbers = Some(value == "on"),
                "start" => parsed.start = value.parse().ok(),
                "ranges" => {
                    parsed.ranges = value
                        .split(',')
                        .filter_map(|r| {
                            let (a, b) = r.split_once('-')?;
                            Some((a.parse().ok()?, b.parse().ok()?))
                        })
                        .collect();
                }
                "file" => parsed.file = Some(value),
                "title" => parsed.title = Some(value),
                _ => {}
            }
        }
        Some(parsed)
    }
}

/// syntect output: `<pre style="…">` (optionally with `data-meta`), a newline,
/// then the lines. `highlighted_html_for_string` always ends with `</pre>\n`
/// (a real trailing newline it bakes into its output), so the match takes
/// that optional newline too; a rebuilt block (numbered/captioned) drops it,
/// an untouched one keeps it via `original` so blocks stay byte-identical.
static HIGHLIGHTED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<pre style="([^"]*)"(?: data-meta="([^"]*)")?>\n(.*?)</pre>\n?"#).unwrap()
});

/// Unhighlighted blocks: `<pre><code[ class="…"][ data-meta="…"]>…</code></pre>`,
/// with the same optional trailing newline as [`HIGHLIGHTED_RE`] (comrak also
/// emits one after block-level elements).
static PLAIN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)<pre><code((?: class="[^"]*")?)(?: data-meta="([^"]*)")?>(.*?)</code></pre>\n?"#,
    )
    .unwrap()
});

/// Line numbers, hidden-line gaps and captions. Runs after syntax highlighting
/// on every code block — highlighted, plain, unknown language or highlighting
/// off. `line_numbers` is the site default (`[syntax] line_numbers`).
/// Blocks with nothing to do are left as they were (minus any `data-meta`).
pub fn process(html: &str, line_numbers: bool) -> String {
    if !html.contains("<pre") {
        return html.to_string();
    }
    let html = HIGHLIGHTED_RE.replace_all(html, |caps: &Captures| {
        let style = &caps[1];
        render_block(
            &caps[0],
            caps.get(2).map(|m| m.as_str()),
            &caps[3],
            line_numbers,
            |attrs, inner| format!("<pre style=\"{style}\"{attrs}>\n{inner}</pre>"),
        )
    });
    PLAIN_RE
        .replace_all(&html, |caps: &Captures| {
            let code_attrs = &caps[1];
            render_block(
                &caps[0],
                caps.get(2).map(|m| m.as_str()),
                &caps[3],
                line_numbers,
                |attrs, inner| format!("<pre{attrs}><code{code_attrs}>{inner}</code></pre>"),
            )
        })
        .into_owned()
}

/// `wrap(pre_attrs, inner)` rebuilds the block's `<pre>` (without `data-meta`).
fn render_block(
    original: &str,
    meta: Option<&str>,
    body: &str,
    site_default: bool,
    wrap: impl Fn(&str, &str) -> String,
) -> String {
    let parsed = meta.and_then(BlockMeta::parse).unwrap_or_default();
    // Block setting → file block with `lines` (real numbers matter) → site default.
    let numbered = parsed
        .numbers
        .unwrap_or(!parsed.ranges.is_empty() || site_default);
    let caption = parsed.caption();

    if !numbered && parsed.ranges.is_empty() && caption.is_none() {
        return match meta {
            None => original.to_string(),
            Some(_) => wrap("", body),
        };
    }

    let lines = split_lines(body);
    let numbers = parsed.line_numbers(lines.len());
    let mut inner = String::new();
    let mut prev: Option<usize> = None;
    for (line, &n) in lines.iter().zip(&numbers) {
        if let Some(p) = prev
            && !parsed.ranges.is_empty()
            && n > p + 1
        {
            // No text inside, so copying the block copies only code.
            inner.push_str(&format!(
                "<span class=\"line-gap\" data-hidden=\"{}\"></span>",
                n - p - 1
            ));
        }
        inner.push_str(&format!(
            "<span class=\"line\" data-line=\"{n}\">{line}</span>\n"
        ));
        prev = Some(n);
    }

    let attrs = if numbered {
        let digits = numbers.last().map_or(1, |n| n.to_string().len());
        format!(" class=\"line-numbers\" data-line-digits=\"{digits}\"")
    } else {
        String::new()
    };
    let pre = wrap(&attrs, &inner);
    match caption {
        Some(caption) => format!("<figure class=\"code-block\">{caption}{pre}</figure>"),
        None => pre,
    }
}

/// Split a block body into lines. syntect leaves each line's newline inside the
/// line's last span (`…\n</span>`), so the `</span>`s straight after a newline
/// are moved back onto the line they close.
fn split_lines(body: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut rest = body;
    while let Some(pos) = rest.find('\n') {
        current.push_str(&rest[..pos]);
        rest = &rest[pos + 1..];
        while let Some(after) = rest.strip_prefix("</span>") {
            current.push_str("</span>");
            rest = after;
        }
        lines.push(std::mem::take(&mut current));
    }
    if !rest.is_empty() {
        current.push_str(rest);
        lines.push(current);
    }
    lines
}

impl BlockMeta {
    /// The number shown for each of `count` lines.
    fn line_numbers(&self, count: usize) -> Vec<usize> {
        if self.ranges.is_empty() {
            let start = self.start.unwrap_or(1);
            return (start..start + count).collect();
        }
        let mut numbers: Vec<usize> = self
            .ranges
            .iter()
            .flat_map(|&(start, end)| start..=end)
            .take(count)
            .collect();
        let mut next = numbers.last().map_or(1, |n| n + 1);
        while numbers.len() < count {
            numbers.push(next);
            next += 1;
        }
        numbers
    }

    /// `<figcaption>` HTML: `title` if given (`""` = none), else the file path
    /// as written plus the line ranges.
    fn caption(&self) -> Option<String> {
        let (title, lines) = match (&self.title, &self.file) {
            (Some(title), _) if title.is_empty() => return None,
            (Some(title), _) => (title.as_str(), None),
            (None, Some(file)) => (file.as_str(), ranges_label(&self.ranges)),
            (None, None) => return None,
        };
        let mut html = format!(
            "<figcaption><span class=\"code-block-title\">{}</span>",
            html_escape(title)
        );
        if let Some(lines) = lines {
            html.push_str(&format!("<span class=\"code-block-lines\">{lines}</span>"));
        }
        html.push_str("</figcaption>");
        Some(html)
    }
}

/// `line 7` or `lines 1–6, 30–35`.
fn ranges_label(ranges: &[(usize, usize)]) -> Option<String> {
    match ranges {
        [] => None,
        [(start, end)] if start == end => Some(format!("line {start}")),
        _ => {
            let parts: Vec<String> = ranges
                .iter()
                .map(|&(start, end)| {
                    if start == end {
                        start.to_string()
                    } else {
                        format!("{start}–{end}")
                    }
                })
                .collect();
            Some(format!("lines {}", parts.join(", ")))
        }
    }
}

fn pct_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_./~".contains(&byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn pct_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = (bytes[i] == b'%' && i + 2 < bytes.len())
            .then(|| std::str::from_utf8(&bytes[i + 1..i + 3]).ok())
            .flatten()
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match hex {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::syntax::{SyntaxHighlighter, highlight_code_blocks};

    fn text_content(html: &str) -> String {
        let mut out = String::new();
        let mut in_tag = false;
        for c in html.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => out.push(c),
                _ => {}
            }
        }
        out
    }

    #[test]
    fn untouched_blocks_are_byte_identical() {
        for input in [
            "<pre><code class=\"language-rust\">x\n</code></pre>",
            "<pre><code>plain\n</code></pre>",
            "<pre style=\"background-color:#2b303b;\">\n<span style=\"color:#c0c5ce;\">x\n</span></pre>\n",
            "<p>no code</p>",
        ] {
            assert_eq!(process(input, false), input);
        }
        let mermaid = "<pre class=\"mermaid\">graph TD</pre>";
        assert_eq!(process(mermaid, true), mermaid);
    }

    #[test]
    fn foreign_meta_is_stripped() {
        assert_eq!(
            process(
                "<pre><code class=\"language-js\" data-meta=\"{1,3}\">x\n</code></pre>",
                false
            ),
            "<pre><code class=\"language-js\">x\n</code></pre>"
        );
        assert_eq!(
            process(
                "<pre style=\"background-color:#2b303b;\" data-meta=\"ignore\">\nx</pre>",
                false
            ),
            "<pre style=\"background-color:#2b303b;\">\nx</pre>"
        );
    }

    #[test]
    fn site_default_numbers_plain_blocks() {
        assert_eq!(
            process(
                "<pre><code class=\"language-text\">a\nb\n</code></pre>",
                true
            ),
            "<pre class=\"line-numbers\" data-line-digits=\"1\"><code class=\"language-text\"><span class=\"line\" data-line=\"1\">a</span>\n<span class=\"line\" data-line=\"2\">b</span>\n</code></pre>"
        );
        assert!(
            process("<pre><code>a\n</code></pre>", true).starts_with("<pre class=\"line-numbers\"")
        );
    }

    #[test]
    fn numbers_highlighted_blocks() {
        let h = SyntaxHighlighter::new("base16-ocean.dark");
        let html = highlight_code_blocks(
            "<pre><code class=\"language-rust\" data-meta=\"docanvil numbers=on\">fn a() {\n    let s = &quot;x\ny&quot;;\n}\n</code></pre>",
            &h,
        );
        let out = process(&html, false);
        assert!(out.starts_with("<pre style=\"background-color:"), "{out}");
        assert!(
            out.contains("class=\"line-numbers\" data-line-digits=\"1\">\n<span class=\"line\" data-line=\"1\">"),
            "{out}"
        );
        assert_eq!(out.matches("<span class=\"line\" ").count(), 4);
        for line in out
            .lines()
            .filter(|l| l.starts_with("<span class=\"line\""))
        {
            assert_eq!(
                line.matches("<span").count(),
                line.matches("</span>").count(),
                "{line}"
            );
        }
        assert!(!out.contains("data-meta"));
        // syntect escapes quotes; a browser's textContent decodes them back.
        assert_eq!(
            text_content(&out),
            "\nfn a() {\n    let s = &quot;x\ny&quot;;\n}\n"
        );
    }

    #[test]
    fn numbers_start_and_off() {
        let out = process(
            "<pre><code class=\"language-text\" data-meta=\"docanvil numbers=on start=998\">a\nb\nc\n</code></pre>",
            false,
        );
        assert!(out.contains("data-line-digits=\"4\""), "{out}");
        assert!(out.contains("<span class=\"line\" data-line=\"998\">a</span>"));
        assert!(out.contains("<span class=\"line\" data-line=\"1000\">c</span>"));

        assert_eq!(
            process(
                "<pre><code class=\"language-text\" data-meta=\"docanvil numbers=off\">a\n</code></pre>",
                true
            ),
            "<pre><code class=\"language-text\">a\n</code></pre>"
        );
    }

    #[test]
    fn file_block_with_ranges_shows_gap_and_caption() {
        let out = process(
            "<pre><code class=\"language-rust\" data-meta=\"docanvil ranges=2-3,7-7 file=src.rs\">one\ntwo\nthree\n</code></pre>",
            false,
        );
        assert_eq!(
            out,
            "<figure class=\"code-block\"><figcaption><span class=\"code-block-title\">src.rs</span><span class=\"code-block-lines\">lines 2–3, 7</span></figcaption><pre class=\"line-numbers\" data-line-digits=\"1\"><code class=\"language-rust\"><span class=\"line\" data-line=\"2\">one</span>\n<span class=\"line\" data-line=\"3\">two</span>\n<span class=\"line-gap\" data-hidden=\"3\"></span><span class=\"line\" data-line=\"7\">three</span>\n</code></pre></figure>"
        );
        assert_eq!(text_content(&out), "src.rslines 2–3, 7one\ntwo\nthree\n");
    }

    #[test]
    fn adjacent_ranges_have_no_gap_and_gaps_survive_numbers_off() {
        let adjacent = process(
            "<pre><code class=\"language-text\" data-meta=\"docanvil ranges=1-2,3-3 file=a.txt\">a\nb\nc\n</code></pre>",
            false,
        );
        assert!(!adjacent.contains("line-gap"));

        let off = process(
            "<pre><code class=\"language-text\" data-meta=\"docanvil numbers=off ranges=1-1,5-5 file=a.txt\">a\nb\n</code></pre>",
            false,
        );
        assert!(!off.contains("line-numbers"), "{off}");
        assert!(
            off.contains("<span class=\"line-gap\" data-hidden=\"3\"></span>"),
            "{off}"
        );
    }

    #[test]
    fn captions() {
        let titled = process(
            "<pre><code class=\"language-rust\" data-meta=\"docanvil ranges=1-1 file=a.rs title=Server%20setup\">x\n</code></pre>",
            false,
        );
        assert!(
            titled.contains(
                "<figcaption><span class=\"code-block-title\">Server setup</span></figcaption>"
            ),
            "{titled}"
        );

        let single = process(
            "<pre><code class=\"language-rust\" data-meta=\"docanvil ranges=7-7 file=a.rs\">x\n</code></pre>",
            false,
        );
        assert!(
            single.contains("<span class=\"code-block-lines\">line 7</span>"),
            "{single}"
        );

        assert_eq!(
            process(
                "<pre><code class=\"language-rust\" data-meta=\"docanvil title= file=a.rs\">x\n</code></pre>",
                false
            ),
            "<pre><code class=\"language-rust\">x\n</code></pre>"
        );

        let plain = process(
            "<pre><code class=\"language-rust\" data-meta=\"docanvil title=Example\">x\n</code></pre>",
            false,
        );
        assert!(
            plain.starts_with(
                "<figure class=\"code-block\"><figcaption><span class=\"code-block-title\">Example</span></figcaption><pre><code"
            ),
            "{plain}"
        );

        let escaped = process(
            "<pre><code class=\"language-rust\" data-meta=\"docanvil file=%3Cb%3E%26.rs\">x\n</code></pre>",
            false,
        );
        assert!(escaped.contains("&lt;b&gt;&amp;.rs"), "{escaped}");
    }

    #[test]
    fn meta_round_trips() {
        let meta = BlockMeta {
            numbers: Some(true),
            start: Some(10),
            ranges: vec![(1, 6), (30, 35)],
            file: Some("/examples/my server.rs".to_string()),
            title: Some("A \"quoted\" `title`, 100%".to_string()),
        };
        let encoded = meta.encode();
        assert!(encoded.starts_with("docanvil "), "{encoded}");
        assert!(
            !encoded.contains('"') && !encoded.contains('`'),
            "{encoded}"
        );
        assert_eq!(BlockMeta::parse(&encoded), Some(meta));
    }

    #[test]
    fn encoding_is_compact() {
        let off = BlockMeta {
            numbers: Some(false),
            ..BlockMeta::default()
        };
        assert_eq!(off.encode(), "docanvil numbers=off");
        let hidden = BlockMeta {
            title: Some(String::new()),
            file: Some("/a.rs".to_string()),
            ranges: vec![(2, 3), (7, 7)],
            ..BlockMeta::default()
        };
        assert_eq!(hidden.encode(), "docanvil ranges=2-3,7-7 file=/a.rs title=");
        assert_eq!(BlockMeta::parse(&hidden.encode()), Some(hidden));
    }

    #[test]
    fn foreign_meta_is_ignored() {
        assert_eq!(BlockMeta::parse("{1,3}"), None);
        assert_eq!(BlockMeta::parse("ignore"), None);
        assert_eq!(BlockMeta::parse("docanvil"), Some(BlockMeta::default()));
    }
}
