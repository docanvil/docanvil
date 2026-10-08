//! Includes: `:::include{file="…"}` and code blocks filled from source files.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

use crate::pipeline::directives::{ATTR_RE, FenceState};
use crate::render::templates::include_cycle_message;
use crate::util::html_escape;

/// Doctor check names for the three kinds of include problem.
pub const CHECK_UNRESOLVED: &str = "include-unresolved";
pub const CHECK_INVALID: &str = "include-invalid";
pub const CHECK_CYCLE: &str = "include-cycle";

const PATH_HINT: &str = "Paths are relative to the file they're written in; start with / for the project root (the folder with docanvil.toml).";

/// `:::include{…}` alone on its line (leading whitespace allowed).
static INCLUDE_LINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*):::include(\{.*\})?\s*$").unwrap());

/// Where includes resolve from.
pub struct IncludeContext<'a> {
    /// The directory with `docanvil.toml`; `/`-rooted paths start here.
    pub project_root: &'a Path,
    /// The page's locale — `Some` only when i18n is enabled.
    pub locale: Option<&'a str>,
}

/// The result of expanding one page.
#[derive(Debug, Default)]
pub struct Expanded {
    /// Markdown with includes spliced in and file code blocks filled.
    pub source: String,
    /// Every file read along the way (canonical paths), for the dev server's watcher.
    pub dependencies: BTreeSet<PathBuf>,
    pub problems: Vec<IncludeProblem>,
}

/// Something that couldn't be included. Shown on the page as an error box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncludeProblem {
    /// `CHECK_UNRESOLVED`, `CHECK_INVALID` or `CHECK_CYCLE`.
    pub check: &'static str,
    /// The file the include or code block was written in (canonical).
    pub file: PathBuf,
    /// 1-based line in `file`.
    pub line: usize,
    pub message: String,
    pub hint: Option<String>,
}

/// Expand `:::include{file="…"}` lines (recursively) in a page's Markdown.
/// Problems never stop the page: each becomes an inline error box and an
/// entry in `problems`.
pub fn expand(source: &str, source_file: &Path, ctx: &IncludeContext) -> Expanded {
    let file = canonical(source_file);
    let mut expander = Expander {
        project_root: ctx.project_root,
        display_root: canonical(ctx.project_root),
        locale: ctx.locale,
        stack: vec![file.clone()],
        expanded: Expanded::default(),
    };
    let lines: Vec<&str> = source.lines().collect();
    let mut out = Vec::new();
    expander.expand_lines(&lines, 0, &file, "", &mut out);

    let mut expanded = expander.expanded;
    expanded.source = out.join("\n");
    expanded.source.push('\n');
    expanded
}

struct Expander<'a> {
    project_root: &'a Path,
    /// Canonical project root, for showing paths in messages.
    display_root: PathBuf,
    locale: Option<&'a str>,
    /// Files currently being expanded, outermost first (the cycle guard).
    stack: Vec<PathBuf>,
    expanded: Expanded,
}

