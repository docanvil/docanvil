use std::collections::HashMap;
use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::doctor::{Diagnostic, Severity};
use crate::project::PageInventory;

static OPEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(:{3,})\s*([\w][\w-]*)\s*(\{.*\})?\s*$").unwrap());

/// Check content: broken wiki-links, unclosed directives, front-matter errors, duplicate slugs.
pub fn check_content(
    _project_root: &Path,
    _config: &Config,
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
    }

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

/// Tracks whether we're inside a ``` / ~~~ fenced code block, line by line.
/// Mirrors `pipeline::directives::FenceState` — kept local since it's a tiny,
/// self-contained state machine and this check works line-by-line already.
#[derive(Default)]
struct FenceState {
    open: Option<(char, usize)>,
}

impl FenceState {
    /// Feed one line; returns true if the line is part of a code block
    /// (an opening fence, its contents, or its closing fence).
    fn consume(&mut self, line: &str) -> bool {
        let trimmed = line.trim();
        match self.open {
            None => {
                let first = trimmed.chars().next();
                if let Some(c @ ('`' | '~')) = first {
                    let len = trimmed.chars().take_while(|&ch| ch == c).count();
                    if len >= 3 {
                        self.open = Some((c, len));
                        return true;
                    }
                }
                false
            }
            Some((c, len)) => {
                let count = trimmed.chars().take_while(|&ch| ch == c).count();
                if count >= len && trimmed.chars().skip(count).all(char::is_whitespace) {
                    self.open = None;
                }
                true
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::Severity;
    use std::fs;

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
