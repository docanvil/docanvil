//! `llms.txt` and `llms-full.txt`: an index of the docs for AI tools, and every
//! page's Markdown in one file (<https://llmstxt.org>).

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;

use regex::{Captures, Regex};

use crate::pipeline::code_blocks::META_MARKER;
use crate::pipeline::directives::{FenceState, inline_code_ranges};
use crate::pipeline::frontmatter::{self, FrontMatter};
use crate::pipeline::images;
use crate::pipeline::wikilinks;
use crate::project::{self, NavNode, PageInfo, PageInventory};

/// DocAnvil's `docanvil key=value …` meta after a fence's language.
static FENCE_META_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"\s+{META_MARKER}(?:\s.*)?$")).unwrap());
/// An ATX heading ending in a `{#id .class}` attribute block.
static HEADING_ATTRS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^( {0,3}#{1,6}\s.*?)\s*\{\s*[#.][^{}]*\}\s*$").unwrap());
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

/// Clean one line outside code blocks: resolve its wiki-links (whose display
/// text may hold inline code) and rewrite images outside inline code spans.
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
    for (start, end) in wikilinks::find_links(&line, &inline_code_ranges(&line)) {
        out.push_str(&clean_text(&line[last..start], base, project_root));
        out.push_str(&link_markdown(
            &line[start + 2..end - 2],
            inventory,
            locale,
            base,
        ));
        last = end;
    }
    out.push_str(&clean_text(&line[last..], base, project_root));
    out
}

/// A wiki-link's inner text as `[display](url)`, or just its display text when
/// the target is missing or a draft.
fn link_markdown(
    inner: &str,
    inventory: &PageInventory,
    locale: Option<&str>,
    base: &str,
) -> String {
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
}

/// Rewrite relative images in text with no wiki-links in it, skipping its
/// inline code spans.
fn clean_text(text: &str, base: &str, project_root: &Path) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for (start, end) in inline_code_ranges(text) {
        out.push_str(&rewrite_images(&text[last..start], base, project_root));
        out.push_str(&text[start..end]);
        last = end;
    }
    out.push_str(&rewrite_images(&text[last..], base, project_root));
    out
}

/// Rewrite relative images in text with no code in it.
fn rewrite_images(text: &str, base: &str, project_root: &Path) -> String {
    IMAGE_RE
        .replace_all(text, |caps: &Captures| {
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

/// Section for top-level pages before any labelled nav separator.
const DOCS_SECTION: &str = "Docs";
/// Section for published pages the nav doesn't list.
const OTHER_SECTION: &str = "Other pages";

/// One (version, locale) part of the site and the pages it lists.
#[derive(Debug)]
pub struct LlmsScope {
    /// Output folder relative to the site root: `""`, `"fr/"`, `"v2/"` or `"v2/fr/"`.
    pub dir: String,
    /// How the root file's Optional section names it (e.g. `"v1.0 · Français"`).
    pub label: String,
    pub nav: Vec<NavNode>,
    pub pages: Vec<LlmsPage>,
}

/// A `## heading` in `llms.txt` and the pages under it.
#[derive(Debug)]
pub struct Section<'a> {
    pub title: String,
    pub pages: Vec<&'a LlmsPage>,
}

/// Group `pages` by the nav: each top-level group is a section (nested groups
/// flattened), top-level pages go under the labelled separator above them (or
/// "Docs"), and pages the nav doesn't list go last under "Other pages". Each
/// page appears once; empty sections are dropped.
pub fn sections_from_nav<'a>(nav: &[NavNode], pages: &'a [LlmsPage]) -> Vec<Section<'a>> {
    let by_slug: HashMap<&str, &'a LlmsPage> = pages.iter().map(|p| (p.slug.as_str(), p)).collect();
    let mut listed: HashSet<String> = HashSet::new();
    let mut sections: Vec<Section<'a>> = Vec::new();
    let mut loose_title = DOCS_SECTION.to_string();
    // Index of the section collecting top-level pages, once it has one.
    let mut loose_at: Option<usize> = None;

    for node in nav {
        match node {
            NavNode::Page { slug, .. } => {
                if let Some(page) = take(slug, &by_slug, &mut listed) {
                    let at = *loose_at.get_or_insert_with(|| {
                        sections.push(Section {
                            title: loose_title.clone(),
                            pages: Vec::new(),
                        });
                        sections.len() - 1
                    });
                    sections[at].pages.push(page);
                }
            }
            NavNode::Group {
                label,
                slug,
                children,
            } => {
                let slugs = slug.iter().cloned().chain(
                    project::flatten_nav_pages(children)
                        .into_iter()
                        .map(|(slug, _)| slug),
                );
                let pages = slugs
                    .filter_map(|s| take(&s, &by_slug, &mut listed))
                    .collect();
                sections.push(Section {
                    title: label.clone(),
                    pages,
                });
            }
            NavNode::Separator { label: Some(label) } => {
                loose_title = label.clone();
                loose_at = None;
            }
            NavNode::Separator { label: None } => {}
        }
    }

    let others = pages.iter().filter(|p| !listed.contains(&p.slug)).collect();
    sections.push(Section {
        title: OTHER_SECTION.to_string(),
        pages: others,
    });
    sections.retain(|s| !s.pages.is_empty());
    sections
}

