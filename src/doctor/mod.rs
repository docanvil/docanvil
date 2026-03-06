pub mod checks;

use std::fmt;
use std::path::{Path, PathBuf};

use owo_colors::OwoColorize;

use crate::config::Config;
use crate::project::PageInventory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug)]
pub enum Fix {
    CreateDir(PathBuf),
    CreateFile { path: PathBuf, content: String },
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Diagnostic {
    pub check: &'static str,
    pub category: &'static str,
    pub severity: Severity,
    pub message: String,
    pub file: Option<PathBuf>,
    pub line: Option<usize>,
    pub fix: Option<Fix>,
}

pub struct Summary {
    pub infos: usize,
    pub warnings: usize,
    pub errors: usize,
}

impl Summary {
    pub fn print(&self) {
        let mut parts = Vec::new();
        if self.errors > 0 {
            parts.push(format!(
                "{} error{}",
                self.errors,
                if self.errors == 1 { "" } else { "s" }
            ));
        }
        if self.warnings > 0 {
            parts.push(format!(
                "{} warning{}",
                self.warnings,
                if self.warnings == 1 { "" } else { "s" }
            ));
        }
        if self.infos > 0 {
            parts.push(format!(
                "{} info{}",
                self.infos,
                if self.infos == 1 { "" } else { "s" }
            ));
        }

        if parts.is_empty() {
            eprintln!("{}", "No issues found.".green().bold());
        } else {
            eprintln!("Found {}.", parts.join(", "));
        }
    }

