//! `llms.txt` and `llms-full.txt`: an index of the docs for AI tools, and every
//! page's Markdown in one file (<https://llmstxt.org>).

use std::path::Path;
use std::sync::LazyLock;

use regex::{Captures, Regex};

use crate::pipeline::code_blocks::META_MARKER;
use crate::pipeline::directives::{FenceState, inline_code_ranges};
use crate::pipeline::frontmatter::{self, FrontMatter};
use crate::pipeline::images;
use crate::project::{PageInfo, PageInventory};

/// DocAnvil's `docanvil key=value …` meta after a fence's language.
static FENCE_META_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"\s+{META_MARKER}(?:\s.*)?$")).unwrap());
/// An ATX heading ending in a `{#id .class}` attribute block.
static HEADING_ATTRS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^( {0,3}#{1,6}\s.*?)\s*\{\s*[#.][^{}]*\}\s*$").unwrap());
static WIKILINK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[(.*?)\]\]").unwrap());
static IMAGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"!\[([^\]]*)\]\(([^)\s]+)(\s+"[^"]*")?\)"#).unwrap());

/// One page as listed in `llms.txt`.
#[derive(Debug, Clone)]
pub struct LlmsPage {
    /// Base slug, as used in the nav tree.
    pub slug: String,
    pub title: String,
    /// The page's HTML, absolute when `site_url` is set.
    pub url: String,
    pub description: Option<String>,
    /// Cleaned Markdown (see [`clean_markdown`]).
    pub markdown: String,
}

/// A page's URL: `base` (`site_url` or `base_url`) plus its output path.
pub fn page_url(base: &str, page: &PageInfo) -> String {
    format!(
        "{base}{}",
        page.output_path.to_string_lossy().replace('\\', "/")
    )
}

/// One page's entry: URL, front matter description and cleaned Markdown.
pub fn page_entry(
    page: &PageInfo,
    fm: &FrontMatter,
    markdown: &str,
    inventory: &PageInventory,
    locale: Option<&str>,
    base: &str,
    project_root: &Path,
) -> LlmsPage {
    LlmsPage {
        slug: page.slug.clone(),
        title: page.title.clone(),
        url: page_url(base, page),
        description: fm.description.clone(),
        markdown: clean_markdown(markdown, inventory, locale, base, project_root),
    }
}

/// Turn a page's include-expanded source into Markdown other tools can read:
/// front matter dropped, `[[wiki-links]]` resolved to `[text](url)` (plain text
/// when the target is missing or a draft), DocAnvil's fence meta and heading
/// `{#id .class}` blocks removed, and relative images made absolute. Code blocks
/// and inline code are left alone; components and popovers pass through.
/// Never warns: the HTML pass has already reported broken links.
pub fn clean_markdown(
    source: &str,
    inventory: &PageInventory,
    locale: Option<&str>,
    base: &str,
    project_root: &Path,
) -> String {
    let body = frontmatter::strip(source);
    let mut out = String::with_capacity(body.len());
    let mut fence = FenceState::default();
    for line in body.split_inclusive('\n') {
        let text = line.trim_end_matches(['\n', '\r']);
        let eol = &line[text.len()..];
        let was_open = fence.is_open();
        if fence.consume(text) {
            if was_open {
                out.push_str(text);
            } else {
                // Opening fence: keep the language, drop DocAnvil's meta.
                out.push_str(&FENCE_META_RE.replace(text, ""));
            }
        } else {
            out.push_str(&clean_line(text, inventory, locale, base, project_root));
        }
        out.push_str(eol);
    }
    out
}

/// Clean one line outside code blocks, skipping its inline code spans.
fn clean_line(
    line: &str,
    inventory: &PageInventory,
    locale: Option<&str>,
    base: &str,
    project_root: &Path,
) -> String {
    let line = HEADING_ATTRS_RE.replace(line, "$1");
    let mut out = String::with_capacity(line.len());
    let mut last = 0;
    for (start, end) in inline_code_ranges(&line) {
        out.push_str(&clean_text(
            &line[last..start],
            inventory,
            locale,
            base,
            project_root,
        ));
        out.push_str(&line[start..end]);
        last = end;
    }
    out.push_str(&clean_text(
        &line[last..],
        inventory,
        locale,
        base,
        project_root,
    ));
    out
}

