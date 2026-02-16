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

        check_broken_wikilinks(&source, &page.source_path, inventory, &mut diags);
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

fn check_broken_wikilinks(
    source: &str,
    source_path: &Path,
    inventory: &PageInventory,
    diags: &mut Vec<Diagnostic>,
) {
    let mut remaining = source;
    let mut offset = 0;

    while let Some(start) = remaining.find("[[") {
        let after_open = &remaining[start + 2..];
        if let Some(end) = after_open.find("]]") {
            let inner = &after_open[..end];
            let (target, _display) = if let Some(pipe_pos) = inner.find('|') {
                (&inner[..pipe_pos], &inner[pipe_pos + 1..])
            } else {
                (inner, inner)
            };

            let target = target.trim();
            if !target.is_empty() && inventory.resolve_link(target).is_none() {
                // Calculate line number
                let pos = offset + start;
                let line = source[..pos].matches('\n').count() + 1;
                diags.push(Diagnostic {
                    check: "broken-wiki-link",
                    category: "content",
                    severity: Severity::Warning,
                    message: format!("Broken link [[{target}]]"),
                    file: Some(source_path.to_path_buf()),
                    line: Some(line),
                    fix: None,
                });
            }

            offset += start + 2 + end + 2;
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