    fn from_diagnostics(diagnostics: &[Diagnostic]) -> Self {
        let mut infos = 0;
        let mut warnings = 0;
        let mut errors = 0;
        for d in diagnostics {
            match d.severity {
                Severity::Info => infos += 1,
                Severity::Warning => warnings += 1,
                Severity::Error => errors += 1,
            }
        }
        Self {
            infos,
            warnings,
            errors,
        }
    }
}

/// Run all doctor checks against a project root.
///
/// When `silent` is `true`, no progress output is written to stderr (for machine-readable modes).
pub fn run_checks(project_root: &Path, silent: bool) -> (Vec<Diagnostic>, Summary) {
    let mut all = Vec::new();

    // Check if config file exists
    let config_path = project_root.join("docanvil.toml");
    if !config_path.exists() {
        if !silent {
            eprintln!("{}", "Checking project structure...".bold());
            eprintln!(
                "  {} No docanvil.toml found. Run `docanvil init <name>` to create a project.",
                "!".red().bold()
            );
        }
        let summary = Summary {
            infos: 0,
            warnings: 0,
            errors: 0,
        };
        return (all, summary);
    }

    // Try to load config
    let config = match Config::load(project_root) {
        Ok(c) => c,
        Err(_) => {
            // Config parse error — run project checks which will report it
            let diags = checks::project::check_project(project_root);
            let summary = Summary::from_diagnostics(&diags);
            return (diags, summary);
        }
    };

    let content_dir = project_root.join(&config.project.content_dir);

    // A. Project structure checks
    if !silent {
        eprintln!("{}", "Checking project structure...".bold());
    }
    let project_diags = checks::project::check_project(project_root);
    if !silent {
        print_check_results(&project_diags);
    }
    let has_content_dir = content_dir.is_dir();
    all.extend(project_diags);

    // Try to build inventory (needed for config and content checks)
    let enabled_locales = if config.is_i18n_enabled() {
        Some(config.locale.enabled.as_slice())
    } else {
        None
    };
    let inventory = if has_content_dir {
        PageInventory::scan(&content_dir, enabled_locales, config.default_locale(), None).ok()
    } else {
        None
    };

    // B. Configuration checks
    if !silent {
        eprintln!("{}", "Checking configuration...".bold());
    }
    let config_diags = checks::config::check_config(project_root, &config, inventory.as_ref());
    if !silent {
        print_check_results(&config_diags);
    }
    all.extend(config_diags);

    // C. Theme checks
    if !silent {
        eprintln!("{}", "Checking theme...".bold());
    }
    let theme_diags = checks::theme::check_theme(project_root, &config);
    if !silent {
        print_check_results(&theme_diags);
    }
    all.extend(theme_diags);

    // D. Content checks
    if let Some(ref inv) = inventory {
        if !silent {
            let page_count = inv.pages.len();
            eprintln!(
                "{}",
                format!(
                    "Checking content ({page_count} page{})...",
                    if page_count == 1 { "" } else { "s" }
                )
                .bold()
            );
        }
        let content_diags = checks::content::check_content(project_root, &config, inv);
        if !silent {
            print_check_results(&content_diags);
        }
        all.extend(content_diags);
    }

    // E. Readability checks
    if let Some(ref inv) = inventory {
        if !silent {
            eprintln!("{}", "Checking readability...".bold());
        }
        let readability_diags = checks::readability::check_readability(project_root, &config, inv);
        if !silent {
            print_check_results(&readability_diags);
        }
        all.extend(readability_diags);
    }

    // F. Locale checks (only when i18n is enabled)
    if config.is_i18n_enabled() {
        if !silent {
            eprintln!("{}", "Checking translations...".bold());
        }
        let locale_diags = checks::locale::check_locale(project_root, &config, inventory.as_ref());
        if !silent {
            print_check_results(&locale_diags);
        }
        all.extend(locale_diags);
    }

    // G1. Version checks (only when versioning is enabled)
    if config.is_versioning_enabled() {
        if !silent {
            eprintln!("{}", "Checking versions...".bold());
        }
        let version_diags = checks::version::check_version(project_root, &config);
        if !silent {
            print_check_results(&version_diags);
        }
        all.extend(version_diags);
    }

    // G. Output checks
    if !silent {
        eprintln!("{}", "Checking output...".bold());
    }
    let output_diags = checks::output::check_output(project_root, &config);
    if !silent {
        print_check_results(&output_diags);
    }
    all.extend(output_diags);

    let summary = Summary::from_diagnostics(&all);
    (all, summary)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Format diagnostics as Checkstyle XML (compatible with reviewdog, GitHub Actions).
pub fn format_checkstyle(diagnostics: &[Diagnostic], project_root: &Path) -> String {
    // Group diagnostics by file path (preserving encounter order)
    let mut groups: Vec<(String, Vec<&Diagnostic>)> = Vec::new();
    for d in diagnostics {
        let name = match &d.file {
            Some(f) => f
                .strip_prefix(project_root)
                .unwrap_or(f)
                .to_string_lossy()
                .into_owned(),
            None => String::new(),
        };
        if let Some(entry) = groups.iter_mut().find(|(n, _)| n == &name) {
            entry.1.push(d);
        } else {
            groups.push((name, vec![d]));
        }
    }

    let mut out =
        String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<checkstyle version=\"4.3\">\n");
    for (name, diags) in &groups {
        out.push_str(&format!("  <file name=\"{}\">\n", xml_escape(name)));
        for d in diags {
            let severity = match d.severity {
                Severity::Info => "info",
                Severity::Warning => "warning",
                Severity::Error => "error",
            };
            let line = d.line.unwrap_or(0);
            let source = format!("docanvil.{}.{}", d.category, d.check);
            out.push_str(&format!(
                "    <error line=\"{line}\" column=\"0\" severity=\"{severity}\" message=\"{message}\" source=\"{source}\"/>\n",
                message = xml_escape(&d.message),
                source = xml_escape(&source),
            ));
        }
        out.push_str("  </file>\n");
    }
    out.push_str("</checkstyle>\n");
    out
}

/// Format diagnostics as JUnit XML (compatible with test result reporters).
pub fn format_junit(diagnostics: &[Diagnostic], project_root: &Path) -> String {
    const KNOWN_CATEGORIES: &[&str] = &[
        "project",
        "config",
        "theme",
        "content",
        "readability",
        "locale",
        "version",
        "output",
    ];

    // Group by category
    let mut by_category: std::collections::HashMap<&str, Vec<&Diagnostic>> =
        std::collections::HashMap::new();
    for d in diagnostics {
        by_category.entry(d.category).or_default().push(d);
    }

    let total_tests: usize = KNOWN_CATEGORIES
        .iter()
        .map(|cat| by_category.get(cat).map(|v| v.len()).unwrap_or(0).max(1))
        .sum();
    let total_failures: usize = diagnostics
        .iter()
        .filter(|d| matches!(d.severity, Severity::Warning | Severity::Error))
        .count();
    let total_skipped: usize = diagnostics
        .iter()
        .filter(|d| matches!(d.severity, Severity::Info))
        .count();

    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <testsuites tests=\"{total_tests}\" failures=\"{total_failures}\" skipped=\"{total_skipped}\">\n"
    );