/// Resolve wiki-links and rewrite relative images in text with no code in it.
fn clean_text(
    text: &str,
    inventory: &PageInventory,
    locale: Option<&str>,
    base: &str,
    project_root: &Path,
) -> String {
    let text = WIKILINK_RE.replace_all(text, |caps: &Captures| {
        let inner = &caps[1];
        let (target, display) = match inner.split_once('|') {
            Some((target, display)) => (target.trim(), display.trim()),
            None => (inner.trim(), inner.trim()),
        };
        let resolved = match locale {
            Some(l) => inventory.resolve_link_in_locale(target, l),
            None => inventory.resolve_link(target),
        };
        match resolved {
            Some(page) => format!("[{display}]({})", page_url(base, page)),
            None => display.to_string(),
        }
    });
    IMAGE_RE
        .replace_all(&text, |caps: &Captures| {
            match images::rewrite_src(&caps[2], base, project_root) {
                Some(src) => format!(
                    "![{}]({src}{})",
                    &caps[1],
                    caps.get(3).map_or("", |m| m.as_str())
                ),
                None => caps[0].to_string(),
            }
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://x.dev/";

    fn site(files: &[&str], locales: Option<&[String]>) -> (tempfile::TempDir, PageInventory) {
        let dir = tempfile::tempdir().unwrap();
        for file in files {
            let path = dir.path().join("docs").join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, "# Page\n").unwrap();
        }
        let default = locales.map(|_| "en");
        let inventory =
            PageInventory::scan(&dir.path().join("docs"), locales, default, None).unwrap();
        (dir, inventory)
    }

    fn clean(source: &str, inventory: &PageInventory, root: &Path) -> String {
        clean_markdown(source, inventory, None, BASE, root)
    }

    #[test]
    fn strips_front_matter() {
        let (dir, inv) = site(&["index.md"], None);
        assert_eq!(
            clean("---\n{\"title\": \"X\"}\n---\n# X\n", &inv, dir.path()),
            "# X\n"
        );
    }

    #[test]
    fn wiki_links_become_markdown_links() {
        let (dir, inv) = site(&["index.md", "guide/setup.md"], None);
        assert_eq!(
            clean(
                "See [[guide/setup]], [[guide/setup|setup]] and [[missing|gone]].\n",
                &inv,
                dir.path()
            ),
            "See [guide/setup](https://x.dev/guide/setup.html), [setup](https://x.dev/guide/setup.html) and gone.\n"
        );
    }

    #[test]
    fn wiki_links_resolve_in_the_page_locale() {
        let locales = vec!["en".to_string(), "fr".to_string()];
        let (dir, inv) = site(
            &["index.en.md", "index.fr.md", "guide.en.md", "guide.fr.md"],
            Some(&locales),
        );
        assert_eq!(
            clean_markdown("[[guide|Guide]]\n", &inv, Some("fr"), BASE, dir.path()),
            "[Guide](https://x.dev/fr/guide.html)\n"
        );
    }

    #[test]
    fn code_is_untouched() {
        let (dir, inv) = site(&["index.md", "guide/setup.md"], None);
        let source = "````md\n```toml\n[[nav]]\n```\n[[guide/setup]]\n````\n~~~\n[[guide/setup]]\n~~~\nUse `[[nav]]` here.\n";
        assert_eq!(clean(source, &inv, dir.path()), source);
    }

    #[test]
    fn fence_meta_is_dropped() {
        let (dir, inv) = site(&["index.md"], None);
        assert_eq!(
            clean(
                "```rust docanvil numbers=on file=a.rs\nfn a() {}\n```\n  ```text docanvil title=Hi\nx\n  ```\n",
                &inv,
                dir.path()
            ),
            "```rust\nfn a() {}\n```\n  ```text\nx\n  ```\n"
        );
    }

    #[test]
    fn heading_attributes_are_dropped() {
        let (dir, inv) = site(&["index.md"], None);
        assert_eq!(
            clean(
                "## Install {#setup .wide}\n# Top {.hero}\nText {#not-a-heading}\n#hashtag {#x}\n",
                &inv,
                dir.path()
            ),
            "## Install\n# Top\nText {#not-a-heading}\n#hashtag {#x}\n"
        );
    }

    #[test]
    fn relative_images_become_absolute() {
        let (dir, inv) = site(&["index.md"], None);
        std::fs::create_dir_all(dir.path().join("assets")).unwrap();
        std::fs::write(dir.path().join("assets/logo.png"), b"x").unwrap();
        assert_eq!(
            clean(
                "![Logo](logo.png \"The logo\") ![Ext](https://e.com/a.png) `![Code](logo.png)`\n",
                &inv,
                dir.path()
            ),
            "![Logo](https://x.dev/assets/logo.png \"The logo\") ![Ext](https://e.com/a.png) `![Code](logo.png)`\n"
        );
    }

    #[test]
    fn crlf_lines_are_cleaned() {
        let (dir, inv) = site(&["index.md", "guide/setup.md"], None);
        assert_eq!(
            clean(
                "# A {#a}\r\n[[guide/setup|s]]\r\n```rust docanvil numbers=on\r\n[[x]]\r\n```\r\n",
                &inv,
                dir.path()
            ),
            "# A\r\n[s](https://x.dev/guide/setup.html)\r\n```rust\r\n[[x]]\r\n```\r\n"
        );
    }

    #[test]
    fn page_entry_uses_url_title_and_description() {
        let (dir, inv) = site(&["index.md", "guide/setup.md"], None);
        let page = inv.resolve_link("guide/setup").unwrap().clone();
        let fm = FrontMatter {
            description: Some("Get going".into()),
            ..Default::default()
        };
        let entry = page_entry(
            &page,
            &fm,
            "---\n{}\n---\n# Setup\n",
            &inv,
            None,
            BASE,
            dir.path(),
        );
        assert_eq!(entry.slug, "guide/setup");
        assert_eq!(entry.url, "https://x.dev/guide/setup.html");
        assert_eq!(entry.description.as_deref(), Some("Get going"));
        assert_eq!(entry.markdown, "# Setup\n");
    }
}
