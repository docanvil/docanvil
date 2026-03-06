use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
use crate::doctor::{Diagnostic, Severity};
use crate::project::PageInventory;

// ---------------------------------------------------------------------------
// Static regexes
// ---------------------------------------------------------------------------

static HEADING_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(#{1,6})\s*(.*?)\s*$").unwrap());

static IMAGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[([^\]]*)\]\([^\)]+\)").unwrap());

// Matches any inline code span on a single line.
static INLINE_CODE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`[^`\n]*`").unwrap());

// Matches Markdown links and images: [text](url) and ![alt](url).
static MD_LINK_OR_IMAGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!?\[[^\]]*\]\([^\)]*\)").unwrap());

// Matches angle-bracket URLs: <https://...>.
static ANGLE_URL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<https?://[^>\s]+>").unwrap());

// Matches any bare http(s) URL.
static BARE_URL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://\S+").unwrap());

// Matches [text](dest), capturing both groups. Used by empty-link and non-descriptive-link-text.
static LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\(([^\)]*)\)").unwrap());

// Matches the reversed link syntax typo: (text)[url].
static REVERSED_LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\([^\)]{1,200}\)\[[^\]]{1,200}\]").unwrap());

// Matches a whole-line bold or double-underscore block.
static EMPHASIS_HEADING_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\*\*[^*]+\*\*$|^__[^_]+__$").unwrap());

// Matches common editorial annotations in prose.
static TODO_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(TODO|FIXME|HACK|XXX|PLACEHOLDER)\b").unwrap());

// Matches common placeholder text patterns.
static PLACEHOLDER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(lorem ipsum|\bTBD\b|\[insert .{0,40} here\])").unwrap());

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the 0-based line index at which the document body starts,
/// skipping any front matter block (`---` … `---`).
fn body_start(lines: &[&str]) -> usize {
    if lines.first().map(|l| l.trim()) != Some("---") {
        return 0;
    }
    let mut i = 1;
    while i < lines.len() {
        if lines[i].trim() == "---" {
            return i + 1;
        }
        i += 1;
    }
    i
}

/// Returns active lines from a markdown source, skipping front matter and
/// fenced code blocks. Each entry is `(1-indexed line number, line content)`.
fn active_lines(source: &str) -> Vec<(usize, &str)> {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = body_start(&lines);
    let mut result = Vec::new();
    let mut in_fence = false;
    let mut fence_char = '`';
    let mut fence_len = 0usize;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        if in_fence {
            let fc = fence_char;
            let char_count = trimmed.chars().take_while(|&c| c == fc).count();
            if char_count >= fence_len
                && trimmed.chars().skip(char_count).all(|c| c.is_whitespace())
            {
                in_fence = false;
            }
            // Opening, content, and closing fence lines are all excluded.
            i += 1;
            continue;
        }

        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let fc = trimmed.chars().next().unwrap();
            let count = trimmed.chars().take_while(|&c| c == fc).count();
            if count >= 3 {
                in_fence = true;
                fence_char = fc;
                fence_len = count;
                i += 1;
                continue;
            }
        }

        result.push((i + 1, line));
        i += 1;
    }

    result
}

/// Returns the opening fence lines from a source file as
/// `(1-indexed line number, language tag)`. The language tag is empty when
/// the fence has no language specifier.
fn scan_fence_openers<'a>(source: &'a str) -> Vec<(usize, &'a str)> {
    let lines: Vec<&'a str> = source.lines().collect();
    let mut i = body_start(&lines);
    let mut result = Vec::new();
    let mut in_fence = false;
    let mut fence_char = '`';
    let mut fence_len = 0usize;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        if in_fence {
            let fc = fence_char;
            let char_count = trimmed.chars().take_while(|&c| c == fc).count();
            if char_count >= fence_len
                && trimmed.chars().skip(char_count).all(|c| c.is_whitespace())
            {
                in_fence = false;
            }
        } else if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let fc = trimmed.chars().next().unwrap();
            let count = trimmed.chars().take_while(|&c| c == fc).count();
            if count >= 3 {
                let lang = trimmed.trim_start_matches(fc).trim();
                result.push((i + 1, lang));
                in_fence = true;
                fence_char = fc;
                fence_len = count;
            }
        }

        i += 1;
    }

    result
}