    for &cat in KNOWN_CATEGORIES {
        let diags = by_category.get(cat).map(|v| v.as_slice()).unwrap_or(&[]);
        let suite_tests = diags.len().max(1);
        let suite_failures = diags
            .iter()
            .filter(|d| matches!(d.severity, Severity::Warning | Severity::Error))
            .count();
        let suite_skipped = diags
            .iter()
            .filter(|d| matches!(d.severity, Severity::Info))
            .count();

        out.push_str(&format!(
            "  <testsuite name=\"docanvil.{cat}\" tests=\"{suite_tests}\" failures=\"{suite_failures}\" skipped=\"{suite_skipped}\">\n"
        ));

        if diags.is_empty() {
            out.push_str(&format!(
                "    <testcase name=\"all-checks-passed\" classname=\"{cat}\"/>\n"
            ));
        } else {
            for d in diags {
                let classname = match &d.file {
                    Some(f) => f
                        .strip_prefix(project_root)
                        .unwrap_or(f)
                        .to_string_lossy()
                        .into_owned(),
                    None => cat.to_string(),
                };
                out.push_str(&format!(
                    "    <testcase name=\"{name}\" classname=\"{classname}\">\n",
                    name = xml_escape(d.check),
                    classname = xml_escape(&classname),
                ));
                match d.severity {
                    Severity::Info => {
                        out.push_str("      <skipped/>\n");
                    }
                    Severity::Warning | Severity::Error => {
                        let location = match (&d.file, d.line) {
                            (Some(f), Some(l)) => {
                                let rel = f
                                    .strip_prefix(project_root)
                                    .unwrap_or(f)
                                    .to_string_lossy()
                                    .into_owned();
                                format!(" at {rel}:{l}")
                            }
                            (Some(f), None) => {
                                let rel = f
                                    .strip_prefix(project_root)
                                    .unwrap_or(f)
                                    .to_string_lossy()
                                    .into_owned();
                                format!(" at {rel}")
                            }
                            _ => String::new(),
                        };
                        let severity_str = d.severity.to_string();
                        out.push_str(&format!(
                            "      <failure type=\"{severity}\" message=\"{message}\"/>\n",
                            severity = xml_escape(&severity_str),
                            message = xml_escape(&format!("{}{}", d.message, location)),
                        ));
                    }
                }
                out.push_str("    </testcase>\n");
            }
        }

        out.push_str("  </testsuite>\n");
    }

    out.push_str("</testsuites>\n");
    out
}

fn print_check_results(diagnostics: &[Diagnostic]) {
    if diagnostics.is_empty() {
        eprintln!("  {} All checks passed", "✓".green().bold());
        return;
    }
    for d in diagnostics {
        let icon = match d.severity {
            Severity::Info => format!("{}", "i".blue().bold()),
            Severity::Warning => format!("{}", "⚠".yellow().bold()),
            Severity::Error => format!("{}", "✗".red().bold()),
        };
        let fixable = if d.fix.is_some() {
            " (fixable: --fix)"
        } else {
            ""
        };
        let location = match (&d.file, d.line) {
            (Some(f), Some(l)) => format!(" in {}:{}", f.display(), l),
            (Some(f), None) => format!(" in {}", f.display()),
            _ => String::new(),
        };
        eprintln!("  {} {}{}{}", icon, d.message, location, fixable);
    }
}