/// The page for `slug`, unless it isn't listed or was already taken.
fn take<'a>(
    slug: &str,
    by_slug: &HashMap<&str, &'a LlmsPage>,
    listed: &mut HashSet<String>,
) -> Option<&'a LlmsPage> {
    let page = *by_slug.get(slug)?;
    listed.insert(slug.to_string()).then_some(page)
}

/// `llms.txt`: title, optional summary, a link list per section, then the
/// `## Optional` links (other versions and languages) when there are any.
pub fn generate_index(
    name: &str,
    description: Option<&str>,
    sections: &[Section],
    optional: &[(String, String)],
) -> String {
    let mut out = header(name, description);
    for section in sections {
        out.push_str(&format!("\n## {}\n\n", section.title));
        for page in &section.pages {
            out.push_str(&format!("- [{}]({})", link_text(&page.title), page.url));
            if let Some(description) = page
                .description
                .as_deref()
                .map(one_line)
                .filter(|d| !d.is_empty())
            {
                out.push_str(&format!(": {description}"));
            }
            out.push('\n');
        }
    }
    if !optional.is_empty() {
        out.push_str("\n## Optional\n\n");
        for (label, url) in optional {
            out.push_str(&format!("- [{}]({url})\n", link_text(label)));
        }
    }
    out
}

/// `llms-full.txt`: title, optional summary, then every page in `sections`
/// order, each starting with its `# title` and a `Source:` line.
pub fn generate_full(name: &str, description: Option<&str>, sections: &[Section]) -> String {
    let mut out = header(name, description);
    for page in sections.iter().flat_map(|s| s.pages.iter()) {
        out.push('\n');
        out.push_str(&full_page(page));
        out.push_str("\n---\n");
    }
    out
}

/// One page in `llms-full.txt`. A page that opens with its own `# H1` keeps it
/// (no duplicate title); otherwise `# {title}` is added.
fn full_page(page: &LlmsPage) -> String {
    let markdown = page.markdown.trim();
    let first_line = markdown.lines().next().unwrap_or("");
    let (title, body) = if first_line.starts_with("# ") {
        (
            first_line.to_string(),
            markdown[first_line.len()..].trim_start(),
        )
    } else {
        (format!("# {}", page.title), markdown)
    };
    let mut out = format!("{title}\nSource: {}\n", page.url);
    if !body.is_empty() {
        out.push('\n');
        out.push_str(body);
        out.push('\n');
    }
    out
}

/// `# name`, plus `> description` when there is one.
fn header(name: &str, description: Option<&str>) -> String {
    let mut out = format!("# {name}\n");
    if let Some(description) = description.map(one_line).filter(|d| !d.is_empty()) {
        out.push_str(&format!("\n> {description}\n"));
    }
    out
}

/// Escape brackets so a title can't end a Markdown link early.
fn link_text(text: &str) -> String {
    text.replace('[', "\\[").replace(']', "\\]")
}