impl Expander<'_> {
    /// Expand `lines[first..]` of `file`, prefixing every output line with `prefix`.
    fn expand_lines(
        &mut self,
        lines: &[&str],
        first: usize,
        file: &Path,
        prefix: &str,
        out: &mut Vec<String>,
    ) {
        let mut i = first;
        while i < lines.len() {
            if starts_fence(lines[i]) {
                let (body_end, close) = fence_extent(lines, i);
                let end = close.map_or(body_end, |c| c + 1);
                for line in &lines[i..end] {
                    push(out, prefix, line);
                }
                i = end;
                continue;
            }
            if let Some(caps) = INCLUDE_LINE_RE.captures(lines[i]) {
                let indent = format!("{prefix}{}", &caps[1]);
                let attrs = caps
                    .get(2)
                    .map(|m| attr_map(m.as_str()))
                    .unwrap_or_default();
                self.include(&attrs, file, i + 1, &indent, out);
            } else {
                push(out, prefix, lines[i]);
            }
            i += 1;
        }
    }

    fn include(
        &mut self,
        attrs: &BTreeMap<String, String>,
        file: &Path,
        line: usize,
        indent: &str,
        out: &mut Vec<String>,
    ) {
        if attrs.contains_key("lines") {
            let p = problem(
                CHECK_INVALID,
                file,
                line,
                "lines=\"…\" only works on code blocks, not on :::include".to_string(),
                Some(
                    "To include part of a page, move that part into its own fragment file and include that.",
                ),
            );
            self.report(p, indent, out);
            return;
        }
        if let Some(key) = attrs.keys().find(|k| k.as_str() != "file") {
            let p = problem(
                CHECK_INVALID,
                file,
                line,
                format!(":::include only takes file=\"…\", not {key}=\"…\""),
                None,
            );
            self.report(p, indent, out);
            return;
        }
        let Some(written) = attrs.get("file") else {
            let p = problem(
                CHECK_INVALID,
                file,
                line,
                ":::include needs a file, e.g. :::include{file=\"_shared/install.md\"}".to_string(),
                None,
            );
            self.report(p, indent, out);
            return;
        };

        let base = resolve_path(written, file, self.project_root);
        let mut candidates = Vec::new();
        if let Some(locale) = self.locale {
            candidates.push(localized(&base, locale));
        }
        candidates.push(base.clone());
        let Some(found) = candidates.into_iter().find(|p| p.is_file()) else {
            let p = problem(
                CHECK_UNRESOLVED,
                file,
                line,
                format!("can't find {written} (looked for {})", base.display()),
                Some(PATH_HINT),
            );
            self.report(p, indent, out);
            return;
        };
        let found = canonical(&found);

        if let Some(pos) = self.stack.iter().position(|p| *p == found) {
            let mut chain: Vec<String> =
                self.stack[pos..].iter().map(|p| self.display(p)).collect();
            chain.push(self.display(&found));
            let p = problem(
                CHECK_CYCLE,
                file,
                line,
                include_cycle_message(&chain, "files"),
                None,
            );
            self.report(p, indent, out);
            return;
        }

        let text = match read_utf8(&found, written) {
            Ok(text) => text,
            Err(message) => {
                self.report(
                    problem(CHECK_UNRESOLVED, file, line, message, None),
                    indent,
                    out,
                );
                return;
            }
        };
        self.expanded.dependencies.insert(found.clone());

        let lines: Vec<&str> = text.lines().collect();
        let first = front_matter_end(&lines);
        self.stack.push(found.clone());
        self.expand_lines(&lines, first, &found, indent, out);
        self.stack.pop();
    }

    /// Emit the error box in place of the content and record the problem.
    fn report(&mut self, problem: IncludeProblem, indent: &str, out: &mut Vec<String>) {
        out.push(format!(
            "{indent}<div class=\"include-error\">{}</div>",
            html_escape(&problem.message)
        ));
        // A blank line ends the HTML block so the next line renders as Markdown.
        out.push(String::new());
        self.expanded.problems.push(problem);
    }

    /// `path` relative to the project root, with `/` separators.
    fn display(&self, path: &Path) -> String {
        path.strip_prefix(&self.display_root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

fn problem(
    check: &'static str,
    file: &Path,
    line: usize,
    message: String,
    hint: Option<&str>,
) -> IncludeProblem {
    IncludeProblem {
        check,
        file: file.to_path_buf(),
        line,
        message,
        hint: hint.map(String::from),
    }
}

fn push(out: &mut Vec<String>, prefix: &str, line: &str) {
    if line.is_empty() {
        out.push(String::new());
    } else {
        out.push(format!("{prefix}{line}"));
    }
}

fn starts_fence(line: &str) -> bool {
    FenceState::default().consume(line)
}

/// For a fence opening at `start`: the index just past its body, and the
/// closing line's index (`None` when the fence runs to the end of the file).
fn fence_extent(lines: &[&str], start: usize) -> (usize, Option<usize>) {
    let mut fence = FenceState::default();
    fence.consume(lines[start]);
    for (j, line) in lines.iter().enumerate().skip(start + 1) {
        fence.consume(line);
        if !fence.is_open() {
            return (j, Some(j));
        }
    }
    (lines.len(), None)
}

fn attr_map(text: &str) -> BTreeMap<String, String> {
    ATTR_RE
        .captures_iter(text)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect()
}

/// `/x` → project root; anything else → relative to the directory of
/// `current_file`. `\` is treated as `/`. Leading slashes are all stripped, so
/// a path can never reach the filesystem root.
fn resolve_path(written: &str, current_file: &Path, project_root: &Path) -> PathBuf {
    let written = written.replace('\\', "/");
    if written.starts_with('/') {
        project_root.join(written.trim_start_matches('/'))
    } else {
        current_file.parent().unwrap_or(Path::new("")).join(written)
    }
}

/// `dir/name.md` → `dir/name.{locale}.md`.
fn localized(path: &Path, locale: &str) -> PathBuf {
    match (path.file_stem(), path.extension()) {
        (Some(stem), Some(ext)) => path.with_file_name(format!(
            "{}.{locale}.{}",
            stem.to_string_lossy(),
            ext.to_string_lossy()
        )),
        _ => path.to_path_buf(),
    }
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn read_utf8(path: &Path, written: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("can't read {written}: {e}"))?;
    String::from_utf8(bytes)
        .map_err(|_| format!("{written} isn't valid UTF-8 — save it as UTF-8 to include it"))
}

/// Index of the first line after a `---` … `---` front matter block (0 if none).
fn front_matter_end(lines: &[&str]) -> usize {
    if lines.first().map(|l| l.trim()) != Some("---") {
        return 0;
    }
    lines
        .iter()
        .skip(1)
        .position(|l| l.trim() == "---")
        .map_or(0, |i| i + 2)
}

/// Parse a `lines="…"` spec for a file with `line_count` lines.
///
/// Grammar: `N`, `N-M`, `N-` (to the end), `-M` (from the start), joined by
/// commas; 1-based and inclusive. Ranges must go in order without overlapping
/// (adjacent is fine). A range past the end of the file is an error, so stale
/// line numbers are caught at build time rather than shipped.
pub fn parse_line_ranges(spec: &str, line_count: usize) -> Result<Vec<(usize, usize)>, String> {
    if spec.trim().is_empty() {
        return Err(
            "lines=\"\" is empty — give a line or range such as lines=\"5-12\", or leave lines out to show the whole file"
                .to_string(),
        );
    }

    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        let (start, end) = match part.split_once('-') {
            Some((a, b)) => {
                let (a, b) = (a.trim(), b.trim());
                if a.is_empty() && b.is_empty() {
                    return Err(not_a_range(part));
                }
                let start = if a.is_empty() {
                    1
                } else {
                    line_number(a, part)?
                };
                let end = if b.is_empty() {
                    None
                } else {
                    Some(line_number(b, part)?)
                };
                (start, end)
            }
            None => {
                let n = line_number(part, part)?;
                (n, Some(n))
            }
        };

        if let Some(end) = end
            && start > end
        {
            return Err(format!("lines range \"{part}\" ends before it starts"));
        }
        if start > line_count {
            return Err(format!(
                "lines range \"{part}\" starts past the end of the file, which has {}",
                count_lines(line_count)
            ));
        }
        let end = match end {
            Some(end) if end > line_count => {
                return Err(format!(
                    "lines range \"{part}\" runs past the end of the file, which has {}",
                    count_lines(line_count)
                ));
            }
            Some(end) => end,
            None => line_count,
        };

        if let Some(&(prev_start, prev_end)) = ranges.last() {
            if start < prev_start {
                return Err(format!(
                    "lines ranges must go in order — \"{part}\" comes before the range ahead of it"
                ));
            }
            if start <= prev_end {
                return Err(format!(
                    "lines range \"{part}\" overlaps the range before it"
                ));
            }
        }
        ranges.push((start, end));
    }
    Ok(ranges)
}

fn line_number(text: &str, part: &str) -> Result<usize, String> {
    match text.parse::<usize>() {
        Ok(0) => Err(format!(
            "lines range \"{part}\" uses line 0 — lines are numbered from 1"
        )),
        Ok(n) => Ok(n),
        Err(_) => Err(not_a_range(part)),
    }
}

fn not_a_range(part: &str) -> String {
    format!(
        "\"{part}\" isn't a line or range — use e.g. lines=\"7\", \"5-12\", \"5-\", \"-12\" or \"1-6,30-35\""
    )
}

fn count_lines(n: usize) -> String {
    if n == 1 {
        "1 line".to_string()
    } else {
        format!("{n} lines")
    }
}

/// The lines covered by `ranges` (as returned by [`parse_line_ranges`]), in order.
pub fn select_lines<'a>(lines: &[&'a str], ranges: &[(usize, usize)]) -> Vec<&'a str> {
    ranges
        .iter()
        .flat_map(|&(start, end)| lines[start - 1..end].iter().copied())
        .collect()
}

