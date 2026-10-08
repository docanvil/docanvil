//! Code block numbering, hidden-line gaps and captions.

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