/// Returns `true` if the document's JSON front matter contains a non-empty
/// `"title"` field.
fn front_matter_has_title(source: &str) -> bool {
    let trimmed = source.trim_start();
    if !trimmed.starts_with("---") {
        return false;
    }
    let after_open = match trimmed[3..].strip_prefix('\n').or_else(|| trimmed[3..].strip_prefix("\r\n")) {
        Some(s) => s,
        None => return false,
    };
    let end = match after_open.find("\n---") {
        Some(i) => i,
        None => return false,
    };
    let json_str = &after_open[..end];
    serde_json::from_str::<serde_json::Value>(json_str)
        .ok()
        .and_then(|v| {
            v.get("title")
                .and_then(|t| t.as_str())
                .map(|s| !s.trim().is_empty())
        })
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run all readability checks against every page in the inventory.
pub fn check_readability(
    _project_root: &Path,
    config: &Config,
    inventory: &PageInventory,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for slug in &inventory.ordered {
        let page = &inventory.pages[slug];
        let source = match std::fs::read_to_string(&page.source_path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let lines = active_lines(&source);

        // Heading structure
        check_multiple_h1(&lines, &page.source_path, &mut diags);
        check_skipped_heading_level(&lines, &page.source_path, &mut diags);
        check_consecutive_headings(&lines, &page.source_path, &mut diags);
        check_empty_heading(&lines, &page.source_path, &mut diags);
        check_heading_punctuation(&lines, &page.source_path, &mut diags);
        check_duplicate_heading_text(&lines, &page.source_path, &mut diags);
        check_emphasis_used_as_heading(&lines, &page.source_path, &mut diags);
        check_no_document_title(&source, &lines, &page.source_path, &mut diags);

        // Links and images
        check_missing_alt_text(&lines, &page.source_path, &mut diags);
        check_reversed_link_syntax(&lines, &page.source_path, &mut diags);
        check_empty_link(&lines, &page.source_path, &mut diags);
        check_non_descriptive_link_text(&lines, &page.source_path, &mut diags);
        check_bare_url(&lines, &page.source_path, &mut diags);

        // Code blocks
        check_missing_fenced_code_language(&source, &page.source_path, &mut diags);

        // Prose quality
        check_long_paragraph(&lines, &page.source_path, config, &mut diags);
        check_repeated_word(&lines, &page.source_path, &mut diags);
        check_todo_comment(&lines, &page.source_path, &mut diags);
        check_placeholder_text(&lines, &page.source_path, &mut diags);
    }
    diags
}

// ---------------------------------------------------------------------------
// Heading checks
// ---------------------------------------------------------------------------

/// Flag the line of the 2nd (and any further) H1 heading in a page.
fn check_multiple_h1(lines: &[(usize, &str)], source_path: &Path, diags: &mut Vec<Diagnostic>) {
    let mut h1_count = 0;
    for &(line_num, line) in lines {
        if let Some(caps) = HEADING_RE.captures(line)
            && caps[1].len() == 1
        {
            h1_count += 1;
            if h1_count >= 2 {
                diags.push(Diagnostic {
                    check: "multiple-h1",
                    category: "readability",
                    severity: Severity::Warning,
                    message: "Multiple H1 headings in the same page".to_string(),
                    file: Some(source_path.to_path_buf()),
                    line: Some(line_num),
                    fix: None,
                });
            }
        }
    }
}

/// Flag a heading whose level jumps by more than one from the previous heading.
/// Decreases in level (e.g. H3 → H2) are fine.
fn check_skipped_heading_level(
    lines: &[(usize, &str)],
    source_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    let mut prev_level: Option<usize> = None;
    for &(line_num, line) in lines {
        if let Some(caps) = HEADING_RE.captures(line) {
            let level = caps[1].len();
            if let Some(prev) = prev_level
                && level > prev + 1
            {
                diags.push(Diagnostic {
                    check: "skipped-heading-level",
                    category: "readability",
                    severity: Severity::Warning,
                    message: format!("Heading level skipped: H{level} follows H{prev}"),
                    file: Some(source_path.to_path_buf()),
                    line: Some(line_num),
                    fix: None,
                });
            }
            prev_level = Some(level);
        }
    }
}

/// Flag a heading that immediately follows another heading (possibly separated
/// by blank lines) without any real content in between.
fn check_consecutive_headings(
    lines: &[(usize, &str)],
    source_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    let mut seen_heading = false;
    let mut had_content = false;
    for &(line_num, line) in lines {
        if HEADING_RE.is_match(line) {
            if seen_heading && !had_content {
                diags.push(Diagnostic {
                    check: "consecutive-headings",
                    category: "readability",
                    severity: Severity::Warning,
                    message: "Consecutive headings with no content between them".to_string(),
                    file: Some(source_path.to_path_buf()),
                    line: Some(line_num),
                    fix: None,
                });
            }
            seen_heading = true;
            had_content = false;
        } else if !line.trim().is_empty() {
            had_content = true;
        }
    }
}

/// Flag a heading with no text (e.g. `## `).
fn check_empty_heading(lines: &[(usize, &str)], source_path: &Path, diags: &mut Vec<Diagnostic>) {
    for &(line_num, line) in lines {
        if let Some(caps) = HEADING_RE.captures(line)
            && caps[2].trim().is_empty()
        {
            diags.push(Diagnostic {
                check: "empty-heading",
                category: "readability",
                severity: Severity::Error,
                message: "Empty heading".to_string(),
                file: Some(source_path.to_path_buf()),
                line: Some(line_num),
                fix: None,
            });
        }
    }
}

/// Flag headings that end with `.` or `!` (question marks are allowed for FAQ pages).
fn check_heading_punctuation(
    lines: &[(usize, &str)],
    source_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    for &(line_num, line) in lines {
        if let Some(caps) = HEADING_RE.captures(line) {
            let text = caps[2].trim();
            if text.ends_with('.') || text.ends_with('!') {
                let punct = text.chars().next_back().unwrap();
                diags.push(Diagnostic {
                    check: "heading-punctuation",
                    category: "readability",
                    severity: Severity::Info,
                    message: format!("Heading ends with punctuation: '{punct}'"),
                    file: Some(source_path.to_path_buf()),
                    line: Some(line_num),
                    fix: None,
                });
            }
        }
    }
}

/// Flag two or more headings in the same file with identical text (case-insensitive).
/// Duplicate headings produce identical anchor IDs, which breaks fragment links.
fn check_duplicate_heading_text(
    lines: &[(usize, &str)],
    source_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    let mut seen: HashMap<String, usize> = HashMap::new();
    for &(line_num, line) in lines {
        if let Some(caps) = HEADING_RE.captures(line) {
            let text = caps[2].trim().to_lowercase();
            if text.is_empty() {
                continue; // empty-heading check covers this
            }
            if let Some(&first_line) = seen.get(&text) {
                diags.push(Diagnostic {
                    check: "duplicate-heading-text",
                    category: "readability",
                    severity: Severity::Warning,
                    message: format!(
                        "Duplicate heading \"{text}\" (first seen at line {first_line})"
                    ),
                    file: Some(source_path.to_path_buf()),
                    line: Some(line_num),
                    fix: None,
                });
            } else {
                seen.insert(text, line_num);
            }
        }
    }
}

/// Flag a line that consists entirely of bold or italic text — a common pattern
/// writers use instead of proper heading syntax (`**Section**` vs `## Section`).
/// Lines ending in sentence punctuation are excluded (those are intentional emphasis).
fn check_emphasis_used_as_heading(
    lines: &[(usize, &str)],
    source_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    for &(line_num, line) in lines {
        let trimmed = line.trim();
        if EMPHASIS_HEADING_RE.is_match(trimmed) {
            // Allow bold text that ends with sentence punctuation — it's likely
            // intentional inline emphasis, not a misplaced heading.
            let last = trimmed.chars().next_back().unwrap_or(' ');
            if matches!(last, '.' | ',' | ':' | ';' | '!' | '?') {
                continue;
            }
            diags.push(Diagnostic {
                check: "emphasis-used-as-heading",
                category: "readability",
                severity: Severity::Warning,
                message: "Entire line is bold — use a Markdown heading (`## Title`) instead"
                    .to_string(),
                file: Some(source_path.to_path_buf()),
                line: Some(line_num),
                fix: None,
            });
        }
    }
}

/// Flag a page that has no H1 heading and no `"title"` in its front matter.
/// Without a title the page renders with no visible heading and the nav falls
/// back to the slug.
fn check_no_document_title(
    source: &str,
    lines: &[(usize, &str)],
    source_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    let has_h1 = lines
        .iter()
        .any(|&(_, line)| HEADING_RE.captures(line).is_some_and(|c| c[1].len() == 1));

    if has_h1 || front_matter_has_title(source) {
        return;
    }

    diags.push(Diagnostic {
        check: "no-document-title",
        category: "readability",
        severity: Severity::Warning,
        message: "Page has no H1 heading and no \"title\" in front matter".to_string(),
        file: Some(source_path.to_path_buf()),
        line: None,
        fix: None,
    });
}

// ---------------------------------------------------------------------------
// Link and image checks
// ---------------------------------------------------------------------------

/// Flag images with no alt text (e.g. `![](photo.jpg)`).
/// Each missing-alt image on a line gets its own diagnostic.
fn check_missing_alt_text(
    lines: &[(usize, &str)],
    source_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    for &(line_num, line) in lines {
        let line = &*INLINE_CODE_RE.replace_all(line, "");
        for caps in IMAGE_RE.captures_iter(line) {
            if caps[1].trim().is_empty() {
                diags.push(Diagnostic {
                    check: "missing-alt-text",
                    category: "readability",
                    severity: Severity::Warning,
                    message: "Image is missing alt text".to_string(),
                    file: Some(source_path.to_path_buf()),
                    line: Some(line_num),
                    fix: None,
                });
            }
        }
    }
}

/// Flag the common typo `(text)[url]` — parentheses and brackets are reversed,
/// so the link does not render.
fn check_reversed_link_syntax(
    lines: &[(usize, &str)],
    source_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    for &(line_num, line) in lines {
        let stripped = INLINE_CODE_RE.replace_all(line, "");
        if REVERSED_LINK_RE.is_match(stripped.as_ref()) {
            diags.push(Diagnostic {
                check: "reversed-link-syntax",
                category: "readability",
                severity: Severity::Error,
                message: "Reversed link syntax — use [text](url), not (text)[url]".to_string(),
                file: Some(source_path.to_path_buf()),
                line: Some(line_num),
                fix: None,
            });
        }
    }
}

/// Flag links with an empty destination (`[text]()`) or no visible text (`[](url)`).
/// Image links (`![alt](url)`) are excluded — those are covered by `missing-alt-text`.
fn check_empty_link(lines: &[(usize, &str)], source_path: &Path, diags: &mut Vec<Diagnostic>) {
    for &(line_num, line) in lines {
        let line_stripped = INLINE_CODE_RE.replace_all(line, "");
        let line = line_stripped.as_ref();
        for caps in LINK_RE.captures_iter(line) {
            let start = caps.get(0).unwrap().start();
            // Skip image links (preceded by '!').
            if start > 0 && line.as_bytes()[start - 1] == b'!' {
                continue;
            }
            let text = caps[1].trim();
            let dest = caps[2].trim();
            if dest.is_empty() {
                diags.push(Diagnostic {
                    check: "empty-link",
                    category: "readability",
                    severity: Severity::Error,
                    message: "Link has no destination URL".to_string(),
                    file: Some(source_path.to_path_buf()),
                    line: Some(line_num),
                    fix: None,
                });
            } else if text.is_empty() {
                diags.push(Diagnostic {
                    check: "empty-link",
                    category: "readability",
                    severity: Severity::Error,
                    message: "Link has no visible text".to_string(),
                    file: Some(source_path.to_path_buf()),
                    line: Some(line_num),
                    fix: None,
                });
            }
        }
    }
}

/// Flag links whose visible text is a generic, non-descriptive phrase such as
/// "click here" or "here" — these fail WCAG 2.4.4 (link purpose).
fn check_non_descriptive_link_text(
    lines: &[(usize, &str)],
    source_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    const DENY_LIST: &[&str] = &[
        "click here",
        "here",
        "link",
        "more",
        "read more",
        "learn more",
        "this",
        "see here",
        "this link",
        "click",
        "source",
    ];

    for &(line_num, line) in lines {
        for caps in LINK_RE.captures_iter(line) {
            let start = caps.get(0).unwrap().start();
            // Skip image links.
            if start > 0 && line.as_bytes()[start - 1] == b'!' {
                continue;
            }
            let text = caps[1].trim().to_lowercase();
            if DENY_LIST.contains(&text.as_str()) {
                diags.push(Diagnostic {
                    check: "non-descriptive-link-text",
                    category: "readability",
                    severity: Severity::Warning,
                    message: format!("Link text \"{text}\" is not descriptive"),
                    file: Some(source_path.to_path_buf()),
                    line: Some(line_num),
                    fix: None,
                });
            }
        }
    }
}

/// Flag raw `https?://` URLs in prose that are not wrapped in angle brackets
/// (`<url>`) or inside a Markdown link (`[text](url)`).
/// Bare URLs may not render as clickable links in all parsers and are
/// harder for screen readers to announce meaningfully.
fn check_bare_url(lines: &[(usize, &str)], source_path: &Path, diags: &mut Vec<Diagnostic>) {
    for &(line_num, line) in lines {
        let trimmed = line.trim();

        // Reference link definitions (`[ref]: https://...`) are legitimate.
        if trimmed.starts_with('[') && trimmed.contains("]:") {
            continue;
        }

        // Strip contexts where a URL is already properly wrapped.
        let s = INLINE_CODE_RE.replace_all(line, "");
        let s = ANGLE_URL_RE.replace_all(&s, "");
        let s = MD_LINK_OR_IMAGE_RE.replace_all(&s, "");

        for _ in BARE_URL_RE.find_iter(s.as_ref()) {
            diags.push(Diagnostic {
                check: "bare-url",
                category: "readability",
                severity: Severity::Warning,
                message: "Bare URL in prose — wrap it as <url> or [text](url)".to_string(),
                file: Some(source_path.to_path_buf()),
                line: Some(line_num),
                fix: None,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Code block checks
// ---------------------------------------------------------------------------

/// Flag fenced code blocks that have no language tag. Without a tag DocAnvil's
/// syntax highlighter cannot apply highlighting to the block.
fn check_missing_fenced_code_language(
    source: &str,
    source_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    for (line_num, lang) in scan_fence_openers(source) {
        if lang.is_empty() {
            diags.push(Diagnostic {
                check: "missing-fenced-code-language",
                category: "readability",
                severity: Severity::Info,
                message: "Code block has no language tag — syntax highlighting won't apply"
                    .to_string(),
                file: Some(source_path.to_path_buf()),
                line: Some(line_num),
                fix: None,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Prose quality checks
// ---------------------------------------------------------------------------

/// Flag paragraphs whose word count exceeds `config.doctor.max_paragraph_words`.
/// The check is disabled entirely when the threshold is 0.
fn check_long_paragraph(
    lines: &[(usize, &str)],
    source_path: &Path,
    config: &Config,
    diags: &mut Vec<Diagnostic>,
) {
    let threshold = config.doctor.max_paragraph_words;
    if threshold == 0 {
        return;
    }

    let mut word_count = 0usize;
    let mut para_start: Option<usize> = None;

    let mut flush = |word_count: usize, para_start: Option<usize>| {
        if word_count > threshold
            && let Some(start) = para_start
        {
            diags.push(Diagnostic {
                check: "long-paragraph",
                category: "readability",
                severity: Severity::Info,
                message: format!(
                    "Paragraph is very long ({word_count} words; consider breaking it up)"
                ),
                file: Some(source_path.to_path_buf()),
                line: Some(start),
                fix: None,
            });
        }
    };

    for &(line_num, line) in lines {
        let trimmed = line.trim();
        let is_break = trimmed.is_empty()
            || HEADING_RE.is_match(line)
            || trimmed.starts_with(":::")
            || trimmed.starts_with('|');
        if is_break {
            flush(word_count, para_start);
            word_count = 0;
            para_start = None;
        } else {
            if para_start.is_none() {
                para_start = Some(line_num);
            }
            word_count += trimmed.split_whitespace().count();
        }
    }
    flush(word_count, para_start);
}

/// Flag consecutive duplicate words in prose (e.g. "the the", "is is").
/// Inline code spans are excluded to avoid false positives.
fn check_repeated_word(lines: &[(usize, &str)], source_path: &Path, diags: &mut Vec<Diagnostic>) {
    for &(line_num, line) in lines {
        let stripped = INLINE_CODE_RE.replace_all(line, "");
        let words: Vec<&str> = stripped.split_whitespace().collect();
        for pair in words.windows(2) {
            let a = pair[0].trim_matches(|c: char| !c.is_alphabetic());
            let b = pair[1].trim_matches(|c: char| !c.is_alphabetic());
            if a.len() >= 2 && a.eq_ignore_ascii_case(b) {
                diags.push(Diagnostic {
                    check: "repeated-word",
                    category: "readability",
                    severity: Severity::Warning,
                    message: format!("Repeated word: \"{} {}\"", pair[0], pair[1]),
                    file: Some(source_path.to_path_buf()),
                    line: Some(line_num),
                    fix: None,
                });
            }
        }
    }
}

/// Flag editorial annotations (`TODO`, `FIXME`, `HACK`, `XXX`, `PLACEHOLDER`)
/// left in prose. These are unfinished content markers that should not reach
/// readers. Inline code spans are excluded.
fn check_todo_comment(lines: &[(usize, &str)], source_path: &Path, diags: &mut Vec<Diagnostic>) {
    for &(line_num, line) in lines {
        let stripped = INLINE_CODE_RE.replace_all(line, "");
        if let Some(m) = TODO_RE.find(stripped.as_ref()) {
            diags.push(Diagnostic {
                check: "todo-comment",
                category: "readability",
                severity: Severity::Warning,
                message: format!("Editorial annotation in prose: {}", m.as_str().to_uppercase()),
                file: Some(source_path.to_path_buf()),
                line: Some(line_num),
                fix: None,
            });
        }
    }
}

/// Flag common placeholder text (`Lorem ipsum`, `TBD`, `[Insert … here]`)
/// that should never reach readers. Inline code spans are excluded.
fn check_placeholder_text(
    lines: &[(usize, &str)],
    source_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    for &(line_num, line) in lines {
        let stripped = INLINE_CODE_RE.replace_all(line, "");
        if let Some(m) = PLACEHOLDER_RE.find(stripped.as_ref()) {
            diags.push(Diagnostic {
                check: "placeholder-text",
                category: "readability",
                severity: Severity::Warning,
                message: format!("Placeholder text in prose: \"{}\"", m.as_str()),
                file: Some(source_path.to_path_buf()),
                line: Some(line_num),
                fix: None,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fake_path() -> PathBuf {
        PathBuf::from("test.md")
    }

    fn test_config() -> Config {
        Config::default()
    }

    // --- body_start / active_lines ---

    #[test]
    fn active_lines_empty() {
        assert!(active_lines("").is_empty());
    }

    #[test]
    fn active_lines_plain() {
        let src = "line one\nline two\nline three";
        let lines = active_lines(src);
        assert_eq!(lines, vec![(1, "line one"), (2, "line two"), (3, "line three")]);
    }

    #[test]
    fn active_lines_one_indexed() {
        let src = "a\nb\nc";
        let lines = active_lines(src);
        assert_eq!(lines[0].0, 1);
        assert_eq!(lines[1].0, 2);
        assert_eq!(lines[2].0, 3);
    }

    #[test]
    fn active_lines_skips_front_matter() {
        let src = "---\ntitle: Test\n---\nContent here";
        let lines = active_lines(src);
        assert_eq!(lines, vec![(4, "Content here")]);
    }

    #[test]
    fn active_lines_skips_backtick_fence() {
        let src = "before\n```rust\nlet x = 1;\n```\nafter";
        let lines = active_lines(src);
        assert_eq!(lines, vec![(1, "before"), (5, "after")]);
    }

    #[test]
    fn active_lines_skips_tilde_fence() {
        let src = "before\n~~~\ncode here\n~~~\nafter";
        let lines = active_lines(src);
        assert_eq!(lines, vec![(1, "before"), (5, "after")]);
    }

    #[test]
    fn active_lines_mid_body_dash_not_frontmatter() {
        let src = "# Title\n\n---\n\nContent";
        let lines = active_lines(src);
        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0], (1, "# Title"));
        assert_eq!(lines[2], (3, "---"));
        assert_eq!(lines[4], (5, "Content"));
    }

    #[test]
    fn active_lines_lines_after_fence_are_emitted() {
        let src = "```\ncode\n```\nregular line";
        let lines = active_lines(src);
        assert_eq!(lines, vec![(4, "regular line")]);
    }

    // --- scan_fence_openers ---

    #[test]
    fn fence_openers_with_language() {
        let src = "```rust\ncode\n```";
        let openers = scan_fence_openers(src);
        assert_eq!(openers.len(), 1);
        assert_eq!(openers[0], (1, "rust"));
    }

    #[test]
    fn fence_openers_without_language() {
        let src = "```\ncode\n```";
        let openers = scan_fence_openers(src);
        assert_eq!(openers.len(), 1);
        assert_eq!(openers[0], (1, ""));
    }

    #[test]
    fn fence_openers_skips_front_matter() {
        // The "---" front matter delimiter must not be detected as a fence opener.
        let src = "---\n{}\n---\n```rust\ncode\n```";
        let openers = scan_fence_openers(src);
        assert_eq!(openers.len(), 1);
        assert_eq!(openers[0].1, "rust");
    }

    // --- front_matter_has_title ---

    #[test]
    fn fm_title_present() {
        let src = "---\n{\"title\": \"My Page\"}\n---\nContent";
        assert!(front_matter_has_title(src));
    }

    #[test]
    fn fm_title_empty_string() {
        let src = "---\n{\"title\": \"\"}\n---\nContent";
        assert!(!front_matter_has_title(src));
    }

    #[test]
    fn fm_no_title_key() {
        let src = "---\n{\"slug\": \"my-page\"}\n---\nContent";
        assert!(!front_matter_has_title(src));
    }

    #[test]
    fn fm_no_front_matter() {
        assert!(!front_matter_has_title("# Just a heading"));
    }

    // --- check_multiple_h1 ---

    #[test]
    fn multiple_h1_no_issue() {
        let src = "# Title\n## Section\n### Subsection";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_multiple_h1(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn multiple_h1_two_h1s_flagged() {
        let src = "# First\n\n# Second";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_multiple_h1(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "multiple-h1");
        assert_eq!(diags[0].line, Some(3));
    }

    #[test]
    fn multiple_h1_three_h1s_two_warnings() {
        let src = "# A\n# B\n# C";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_multiple_h1(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 2);
    }

    // --- check_skipped_heading_level ---

    #[test]
    fn skipped_heading_sequential_no_issue() {
        let src = "# H1\n## H2\n### H3";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_skipped_heading_level(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn skipped_heading_h1_to_h3_flagged() {
        let src = "# H1\n### H3";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_skipped_heading_level(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "skipped-heading-level");
        assert_eq!(diags[0].line, Some(2));
    }

    #[test]
    fn skipped_heading_decrease_not_flagged() {
        let src = "# H1\n### H3\n## H2";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_skipped_heading_level(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].line, Some(2));
    }

    #[test]
    fn skipped_heading_first_heading_h2_not_flagged() {
        let src = "## Start with H2";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_skipped_heading_level(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_consecutive_headings ---

    #[test]
    fn consecutive_headings_with_content_no_issue() {
        let src = "# Title\n\nSome content here.\n\n## Section";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_consecutive_headings(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn consecutive_headings_blank_only_flagged() {
        let src = "# Title\n\n## Section";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_consecutive_headings(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "consecutive-headings");
        assert_eq!(diags[0].line, Some(3));
    }

    #[test]
    fn consecutive_headings_single_heading_no_issue() {
        let src = "# Only Heading";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_consecutive_headings(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_empty_heading ---

    #[test]
    fn empty_heading_valid_no_issue() {
        let src = "# Valid Heading";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_empty_heading(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn empty_heading_spaces_only_flagged() {
        let src = "## ";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_empty_heading(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "empty-heading");
        assert_eq!(diags[0].severity, Severity::Error);
    }

    // --- check_heading_punctuation ---

    #[test]
    fn heading_punctuation_clean_no_issue() {
        let src = "# Clean Heading";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_heading_punctuation(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn heading_punctuation_period_flagged() {
        let src = "# Ends With Period.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_heading_punctuation(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "heading-punctuation");
        assert_eq!(diags[0].severity, Severity::Info);
    }

    #[test]
    fn heading_punctuation_exclamation_flagged() {
        let src = "# Wow!";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_heading_punctuation(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn heading_punctuation_question_not_flagged() {
        let src = "# What is this?";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_heading_punctuation(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_duplicate_heading_text ---

    #[test]
    fn duplicate_heading_text_no_issue() {
        let src = "# Introduction\n## Installation\n## Configuration";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_duplicate_heading_text(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn duplicate_heading_text_same_text_flagged() {
        let src = "# Overview\n## Details\n## Overview";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_duplicate_heading_text(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "duplicate-heading-text");
        assert_eq!(diags[0].line, Some(3));
    }

    #[test]
    fn duplicate_heading_text_case_insensitive() {
        let src = "# Setup\n## setup";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_duplicate_heading_text(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    // --- check_emphasis_used_as_heading ---

    #[test]
    fn emphasis_heading_clean_bold_flagged() {
        let src = "**Installation**";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_emphasis_used_as_heading(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "emphasis-used-as-heading");
        assert_eq!(diags[0].severity, Severity::Warning);
    }

    #[test]
    fn emphasis_heading_double_underscore_flagged() {
        let src = "__Configuration__";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_emphasis_used_as_heading(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn emphasis_heading_with_sentence_punct_not_flagged() {
        let src = "**Note:** this is important.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_emphasis_used_as_heading(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn emphasis_heading_partial_bold_not_flagged() {
        let src = "See **this section** for more details.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_emphasis_used_as_heading(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_no_document_title ---

    #[test]
    fn no_title_has_h1_no_issue() {
        let src = "# My Page\n\nSome content.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_no_document_title(src, &lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn no_title_has_front_matter_title_no_issue() {
        let src = "---\n{\"title\": \"My Page\"}\n---\nContent without H1.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_no_document_title(src, &lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn no_title_missing_flagged() {
        let src = "Some content with no heading.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_no_document_title(src, &lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "no-document-title");
        assert_eq!(diags[0].severity, Severity::Warning);
    }

    #[test]
    fn no_title_empty_fm_title_flagged() {
        let src = "---\n{\"title\": \"\"}\n---\nContent.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_no_document_title(src, &lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    // --- check_missing_alt_text ---

    #[test]
    fn missing_alt_text_with_alt_no_issue() {
        let src = "![A nice photo](photo.jpg)";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_missing_alt_text(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn missing_alt_text_empty_brackets_flagged() {
        let src = "![](photo.jpg)";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_missing_alt_text(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "missing-alt-text");
        assert_eq!(diags[0].severity, Severity::Warning);
    }

    #[test]
    fn missing_alt_text_multiple_on_one_line() {
        let src = "![](a.png) and ![](b.png)";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_missing_alt_text(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 2);
    }

    #[test]
    fn missing_alt_text_in_code_fence_not_flagged() {
        let src = "```\n![](photo.jpg)\n```";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_missing_alt_text(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_reversed_link_syntax ---

    #[test]
    fn reversed_link_syntax_correct_no_issue() {
        let src = "[See the docs](https://example.com)";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_reversed_link_syntax(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn reversed_link_syntax_flagged() {
        let src = "(See the docs)[https://example.com]";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_reversed_link_syntax(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "reversed-link-syntax");
        assert_eq!(diags[0].severity, Severity::Error);
    }

    #[test]
    fn reversed_link_syntax_in_code_not_flagged() {
        let src = "```\n(text)[url]\n```";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_reversed_link_syntax(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_empty_link ---

    #[test]
    fn empty_link_valid_no_issue() {
        let src = "[Read the docs](https://example.com)";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_empty_link(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn empty_link_empty_destination_flagged() {
        let src = "[Read the docs]()";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_empty_link(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "empty-link");
        assert_eq!(diags[0].severity, Severity::Error);
    }

    #[test]
    fn empty_link_empty_text_flagged() {
        let src = "[](https://example.com)";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_empty_link(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "empty-link");
    }

    #[test]
    fn empty_link_image_not_flagged() {
        // Images with empty alt are caught by missing-alt-text, not empty-link.
        let src = "![](photo.jpg)";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_empty_link(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_non_descriptive_link_text ---

    #[test]
    fn non_descriptive_link_descriptive_no_issue() {
        let src = "[Read the full documentation](https://example.com)";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_non_descriptive_link_text(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn non_descriptive_link_click_here_flagged() {
        let src = "[click here](https://example.com)";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_non_descriptive_link_text(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "non-descriptive-link-text");
        assert_eq!(diags[0].severity, Severity::Warning);
    }

    #[test]
    fn non_descriptive_link_here_flagged() {
        let src = "See [here](https://example.com) for details.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_non_descriptive_link_text(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn non_descriptive_link_image_not_flagged() {
        let src = "![click here](photo.jpg)";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_non_descriptive_link_text(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_bare_url ---

    #[test]
    fn bare_url_in_link_no_issue() {
        let src = "[Example](https://example.com)";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_bare_url(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn bare_url_in_angle_brackets_no_issue() {
        let src = "See <https://example.com> for details.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_bare_url(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn bare_url_in_inline_code_no_issue() {
        let src = "Use `https://example.com` as the base URL.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_bare_url(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn bare_url_in_prose_flagged() {
        let src = "Visit https://example.com for more information.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_bare_url(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "bare-url");
        assert_eq!(diags[0].severity, Severity::Warning);
    }

    #[test]
    fn bare_url_reference_definition_not_flagged() {
        let src = "[docs]: https://example.com";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_bare_url(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn bare_url_in_code_fence_not_flagged() {
        let src = "```\nhttps://example.com\n```";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_bare_url(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_missing_fenced_code_language ---

    #[test]
    fn fenced_code_with_language_no_issue() {
        let src = "```rust\nfn main() {}\n```";
        let mut diags = Vec::new();
        check_missing_fenced_code_language(src, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn fenced_code_without_language_flagged() {
        let src = "```\nsome code\n```";
        let mut diags = Vec::new();
        check_missing_fenced_code_language(src, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "missing-fenced-code-language");
        assert_eq!(diags[0].severity, Severity::Info);
        assert_eq!(diags[0].line, Some(1));
    }

    #[test]
    fn fenced_code_tilde_without_language_flagged() {
        let src = "~~~\nsome code\n~~~";
        let mut diags = Vec::new();
        check_missing_fenced_code_language(src, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "missing-fenced-code-language");
    }

    #[test]
    fn fenced_code_tilde_with_language_no_issue() {
        let src = "~~~python\nprint('hi')\n~~~";
        let mut diags = Vec::new();
        check_missing_fenced_code_language(src, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_long_paragraph ---

    #[test]
    fn long_paragraph_short_no_issue() {
        let src = "Short paragraph.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        let config = test_config();
        check_long_paragraph(&lines, &fake_path(), &config, &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn long_paragraph_over_threshold_flagged() {
        let src = vec!["word"; 151].join(" ");
        let lines = active_lines(&src);
        let mut diags = Vec::new();
        let config = test_config();
        check_long_paragraph(&lines, &fake_path(), &config, &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "long-paragraph");
        assert_eq!(diags[0].severity, Severity::Info);
    }

    #[test]
    fn long_paragraph_exactly_at_threshold_no_issue() {
        let src = vec!["word"; 150].join(" ");
        let lines = active_lines(&src);
        let mut diags = Vec::new();
        let config = test_config();
        check_long_paragraph(&lines, &fake_path(), &config, &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn long_paragraph_disabled_when_zero() {
        let src = vec!["word"; 200].join(" ");
        let lines = active_lines(&src);
        let mut diags = Vec::new();
        let mut config = test_config();
        config.doctor.max_paragraph_words = 0;
        check_long_paragraph(&lines, &fake_path(), &config, &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn long_paragraph_split_by_blank_no_issue() {
        let para = vec!["word"; 100].join(" ");
        let src = format!("{para}\n\n{para}");
        let lines = active_lines(&src);
        let mut diags = Vec::new();
        let config = test_config();
        check_long_paragraph(&lines, &fake_path(), &config, &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_repeated_word ---

    #[test]
    fn repeated_word_no_issue() {
        let src = "The quick brown fox.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_repeated_word(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn repeated_word_flagged() {
        let src = "This is is a problem.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_repeated_word(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "repeated-word");
        assert_eq!(diags[0].severity, Severity::Warning);
    }

    #[test]
    fn repeated_word_case_insensitive() {
        let src = "The the quick fox.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_repeated_word(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn repeated_word_in_inline_code_not_flagged() {
        let src = "Use `the the` syntax.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_repeated_word(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_todo_comment ---

    #[test]
    fn todo_comment_no_issue() {
        let src = "This section explains the feature.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_todo_comment(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn todo_comment_todo_flagged() {
        let src = "TODO: finish writing this section.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_todo_comment(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "todo-comment");
        assert_eq!(diags[0].severity, Severity::Warning);
    }

    #[test]
    fn todo_comment_fixme_flagged() {
        let src = "FIXME: this example is wrong.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_todo_comment(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn todo_comment_case_insensitive() {
        let src = "todo: update this.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_todo_comment(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn todo_comment_in_inline_code_not_flagged() {
        let src = "Add a `// TODO` comment in your code.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_todo_comment(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn todo_comment_in_code_fence_not_flagged() {
        let src = "```rust\n// TODO: fix this\n```";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_todo_comment(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    // --- check_placeholder_text ---

    #[test]
    fn placeholder_text_no_issue() {
        let src = "This section explains authentication.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_placeholder_text(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn placeholder_text_lorem_ipsum_flagged() {
        let src = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_placeholder_text(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "placeholder-text");
        assert_eq!(diags[0].severity, Severity::Warning);
    }

    #[test]
    fn placeholder_text_tbd_flagged() {
        let src = "The release date is TBD.";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_placeholder_text(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn placeholder_text_insert_here_flagged() {
        let src = "[Insert your introduction here]";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_placeholder_text(&lines, &fake_path(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn placeholder_text_in_code_fence_not_flagged() {
        let src = "```\nLorem ipsum\n```";
        let lines = active_lines(src);
        let mut diags = Vec::new();
        check_placeholder_text(&lines, &fake_path(), &mut diags);
        assert!(diags.is_empty());
    }
}