/// Remove the leading whitespace shared by every non-blank line, keeping
/// relative indentation. Tabs and spaces are compared literally (a tab never
/// matches spaces). Whitespace-only lines become empty.
pub fn dedent(lines: &[&str]) -> Vec<String> {
    let common = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| &line[..line.len() - line.trim_start().len()])
        .reduce(common_prefix)
        .unwrap_or("");
    lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                line[common.len()..].to_string()
            }
        })
        .collect()
}

fn common_prefix<'a>(a: &'a str, b: &str) -> &'a str {
    let len: usize = a
        .chars()
        .zip(b.chars())
        .take_while(|(x, y)| x == y)
        .map(|(x, _)| x.len_utf8())
        .sum();
    &a[..len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_line_and_closed_range() {
        assert_eq!(parse_line_ranges("7", 10), Ok(vec![(7, 7)]));
        assert_eq!(parse_line_ranges("5-12", 20), Ok(vec![(5, 12)]));
    }

    #[test]
    fn open_ended_ranges() {
        assert_eq!(parse_line_ranges("5-", 10), Ok(vec![(5, 10)]));
        assert_eq!(parse_line_ranges("-3", 10), Ok(vec![(1, 3)]));
        assert_eq!(parse_line_ranges("40-", 40), Ok(vec![(40, 40)]));
    }

    #[test]
    fn lists_whitespace_and_adjacent_ranges() {
        assert_eq!(
            parse_line_ranges(" 1 - 6 , 30-35 ", 40),
            Ok(vec![(1, 6), (30, 35)])
        );
        assert_eq!(parse_line_ranges("1-6,7-9", 40), Ok(vec![(1, 6), (7, 9)]));
    }

    #[test]
    fn rejects_bad_specs() {
        for spec in [
            "", " ", "0", "0-3", "7-5", "abc", "1-x", "-", "1--3", "1-6,",
        ] {
            assert!(
                parse_line_ranges(spec, 40).is_err(),
                "{spec:?} should be rejected"
            );
        }
    }

    #[test]
    fn rejects_descending_and_overlapping() {
        assert!(
            parse_line_ranges("30-35,1-6", 40)
                .unwrap_err()
                .contains("in order")
        );
        assert!(
            parse_line_ranges("1-6,5-9", 40)
                .unwrap_err()
                .contains("overlaps")
        );
        assert!(
            parse_line_ranges("1-6,6-9", 40)
                .unwrap_err()
                .contains("overlaps")
        );
    }

    #[test]
    fn rejects_ranges_past_the_end() {
        let err = parse_line_ranges("41", 40).unwrap_err();
        assert!(
            err.contains("past the end") && err.contains("40 lines"),
            "{err}"
        );
        assert!(
            parse_line_ranges("38-45", 40)
                .unwrap_err()
                .contains("past the end")
        );
        assert!(parse_line_ranges("41-", 40).is_err());
        assert!(parse_line_ranges("-41", 40).is_err());
        assert!(parse_line_ranges("1", 0).is_err());
    }

    #[test]
    fn selects_ranges_in_order() {
        let lines = ["a", "b", "c", "d", "e"];
        assert_eq!(select_lines(&lines, &[(1, 2), (4, 4)]), vec!["a", "b", "d"]);
    }

    #[test]
    fn dedent_removes_common_indent() {
        assert_eq!(
            dedent(&["    fn a() {", "        x", "    }"]),
            vec!["fn a() {", "    x", "}"]
        );
    }

    #[test]
    fn dedent_ignores_blank_lines() {
        assert_eq!(
            dedent(&["    a", "", "  ", "    b"]),
            vec!["a", "", "", "b"]
        );
    }

    #[test]
    fn dedent_keeps_relative_indent_across_ranges() {
        assert_eq!(
            dedent(&["        inner", "    outer"]),
            vec!["    inner", "outer"]
        );
    }

    #[test]
    fn dedent_compares_tabs_and_spaces_literally() {
        assert_eq!(dedent(&["\tx", "    y"]), vec!["\tx", "    y"]);
        assert_eq!(dedent(&["\t\tx", "\ty"]), vec!["\tx", "y"]);
        assert_eq!(dedent(&["a", " b"]), vec!["a", " b"]);
    }

    fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (path, content) in files {
            let p = dir.path().join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, content).unwrap();
        }
        dir
    }

    fn run(dir: &tempfile::TempDir, page: &str, locale: Option<&str>) -> Expanded {
        let file = dir.path().join(page);
        let source = std::fs::read_to_string(&file).unwrap();
        expand(
            &source,
            &file,
            &IncludeContext {
                project_root: dir.path(),
                locale,
            },
        )
    }

    fn canon(dir: &tempfile::TempDir, rel: &str) -> PathBuf {
        dir.path().join(rel).canonicalize().unwrap()
    }

    #[test]
    fn splices_fragment() {
        let dir = project(&[
            (
                "docs/page.md",
                "# Page\n\n:::include{file=\"_shared/install.md\"}\n\nAfter\n",
            ),
            ("docs/_shared/install.md", "## Install\n\nRun it.\n"),
        ]);
        let out = run(&dir, "docs/page.md", None);
        assert_eq!(out.source, "# Page\n\n## Install\n\nRun it.\n\nAfter\n");
        assert!(out.problems.is_empty());
        assert_eq!(
            out.dependencies,
            BTreeSet::from([canon(&dir, "docs/_shared/install.md")])
        );
    }

    #[test]
    fn nested_paths_resolve_relative_to_each_fragment() {
        let dir = project(&[
            ("docs/page.md", ":::include{file=\"_shared/a.md\"}\n"),
            ("docs/_shared/a.md", "A\n:::include{file=\"parts/b.md\"}\n"),
            ("docs/_shared/parts/b.md", "B\n"),
        ]);
        let out = run(&dir, "docs/page.md", None);
        assert_eq!(out.source, "A\nB\n");
        assert_eq!(out.dependencies.len(), 2);
    }

    #[test]
    fn leading_slash_is_project_root_and_backslashes_work() {
        let dir = project(&[
            (
                "docs/guide/page.md",
                ":::include{file=\"/_shared/x.md\"}\n:::include{file=\"..\\..\\_shared\\x.md\"}\n",
            ),
            ("_shared/x.md", "X\n"),
        ]);
        let out = run(&dir, "docs/guide/page.md", None);
        assert_eq!(out.source, "X\nX\n");
        assert!(out.problems.is_empty(), "{:?}", out.problems);
    }

    #[test]
    fn double_slash_never_reaches_the_filesystem_root() {
        let dir = project(&[("docs/page.md", ":::include{file=\"//etc/hosts\"}\n")]);
        let out = run(&dir, "docs/page.md", None);
        assert_eq!(out.problems.len(), 1);
        assert_eq!(out.problems[0].check, CHECK_UNRESOLVED);
        assert!(out.dependencies.is_empty());
    }

    #[test]
    fn parent_directories_are_allowed() {
        let dir = project(&[
            ("docs/page.md", ":::include{file=\"../README.md\"}\n"),
            ("README.md", "Readme\n"),
        ]);
        assert_eq!(run(&dir, "docs/page.md", None).source, "Readme\n");
    }

    #[test]
    fn locale_variant_is_preferred_then_falls_back() {
        let dir = project(&[
            ("docs/page.fr.md", ":::include{file=\"_shared/i.md\"}\n"),
            ("docs/_shared/i.md", "EN\n"),
            ("docs/_shared/i.fr.md", "FR\n"),
        ]);
        let fr = run(&dir, "docs/page.fr.md", Some("fr"));
        assert_eq!(fr.source, "FR\n");
        assert_eq!(
            fr.dependencies,
            BTreeSet::from([canon(&dir, "docs/_shared/i.fr.md")])
        );
        assert_eq!(run(&dir, "docs/page.fr.md", Some("de")).source, "EN\n");
        assert_eq!(run(&dir, "docs/page.fr.md", None).source, "EN\n");
    }

    #[test]
    fn indentation_matches_include_line() {
        let dir = project(&[
            (
                "docs/page.md",
                "- Step one\n\n  :::include{file=\"_s.md\"}\n",
            ),
            ("docs/_s.md", "Line A\n\n```sh\nrun\n```\n"),
        ]);
        assert_eq!(
            run(&dir, "docs/page.md", None).source,
            "- Step one\n\n  Line A\n\n  ```sh\n  run\n  ```\n"
        );
    }

    #[test]
    fn include_inside_code_fence_is_untouched() {
        let source = "```markdown\n:::include{file=\"_x.md\"}\n```\n";
        let dir = project(&[("docs/page.md", source)]);
        let out = run(&dir, "docs/page.md", None);
        assert_eq!(out.source, source);
        assert!(out.problems.is_empty());
    }

    #[test]
    fn mid_line_include_is_left_alone() {
        let source = "See :::include{file=\"_x.md\"} here\n";
        let dir = project(&[("docs/page.md", source), ("docs/_x.md", "X")]);
        let out = run(&dir, "docs/page.md", None);
        assert_eq!(out.source, source);
        assert!(out.problems.is_empty());
    }

    #[test]
    fn fragment_front_matter_is_stripped() {
        let dir = project(&[
            ("docs/page.md", ":::include{file=\"_f.md\"}\n"),
            ("docs/_f.md", "---\n{\"title\": \"X\"}\n---\nBody\n"),
        ]);
        assert_eq!(run(&dir, "docs/page.md", None).source, "Body\n");
    }

    #[test]
    fn same_fragment_twice_is_not_a_cycle() {
        let dir = project(&[
            (
                "docs/page.md",
                ":::include{file=\"_x.md\"}\n:::include{file=\"_x.md\"}\n",
            ),
            ("docs/_x.md", "X\n"),
        ]);
        let out = run(&dir, "docs/page.md", None);
        assert_eq!(out.source, "X\nX\n");
        assert!(out.problems.is_empty());
    }

    #[test]
    fn missing_file_reports_page_line_and_shows_error_box() {
        let dir = project(&[("docs/page.md", "# P\n\n:::include{file=\"_missing.md\"}\n")]);
        let out = run(&dir, "docs/page.md", None);
        assert_eq!(out.problems.len(), 1);
        let p = &out.problems[0];
        assert_eq!(p.check, CHECK_UNRESOLVED);
        assert_eq!(p.file, canon(&dir, "docs/page.md"));
        assert_eq!(p.line, 3);
        assert!(
            p.message.starts_with("can't find _missing.md"),
            "{}",
            p.message
        );
        assert!(p.hint.is_some());
        assert!(
            out.source
                .contains("<div class=\"include-error\">can't find _missing.md")
        );
    }

    #[test]
    fn problem_in_nested_fragment_points_at_the_fragment() {
        let dir = project(&[
            ("docs/page.md", ":::include{file=\"_a.md\"}\n"),
            ("docs/_a.md", "One\n:::include{file=\"_nope.md\"}\n"),
        ]);
        let p = &run(&dir, "docs/page.md", None).problems[0];
        assert_eq!(p.file, canon(&dir, "docs/_a.md"));
        assert_eq!(p.line, 2);
    }

    #[test]
    fn invalid_include_attributes() {
        let dir = project(&[
            (
                "docs/page.md",
                ":::include{file=\"_x.md\" lines=\"1-3\"}\n:::include{file=\"_x.md\" title=\"X\"}\n:::include\n",
            ),
            ("docs/_x.md", "X\n"),
        ]);
        let problems = run(&dir, "docs/page.md", None).problems;
        assert_eq!(problems.len(), 3);
        assert!(problems.iter().all(|p| p.check == CHECK_INVALID));
        assert!(
            problems[0]
                .hint
                .as_deref()
                .unwrap()
                .contains("its own fragment file")
        );
        assert!(problems[1].message.contains("only takes file"));
        assert!(problems[2].message.contains("needs a file"));
    }

    #[test]
    fn non_utf8_fragment_is_reported() {
        let dir = project(&[("docs/page.md", ":::include{file=\"_bin.md\"}\n")]);
        std::fs::write(dir.path().join("docs/_bin.md"), [0xff, 0xfe, 0x00]).unwrap();
        let p = &run(&dir, "docs/page.md", None).problems[0];
        assert_eq!(p.check, CHECK_UNRESOLVED);
        assert!(p.message.contains("isn't valid UTF-8"), "{}", p.message);
    }

    #[test]
    fn self_include_is_a_cycle() {
        let dir = project(&[
            ("docs/page.md", ":::include{file=\"_a.md\"}\n"),
            ("docs/_a.md", ":::include{file=\"_a.md\"}\n"),
        ]);
        let p = &run(&dir, "docs/page.md", None).problems[0];
        assert_eq!(p.check, CHECK_CYCLE);
        assert_eq!(
            p.message,
            "docs/_a.md includes itself, so it would never finish rendering"
        );
    }

    #[test]
    fn indirect_cycle_names_the_chain() {
        let dir = project(&[
            ("docs/page.md", ":::include{file=\"_a.md\"}\n"),
            ("docs/_a.md", ":::include{file=\"_b.md\"}\n"),
            ("docs/_b.md", ":::include{file=\"_a.md\"}\n"),
        ]);
        let p = &run(&dir, "docs/page.md", None).problems[0];
        assert_eq!(p.file, canon(&dir, "docs/_b.md"));
        assert_eq!(p.line, 1);
        assert_eq!(
            p.message,
            "files include each other in a loop (docs/_a.md → docs/_b.md → docs/_a.md), so they would never finish rendering"
        );
    }

    #[test]
    fn page_including_itself_is_a_cycle() {
        let dir = project(&[("docs/page.md", ":::include{file=\"page.md\"}\n")]);
        let p = &run(&dir, "docs/page.md", None).problems[0];
        assert_eq!(
            p.message,
            "docs/page.md includes itself, so it would never finish rendering"
        );
    }
}
