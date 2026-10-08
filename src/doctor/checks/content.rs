use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::doctor::{Diagnostic, Severity};
use crate::pipeline::directives::{FenceState, INCLUDE_DIRECTIVE};
use crate::pipeline::includes::{self, IncludeContext, IncludeProblem};
use crate::project::{self, PageInventory};

static OPEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(:{3,})\s*([\w][\w-]*)\s*(\{.*\})?\s*$").unwrap());

/// Check content: broken wiki-links, unclosed directives, front-matter errors,
/// duplicate slugs, and includes (pages and `_` fragments).
pub fn check_content(
    project_root: &Path,
    config: &Config,
    inventory: &PageInventory,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Check for duplicate slugs (detected by checking if the inventory has fewer entries
    // than files scanned — but since PageInventory uses HashMap, duplicates overwrite silently).
    // We re-scan to detect duplicates.
    check_duplicate_slugs(inventory, &mut diags);

    // Scan each page for content issues
    for slug in &inventory.ordered {
        let page = &inventory.pages[slug];
        let source = match std::fs::read_to_string(&page.source_path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        check_broken_wikilinks(
            &source,
            &page.source_path,
            page.locale.as_deref(),
            inventory,
            &mut diags,
        );
        check_unclosed_directives(&source, &page.source_path, &mut diags);
        check_frontmatter(&source, &page.source_path, &mut diags);
        check_inline_includes(&source, &page.source_path, &mut diags);
    }

    // Fragments aren't pages, but their text ends up on pages — check each once,
    // as written, so diagnostics point at the fragment's own lines.
    let fragments = project::fragment_files(&project_root.join(&config.project.content_dir));
    for path in &fragments {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        check_broken_wikilinks(&source, path, None, inventory, &mut diags);
        check_unclosed_directives(&source, path, &mut diags);
        check_inline_includes(&source, path, &mut diags);
    }

    check_includes(project_root, config, inventory, &fragments, &mut diags);

    diags
}

fn check_duplicate_slugs(inventory: &PageInventory, diags: &mut Vec<Diagnostic>) {
    // Group source paths by their slug to detect if multiple files map to the same slug.
    // Since PageInventory deduplicates, we check the ordered list for duplicates.
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for slug in &inventory.ordered {
        *seen.entry(slug.as_str()).or_insert(0) += 1;
    }
    for (slug, count) in &seen {
        if *count > 1 {
            diags.push(Diagnostic {
                check: "duplicate-slug",
                category: "content",
                severity: Severity::Error,
                message: format!("Duplicate slug: {slug} ({count} files)"),
                file: None,
                line: None,
                fix: None,
            });
        }
    }
}

/// Blank out inline `` `code` `` spans on a single line, replacing the backticks
/// and their content with spaces so `[[…]]` inside them is never matched, while
/// keeping byte offsets (and thus column positions) unchanged.
fn mask_inline_code(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut remaining = line;

    while let Some(start) = remaining.find('`') {
        result.push_str(&remaining[..start]);
        let after = &remaining[start + 1..];
        if let Some(end) = after.find('`') {
            let span = &remaining[start..start + 1 + end + 1];
            result.push_str(&" ".repeat(span.len()));
            remaining = &after[end + 1..];
        } else {
            // No closing backtick on this line — leave the rest untouched.
            result.push_str(&remaining[start..]);
            remaining = "";
            break;
        }
    }

    result.push_str(remaining);
    result
}

fn check_broken_wikilinks(
    source: &str,
    source_path: &Path,
    locale: Option<&str>,
    inventory: &PageInventory,
    diags: &mut Vec<Diagnostic>,
) {
    let mut fence = FenceState::default();

    for (i, line) in source.lines().enumerate() {
        if fence.consume(line) {
            // Inside a fenced code block — code is left exactly as written.
            continue;
        }

        let masked = mask_inline_code(line);
        check_broken_wikilinks_in_line(&masked, i + 1, source_path, locale, inventory, diags);
    }
}

fn check_broken_wikilinks_in_line(
    line: &str,
    line_number: usize,
    source_path: &Path,
    locale: Option<&str>,
    inventory: &PageInventory,
    diags: &mut Vec<Diagnostic>,
) {
    let mut remaining = line;

    while let Some(start) = remaining.find("[[") {
        let after_open = &remaining[start + 2..];
        if let Some(end) = after_open.find("]]") {
            let inner = &after_open[..end];
            // Find the pipe separator. In Markdown table cells, `\|` escapes the pipe but
            // still acts as the target/display separator — strip the trailing `\` from the
            // target so the lookup works correctly.
            let (target, _display) = if let Some(pipe_pos) = inner.find('|') {
                (
                    inner[..pipe_pos].trim_end_matches('\\'),
                    &inner[pipe_pos + 1..],
                )
            } else {
                (inner, inner)
            };

            let target = target.trim();
            let resolved = match locale {
                Some(loc) => inventory
                    .resolve_link_in_locale(target, loc)
                    .or_else(|| inventory.resolve_link(target)),
                None => inventory.resolve_link(target),
            };
            if !target.is_empty() && resolved.is_none() {
                diags.push(Diagnostic {
                    check: "broken-wiki-link",
                    category: "content",
                    severity: Severity::Warning,
                    message: format!("Broken link [[{target}]]"),
                    file: Some(source_path.to_path_buf()),
                    line: Some(line_number),
                    fix: None,
                });
            }

            remaining = &after_open[end + 2..];
        } else {
            break;
        }
    }
}

fn check_unclosed_directives(source: &str, source_path: &Path, diags: &mut Vec<Diagnostic>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut stack: Vec<(String, usize, usize)> = Vec::new(); // (name, colons, line_number)

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check for opening directive
        if let Some(caps) = OPEN_RE.captures(trimmed) {
            let colons = caps[1].len();
            let name = caps[2].to_string();
            // `:::include{…}` lines have no closing fence.
            if name == INCLUDE_DIRECTIVE {
                continue;
            }
            stack.push((name, colons, i + 1));
            continue;
        }

        // Check for closing fence
        if trimmed.starts_with(":::") && trimmed.chars().all(|c| c == ':') && trimmed.len() >= 3 {
            let close_colons = trimmed.len();
            // Find matching open directive (same colon count)
            if let Some(pos) = stack.iter().rposition(|(_n, c, _l)| *c == close_colons) {
                stack.truncate(pos);
            }
        }
    }

    // Any remaining items on the stack are unclosed
    for (name, _colons, line_number) in stack {
        diags.push(Diagnostic {
            check: "unclosed-directive",
            category: "content",
            severity: Severity::Warning,
            message: format!("Unclosed directive :::{name}"),
            file: Some(source_path.to_path_buf()),
            line: Some(line_number),
            fix: None,
        });
    }
}

fn check_frontmatter(source: &str, source_path: &Path, diags: &mut Vec<Diagnostic>) {
    let trimmed = source.trim_start();
    if !trimmed.starts_with("---") {
        return;
    }

    let after_open = &trimmed[3..];
    let rest = after_open
        .strip_prefix('\n')
        .or_else(|| after_open.strip_prefix("\r\n"));
    let Some(rest) = rest else {
        return;
    };

    let Some(end) = rest.find("\n---") else {
        return;
    };

    let content = &rest[..end];
    if let Err(e) = serde_json::from_str::<serde_json::Value>(content) {
        diags.push(Diagnostic {
            check: "frontmatter-parse-error",
            category: "content",
            severity: Severity::Warning,
            message: format!("Front-matter JSON parse error: {e}"),
            file: Some(source_path.to_path_buf()),
            line: Some(1),
            fix: None,
        });
    }
}

fn check_inline_includes(source: &str, source_path: &Path, diags: &mut Vec<Diagnostic>) {
    for line in includes::inline_include_lines(source) {
        diags.push(Diagnostic {
            check: "include-inline",
            category: "content",
            severity: Severity::Warning,
            message: ":::include needs a line of its own — inside other text it's shown as written, not included".to_string(),
            file: Some(source_path.to_path_buf()),
            line: Some(line),
            fix: None,
        });
    }
}

/// Expand every page (and every fragment on its own) the way the build does,
/// reporting each include problem once; then list unused and partly
/// translated fragments.
fn check_includes(
    project_root: &Path,
    config: &Config,
    inventory: &PageInventory,
    fragments: &[PathBuf],
    diags: &mut Vec<Diagnostic>,
) {
    let mut seen: HashSet<(PathBuf, usize)> = HashSet::new();
    let mut used: BTreeSet<PathBuf> = BTreeSet::new();

    for slug in &inventory.ordered {
        let page = &inventory.pages[slug];
        let Ok(source) = std::fs::read_to_string(&page.source_path) else {
            continue;
        };
        let locale = if config.is_i18n_enabled() {
            page.locale.as_deref()
        } else {
            None
        };
        let expanded = includes::expand(
            &source,
            &page.source_path,
            &IncludeContext {
                project_root,
                locale,
            },
        );
        used.extend(expanded.dependencies);
        push_include_problems(expanded.problems, &mut seen, diags);
    }

    // Fragments nothing includes still deserve their includes checked.
    for path in fragments {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        let expanded = includes::expand(
            &source,
            path,
            &IncludeContext {
                project_root,
                locale: None,
            },
        );
        push_include_problems(expanded.problems, &mut seen, diags);
    }

    for path in fragments {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
        if !used.contains(&canonical) {
            diags.push(Diagnostic {
                check: "include-unused-fragment",
                category: "content",
                severity: Severity::Info,
                message: "Fragment isn't included by any page".to_string(),
                file: Some(path.clone()),
                line: None,
                fix: None,
            });
        }
    }

    if config.is_i18n_enabled() {
        check_fragment_locale_coverage(config, fragments, diags);
    }
}

fn push_include_problems(
    problems: Vec<IncludeProblem>,
    seen: &mut HashSet<(PathBuf, usize)>,
    diags: &mut Vec<Diagnostic>,
) {
    for problem in problems {
        if seen.insert((problem.file.clone(), problem.line)) {
            diags.push(Diagnostic {
                check: problem.check,
                category: "content",
                severity: Severity::Error,
                message: problem.message,
                file: Some(problem.file),
                line: Some(problem.line),
                fix: None,
            });
        }
    }
}

/// A fragment translated into some enabled locales but not others.
fn check_fragment_locale_coverage(
    config: &Config,
    fragments: &[PathBuf],
    diags: &mut Vec<Diagnostic>,
) {
    let enabled = &config.locale.enabled;
    let default = config.default_locale().unwrap_or("en");

    // Untranslated path → locales it exists in (an unsuffixed file counts as the default).
    let mut groups: BTreeMap<PathBuf, BTreeSet<String>> = BTreeMap::new();
    for path in fragments {
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let (base, locale) = project::extract_locale_suffix(&stem, enabled);
        groups
            .entry(path.with_file_name(format!("{base}.md")))
            .or_default()
            .insert(locale.unwrap_or_else(|| default.to_string()));
    }

    for (base, locales) in groups {
        let translated = locales.iter().any(|l| l != default);
        let missing: Vec<&str> = enabled
            .iter()
            .filter(|l| !locales.contains(*l))
            .map(String::as_str)
            .collect();
        if translated && !missing.is_empty() {
            diags.push(Diagnostic {
                check: "include-locale-coverage",
                category: "content",
                severity: Severity::Warning,
                message: format!(
                    "Fragment is translated, but has no {} version — pages in {} fall back to the untranslated file",
                    missing.join(", "),
                    if missing.len() == 1 { "that language" } else { "those languages" }
                ),
                file: Some(base),
                line: None,
                fix: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::Severity;
    use std::fs;
    use std::path::PathBuf;

    const CONFIG: &str = "[project]\nname = \"T\"\n";
    const I18N_CONFIG: &str = "[project]\nname = \"T\"\n\n[locale]\ndefault = \"en\"\nenabled = [\"en\", \"fr\", \"de\"]\n";

    fn doctor(config: &str, files: &[(&str, &str)]) -> (tempfile::TempDir, Vec<Diagnostic>) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("docanvil.toml"), config).unwrap();
        for (path, content) in files {
            let p = dir.path().join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, content).unwrap();
        }
        let config = Config::load(dir.path()).unwrap();
        let locales = config
            .is_i18n_enabled()
            .then(|| config.locale.enabled.clone());
        let inventory = PageInventory::scan(
            &dir.path().join("docs"),
            locales.as_deref(),
            config.default_locale(),
            None,
        )
        .unwrap();
        let diags = check_content(dir.path(), &config, &inventory);
        (dir, diags)
    }

    fn found(diags: &[Diagnostic], check: &str) -> Vec<(PathBuf, Option<usize>)> {
        diags
            .iter()
            .filter(|d| d.check == check)
            .map(|d| (d.file.clone().unwrap_or_default(), d.line))
            .collect()
    }

    fn canon(dir: &tempfile::TempDir, rel: &str) -> PathBuf {
        dir.path().join(rel).canonicalize().unwrap()
    }

    #[test]
    fn clean_project_has_no_include_diagnostics() {
        let (_dir, diags) = doctor(
            CONFIG,
            &[
                (
                    "docs/index.md",
                    "# Home\n\n:::include{file=\"_shared/a.md\"}\n",
                ),
                ("docs/_shared/a.md", "Shared.\n"),
            ],
        );
        let include: Vec<_> = diags
            .iter()
            .filter(|d| d.check.starts_with("include-"))
            .collect();
        assert!(include.is_empty(), "{include:?}");
    }

    #[test]
    fn include_lines_are_not_unclosed_directives() {
        let (_dir, diags) = doctor(
            CONFIG,
            &[
                (
                    "docs/index.md",
                    "# Home\n\n  :::include{file=\"_shared/a.md\"}\n",
                ),
                ("docs/_shared/a.md", "Shared.\n"),
            ],
        );
        assert!(found(&diags, "unclosed-directive").is_empty());
    }

    #[test]
    fn missing_include_is_an_error_at_the_page_line() {
        let (dir, diags) = doctor(
            CONFIG,
            &[("docs/index.md", "# Home\n\n:::include{file=\"_nope.md\"}\n")],
        );
        assert_eq!(
            found(&diags, "include-unresolved"),
            vec![(canon(&dir, "docs/index.md"), Some(3))]
        );
        assert!(
            diags
                .iter()
                .any(|d| d.check == "include-unresolved" && d.severity == Severity::Error)
        );
    }

    #[test]
    fn fragment_problem_is_reported_once() {
        let (dir, diags) = doctor(
            CONFIG,
            &[
                ("docs/a.md", "# A\n\n:::include{file=\"_shared/code.md\"}\n"),
                ("docs/b.md", "# B\n\n:::include{file=\"_shared/code.md\"}\n"),
                (
                    "docs/_shared/code.md",
                    "```rust file=\"x.rs\" lines=\"5-9\"\n```\n",
                ),
                ("docs/_shared/x.rs", "fn x() {}\n"),
            ],
        );
        assert_eq!(
            found(&diags, "include-invalid"),
            vec![(canon(&dir, "docs/_shared/code.md"), Some(1))]
        );
    }

    #[test]
    fn include_cycle_is_reported() {
        let (_dir, diags) = doctor(
            CONFIG,
            &[
                ("docs/index.md", "# Home\n\n:::include{file=\"_a.md\"}\n"),
                ("docs/_a.md", ":::include{file=\"_b.md\"}\n"),
                ("docs/_b.md", ":::include{file=\"_a.md\"}\n"),
            ],
        );
        assert!(!found(&diags, "include-cycle").is_empty());
    }

    #[test]
    fn inline_include_is_a_warning() {
        let (dir, diags) = doctor(
            CONFIG,
            &[
                (
                    "docs/index.md",
                    "# Home\n\nSee :::include{file=\"_a.md\"} here.\n",
                ),
                ("docs/_a.md", "A\n"),
            ],
        );
        assert_eq!(
            found(&diags, "include-inline"),
            vec![(dir.path().join("docs/index.md"), Some(3))]
        );
    }

    #[test]
    fn unused_fragments_are_listed() {
        let (dir, diags) = doctor(
            CONFIG,
            &[
                (
                    "docs/index.md",
                    "# Home\n\n:::include{file=\"_shared/used.md\"}\n",
                ),
                ("docs/_shared/used.md", ":::include{file=\"nested.md\"}\n"),
                ("docs/_shared/nested.md", "Nested.\n"),
                ("docs/_shared/orphan.md", "Nobody includes me.\n"),
            ],
        );
        assert_eq!(
            found(&diags, "include-unused-fragment"),
            vec![(dir.path().join("docs/_shared/orphan.md"), None)]
        );
        assert!(
            diags
                .iter()
                .any(|d| d.check == "include-unused-fragment" && d.severity == Severity::Info)
        );
    }

    #[test]
    fn partly_translated_fragment_is_flagged() {
        let (_dir, diags) = doctor(
            I18N_CONFIG,
            &[
                (
                    "docs/index.md",
                    "# Home\n\n:::include{file=\"_shared/a.md\"}\n:::include{file=\"_shared/b.md\"}\n",
                ),
                (
                    "docs/index.fr.md",
                    "# Accueil\n\n:::include{file=\"_shared/a.md\"}\n:::include{file=\"_shared/b.md\"}\n",
                ),
                ("docs/_shared/a.md", "A\n"),
                ("docs/_shared/a.fr.md", "A fr\n"),
                ("docs/_shared/b.md", "B\n"),
            ],
        );
        let coverage: Vec<_> = diags
            .iter()
            .filter(|d| d.check == "include-locale-coverage")
            .collect();
        assert_eq!(coverage.len(), 1, "{coverage:?}");
        assert!(
            coverage[0].message.contains("de"),
            "{}",
            coverage[0].message
        );
        assert!(coverage[0].file.as_ref().unwrap().ends_with("_shared/a.md"));
    }

    #[test]
    fn broken_link_in_fragment_points_at_fragment() {
        let (dir, diags) = doctor(
            CONFIG,
            &[
                ("docs/index.md", "# Home\n\n:::include{file=\"_a.md\"}\n"),
                ("docs/_a.md", "Intro\n\nSee [[nowhere]].\n"),
            ],
        );
        assert_eq!(
            found(&diags, "broken-wiki-link"),
            vec![(dir.path().join("docs/_a.md"), Some(3))]
        );
    }

    fn test_inventory() -> (tempfile::TempDir, PageInventory) {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.md"), "# Home").unwrap();
        fs::write(docs.join("setup.md"), "# Setup").unwrap();
        let inv = PageInventory::scan(&docs, None, None, None).unwrap();
        (dir, inv)
    }

    fn broken_link_diags(source: &str) -> Vec<Diagnostic> {
        let (_dir, inv) = test_inventory();
        let mut diags = Vec::new();
        check_broken_wikilinks(source, Path::new("test.md"), None, &inv, &mut diags);
        diags
    }

    #[test]
    fn real_broken_link_still_warns() {
        let diags = broken_link_diags("See [[nonexistent]] page.");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "broken-wiki-link");
        assert_eq!(diags[0].severity, Severity::Warning);
    }

    #[test]
    fn wikilink_resolves_without_warning() {
        let diags = broken_link_diags("See [[setup]] for details.");
        assert!(diags.is_empty());
    }

    #[test]
    fn toml_double_bracket_in_fenced_block_ignored() {
        let source = "```toml\n[[nav]]\npage = \"x\"\n```\n";
        let diags = broken_link_diags(source);
        assert!(diags.is_empty(), "expected no diagnostics, got {diags:?}");
    }

    #[test]
    fn toml_double_bracket_in_inline_code_ignored() {
        let diags = broken_link_diags("Inline `[[nav]]` too.");
        assert!(diags.is_empty(), "expected no diagnostics, got {diags:?}");
    }

    #[test]
    fn broken_link_outside_code_still_detected_alongside_code() {
        let source = "```toml\n[[nav]]\n```\n\nSee [[nonexistent]] please.\n";
        let diags = broken_link_diags(source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].message, "Broken link [[nonexistent]]");
    }
}