pub fn print_diagnostics(diagnostics: &[Diagnostic]) {
    // Diagnostics are already printed inline during run_checks
    let _ = diagnostics;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_diag(
        check: &'static str,
        category: &'static str,
        severity: Severity,
        message: &str,
        file: Option<PathBuf>,
        line: Option<usize>,
    ) -> Diagnostic {
        Diagnostic {
            check,
            category,
            severity,
            message: message.to_string(),
            file,
            line,
            fix: None,
        }
    }

    // --- xml_escape ---

    #[test]
    fn xml_escape_plain_string() {
        assert_eq!(xml_escape("hello world"), "hello world");
    }

    #[test]
    fn xml_escape_all_special_chars() {
        assert_eq!(
            xml_escape("a & b < c > d \" e '"),
            "a &amp; b &lt; c &gt; d &quot; e &apos;"
        );
    }

    #[test]
    fn xml_escape_ampersand_first_no_double_escape() {
        // "&lt;" should become "&amp;lt;", not "&lt;" (i.e., & is replaced before <)
        assert_eq!(xml_escape("&lt;"), "&amp;lt;");
    }

    // --- format_checkstyle ---

    #[test]
    fn checkstyle_empty_diagnostics() {
        let root = PathBuf::from("/project");
        let xml = format_checkstyle(&[], &root);
        assert!(xml.contains("<checkstyle"));
        assert!(xml.contains("</checkstyle>"));
        // No <file> elements
        assert!(!xml.contains("<file"));
    }

    #[test]
    fn checkstyle_no_file_diagnostic_uses_empty_name() {
        let root = PathBuf::from("/project");
        let diags = vec![make_diag(
            "check-a",
            "config",
            Severity::Warning,
            "bad config",
            None,
            None,
        )];
        let xml = format_checkstyle(&diags, &root);
        assert!(xml.contains("name=\"\""));
    }

    #[test]
    fn checkstyle_file_path_is_relative() {
        let root = PathBuf::from("/project");
        let diags = vec![make_diag(
            "check-b",
            "content",
            Severity::Error,
            "broken link",
            Some(PathBuf::from("/project/docs/page.md")),
            Some(10),
        )];
        let xml = format_checkstyle(&diags, &root);
        assert!(xml.contains("name=\"docs/page.md\""));
        assert!(!xml.contains("/project/docs/page.md"));
    }

    #[test]
    fn checkstyle_severity_and_source_mapping() {
        let root = PathBuf::from("/project");
        let diags = vec![
            make_diag("my-check", "theme", Severity::Info, "just info", None, None),
            make_diag(
                "other-check",
                "output",
                Severity::Error,
                "error msg",
                None,
                None,
            ),
        ];
        let xml = format_checkstyle(&diags, &root);
        assert!(xml.contains("severity=\"info\""));
        assert!(xml.contains("severity=\"error\""));
        assert!(xml.contains("source=\"docanvil.theme.my-check\""));
        assert!(xml.contains("source=\"docanvil.output.other-check\""));
    }

    // --- format_junit ---

    #[test]
    fn junit_empty_diagnostics_produces_passing_suites() {
        let root = PathBuf::from("/project");
        let xml = format_junit(&[], &root);
        assert!(xml.contains("<testsuites"));
        // All 8 known categories should appear
        for cat in &[
            "project",
            "config",
            "theme",
            "content",
            "readability",
            "locale",
            "version",
            "output",
        ] {
            assert!(
                xml.contains(&format!("name=\"docanvil.{cat}\"")),
                "missing suite: {cat}"
            );
        }
        // Every suite has a passing testcase
        assert!(xml.contains("name=\"all-checks-passed\""));
        // No failures or skipped
        assert!(!xml.contains("<failure"));
        assert!(!xml.contains("<skipped"));
    }

    #[test]
    fn junit_warning_produces_failure_element() {
        let root = PathBuf::from("/project");
        let diags = vec![make_diag(
            "long-paragraph",
            "readability",
            Severity::Warning,
            "paragraph too long",
            Some(PathBuf::from("/project/docs/page.md")),
            Some(5),
        )];
        let xml = format_junit(&diags, &root);
        assert!(xml.contains("<failure type=\"warning\""));
        assert!(xml.contains("paragraph too long"));
    }

    #[test]
    fn junit_info_produces_skipped_element() {
        let root = PathBuf::from("/project");
        let diags = vec![make_diag(
            "suggestion",
            "config",
            Severity::Info,
            "consider adding a description",
            None,
            None,
        )];
        let xml = format_junit(&diags, &root);
        assert!(xml.contains("<skipped/>"));
        assert!(!xml.contains("<failure"));
    }

    #[test]
    fn junit_global_counts_correct() {
        let root = PathBuf::from("/project");
        let diags = vec![
            make_diag("c1", "project", Severity::Error, "e1", None, None),
            make_diag("c2", "config", Severity::Warning, "w1", None, None),
            make_diag("c3", "theme", Severity::Info, "i1", None, None),
        ];
        let xml = format_junit(&diags, &root);
        // 3 diagnostics + 5 empty categories (each gets 1 passing testcase) = 8 total tests
        assert!(xml.contains("failures=\"2\""));
        assert!(xml.contains("skipped=\"1\""));
        // tests count: 3 real + 5 passing = 8
        let first_line = xml.lines().find(|l| l.contains("<testsuites")).unwrap();
        assert!(
            first_line.contains("tests=\"8\""),
            "unexpected: {first_line}"
        );
    }
}

/// Apply all safe fixes from the diagnostics, returning the number of fixes applied.
pub fn apply_fixes(diagnostics: &[Diagnostic]) -> usize {
    let mut fixed = 0;

    let fixable: Vec<_> = diagnostics.iter().filter(|d| d.fix.is_some()).collect();
    if fixable.is_empty() {
        return 0;
    }

    eprintln!();
    eprintln!("{}", "Applying fixes...".bold());

    for d in fixable {
        match d.fix.as_ref().unwrap() {
            Fix::CreateDir(path) => {
                if let Err(e) = std::fs::create_dir_all(path) {
                    eprintln!(
                        "  {} Failed to create directory {}: {}",
                        "✗".red().bold(),
                        path.display(),
                        e
                    );
                } else {
                    eprintln!(
                        "  {} Created directory {}",
                        "✓".green().bold(),
                        path.display()
                    );
                    fixed += 1;
                }
            }
            Fix::CreateFile { path, content } => {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::write(path, content) {
                    eprintln!(
                        "  {} Failed to create {}: {}",
                        "✗".red().bold(),
                        path.display(),
                        e
                    );
                } else {
                    eprintln!("  {} Created {}", "✓".green().bold(), path.display());
                    fixed += 1;
                }
            }
        }
    }

    fixed
}