/// Collapse whitespace (including newlines) so the text stays on one line.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
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
    fn wiki_links_with_inline_code_in_their_text() {
        let (dir, inv) = site(&["index.md", "guide/setup.md"], None);
        assert_eq!(
            clean(
                "See [[guide/setup|the `setup` page]], [[missing|the `gone` page]] and `[[nav]]`.\n",
                &inv,
                dir.path()
            ),
            "See [the `setup` page](https://x.dev/guide/setup.html), the `gone` page and `[[nav]]`.\n"
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

    fn entry(slug: &str, title: &str) -> LlmsPage {
        LlmsPage {
            slug: slug.into(),
            title: title.into(),
            url: format!("https://x.dev/{slug}.html"),
            description: None,
            markdown: format!("# {title}\n\nBody of {title}.\n"),
        }
    }

    fn nav_page(slug: &str) -> NavNode {
        NavNode::Page {
            label: slug.into(),
            slug: slug.into(),
        }
    }

    fn summary(sections: &[Section]) -> Vec<(String, Vec<String>)> {
        sections
            .iter()
            .map(|s| {
                (
                    s.title.clone(),
                    s.pages.iter().map(|p| p.slug.clone()).collect(),
                )
            })
            .collect()
    }

    fn owned(title: &str, slugs: &[&str]) -> (String, Vec<String>) {
        (
            title.to_string(),
            slugs.iter().map(|s| s.to_string()).collect(),
        )
    }

    #[test]
    fn groups_loose_pages_and_other_pages() {
        let nav = vec![
            nav_page("index"),
            NavNode::Group {
                label: "Guide".into(),
                slug: Some("guide".into()),
                children: vec![
                    nav_page("guide/a"),
                    NavNode::Group {
                        label: "Deep".into(),
                        slug: None,
                        children: vec![nav_page("guide/deep/b")],
                    },
                ],
            },
            nav_page("faq"),
        ];
        let pages: Vec<LlmsPage> = ["index", "guide", "guide/a", "guide/deep/b", "faq", "orphan"]
            .iter()
            .map(|s| entry(s, s))
            .collect();
        assert_eq!(
            summary(&sections_from_nav(&nav, &pages)),
            vec![
                owned("Docs", &["index", "faq"]),
                owned("Guide", &["guide", "guide/a", "guide/deep/b"]),
                owned("Other pages", &["orphan"]),
            ]
        );
    }

    #[test]
    fn labelled_separators_start_sections() {
        let nav = vec![
            nav_page("a"),
            NavNode::Separator {
                label: Some("Reference".into()),
            },
            nav_page("b"),
            NavNode::Separator { label: None },
            nav_page("c"),
        ];
        let pages: Vec<LlmsPage> = ["a", "b", "c"].iter().map(|s| entry(s, s)).collect();
        assert_eq!(
            summary(&sections_from_nav(&nav, &pages)),
            vec![owned("Docs", &["a"]), owned("Reference", &["b", "c"])]
        );
    }

    #[test]
    fn missing_duplicate_and_empty() {
        // "excluded" is in the nav but not a listed page (llms: false); "a" appears twice.
        let nav = vec![
            nav_page("a"),
            NavNode::Group {
                label: "Empty".into(),
                slug: None,
                children: vec![nav_page("excluded")],
            },
            nav_page("a"),
        ];
        let pages = vec![entry("a", "A")];
        assert_eq!(
            summary(&sections_from_nav(&nav, &pages)),
            vec![owned("Docs", &["a"])]
        );
    }

    #[test]
    fn index_with_description_and_optional() {
        let pages = vec![LlmsPage {
            description: Some("First\n  page".into()),
            ..entry("a", "A [beta]")
        }];
        let sections = sections_from_nav(&[nav_page("a")], &pages);
        let optional = vec![(
            "Français".to_string(),
            "https://x.dev/fr/llms.txt".to_string(),
        )];
        assert_eq!(
            generate_index("Proj", Some(" Summary.\n"), &sections, &optional),
            "# Proj\n\n> Summary.\n\n## Docs\n\n- [A \\[beta\\]](https://x.dev/a.html): First page\n\n## Optional\n\n- [Français](https://x.dev/fr/llms.txt)\n"
        );
    }

    #[test]
    fn index_without_extras() {
        let pages = vec![LlmsPage {
            description: Some("  ".into()),
            ..entry("a", "A")
        }];
        let sections = sections_from_nav(&[nav_page("a")], &pages);
        assert_eq!(
            generate_index("Proj", None, &sections, &[]),
            "# Proj\n\n## Docs\n\n- [A](https://x.dev/a.html)\n"
        );
    }

    #[test]
    fn full_text_reuses_or_adds_the_title() {
        let pages = vec![
            entry("a", "A"),
            LlmsPage {
                markdown: "\nIntro without heading.\n".into(),
                ..entry("b", "B")
            },
            LlmsPage {
                markdown: "# Only a title\n".into(),
                ..entry("c", "C")
            },
        ];
        let sections = sections_from_nav(&[nav_page("a"), nav_page("b"), nav_page("c")], &pages);
        assert_eq!(
            generate_full("Proj", Some("Sum."), &sections),
            "# Proj\n\n> Sum.\n\
             \n# A\nSource: https://x.dev/a.html\n\nBody of A.\n\n---\n\
             \n# B\nSource: https://x.dev/b.html\n\nIntro without heading.\n\n---\n\
             \n# Only a title\nSource: https://x.dev/c.html\n\n---\n"
        );
    }
}
