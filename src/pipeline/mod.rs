pub mod attributes;
pub mod code_blocks;
pub mod directives;
pub mod frontmatter;
pub mod headings;
pub mod images;
pub mod includes;
pub mod markdown;
pub mod popovers;
pub mod syntax;
pub mod wikilinks;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::components::ComponentRegistry;
use crate::diagnostics;
use crate::error::Result;
use crate::project::PageInventory;

use self::includes::IncludeContext;
use self::syntax::SyntaxHighlighter;

/// A rendered page body, plus every file pulled in through `:::include` and
/// `file="…"` code blocks (the dev server watches these).
#[derive(Debug)]
pub struct Processed {
    pub html: String,
    pub dependencies: BTreeSet<PathBuf>,
}

/// Full pipeline: includes → directives + popovers + markdown (via the component
/// registry) → syntax highlight → code block numbering/captions → wiki-links →
/// attributes → heading IDs → image paths.
/// When `locale` is provided, wiki-links resolve within that locale only and
/// includes prefer `name.{locale}.md`. `line_numbers` is `[syntax] line_numbers`.
#[allow(clippy::too_many_arguments)]
pub fn process(
    source: &str,
    inventory: &PageInventory,
    source_file: &Path,
    registry: &ComponentRegistry,
    base_url: &str,
    highlighter: Option<&SyntaxHighlighter>,
    line_numbers: bool,
    project_root: &Path,
    locale: Option<&str>,
) -> Result<Processed> {
    // 1. Splice :::include fragments and fill file="…" code blocks
    let expanded = includes::expand(
        source,
        source_file,
        &IncludeContext {
            project_root,
            locale,
        },
    );
    for problem in &expanded.problems {
        diagnostics::warn_include(
            &problem.file,
            problem.line,
            &problem.message,
            problem.hint.as_deref(),
        );
    }

    // 2. Directives (nested components), popovers, {#id} extraction, then comrak
    let html = registry.render_markdown(&expanded.source, source_file);

    // 3. Syntax-highlight code blocks (if enabled)
    let html = match highlighter {
        Some(h) => syntax::highlight_code_blocks(&html, h),
        None => html,
    };

    // 4. Line numbers, hidden-line gaps and captions
    let html = code_blocks::process(&html, line_numbers);

    // 5. Resolve wiki-links
    let html = wikilinks::resolve(&html, inventory, source_file, base_url, locale);

    // 6. Post-comrak: inject inline attributes ({.class})
    let html = attributes::inject_attributes(&html);

    // 7. Auto-generate heading IDs (after attributes so manual {#id} wins)
    let html = headings::inject_heading_ids(&html);

    // 8. Rewrite relative image paths with base_url
    let html = images::rewrite_image_paths(&html, base_url, project_root);

    Ok(Processed {
        html,
        dependencies: expanded.dependencies,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{reset_warnings, warning_count};

    fn site(files: &[(&str, &str)]) -> (tempfile::TempDir, PageInventory) {
        let dir = tempfile::tempdir().unwrap();
        for (path, content) in files {
            let p = dir.path().join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, content).unwrap();
        }
        let inventory = PageInventory::scan(&dir.path().join("docs"), None, None, None).unwrap();
        (dir, inventory)
    }

    fn run(dir: &Path, inventory: &PageInventory, line_numbers: bool) -> Processed {
        let file = dir.join("docs/index.md");
        let source = std::fs::read_to_string(&file).unwrap();
        process(
            &source,
            inventory,
            &file,
            &ComponentRegistry::with_builtins(),
            "/",
            None,
            line_numbers,
            dir,
            None,
        )
        .unwrap()
    }

    #[test]
    fn includes_and_file_blocks_render() {
        let (dir, inv) = site(&[
            (
                "docs/index.md",
                "# Home\n\n:::include{file=\"_shared/intro.md\"}\n\n```rust file=\"/examples/a.rs\" lines=\"1,3\"\n```\n",
            ),
            ("docs/_shared/intro.md", "## Intro\n\nShared words.\n"),
            ("examples/a.rs", "fn a() {}\n// skipped\nfn b() {}\n"),
        ]);
        let out = run(dir.path(), &inv, false);
        assert!(out.html.contains("id=\"intro\""), "{}", out.html);
        assert!(out.html.contains("Shared words."));
        assert!(
            out.html
                .contains("<span class=\"line\" data-line=\"3\">fn b() {}</span>"),
            "{}",
            out.html
        );
        assert!(out.html.contains("data-hidden=\"1\""));
        assert!(out.html.contains("<figcaption>"));
        assert!(!out.html.contains("data-meta"));
        assert_eq!(out.dependencies.len(), 2);
    }

    #[test]
    fn include_problems_are_warnings() {
        reset_warnings();
        let (dir, inv) = site(&[(
            "docs/index.md",
            "# Home\n\n:::include{file=\"_missing.md\"}\n",
        )]);
        let out = run(dir.path(), &inv, false);
        assert_eq!(warning_count(), 1);
        assert!(
            out.html.contains("<div class=\"include-error\">"),
            "{}",
            out.html
        );
    }

    #[test]
    fn mid_line_include_stays_literal_text() {
        let (dir, inv) = site(&[
            ("docs/index.md", "See :::include{file=\"_x.md\"} here.\n"),
            ("docs/_x.md", "X"),
        ]);
        let out = run(dir.path(), &inv, false);
        assert!(out.html.contains(":::include{file="), "{}", out.html);
        assert!(!out.html.contains("class=\"include\""), "{}", out.html);
    }

    /// A plain (unhighlighted) numbered block — unknown language here, but the
    /// same applies with highlighting disabled — must not leave a doubled
    /// trailing newline or stray blank line before the next block.
    #[test]
    fn unhighlighted_numbered_block_has_no_stray_blank_line() {
        let (dir, inv) = site(&[("docs/index.md", "```zzzlang numbers\na\nb\n```\n\nAfter\n")]);
        let out = run(dir.path(), &inv, false);
        assert!(
            out.html.contains("</code></pre><p>After</p>"),
            "{}",
            out.html
        );
        assert!(!out.html.contains("</pre>\n\n"), "{}", out.html);
        assert!(!out.html.contains("<p></p>"), "{}", out.html);
    }

    #[test]
    fn site_wide_line_numbers() {
        let (dir, inv) = site(&[("docs/index.md", "```text\na\nb\n```\n")]);
        let out = run(dir.path(), &inv, true);
        assert!(
            out.html.contains("<pre class=\"line-numbers\" data-line-digits=\"1\"><code class=\"language-text\"><span class=\"line\" data-line=\"1\">a</span>"),
            "{}",
            out.html
        );
    }
}
