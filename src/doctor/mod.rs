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
pub fn run_checks(project_root: &Path) -> (Vec<Diagnostic>, Summary) {
    let mut all = Vec::new();

    // Check if config file exists
    let config_path = project_root.join("docanvil.toml");
    if !config_path.exists() {
        eprintln!("{}", "Checking project structure...".bold());
        eprintln!(
            "  {} No docanvil.toml found. Run `docanvil init <name>` to create a project.",
            "!".red().bold()
        );
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
    eprintln!("{}", "Checking project structure...".bold());
    let project_diags = checks::project::check_project(project_root);
    print_check_results(&project_diags);
    let has_content_dir = content_dir.is_dir();
    all.extend(project_diags);

    // Try to build inventory (needed for config and content checks)
    let inventory = if has_content_dir {
        PageInventory::scan(&content_dir).ok()
    } else {
        None
    };

    // B. Configuration checks
    eprintln!("{}", "Checking configuration...".bold());
    let config_diags = checks::config::check_config(project_root, &config, inventory.as_ref());
    print_check_results(&config_diags);
    all.extend(config_diags);

    // C. Theme checks
    eprintln!("{}", "Checking theme...".bold());
    let theme_diags = checks::theme::check_theme(project_root, &config);
    print_check_results(&theme_diags);
    all.extend(theme_diags);

    // D. Content checks
    if let Some(ref inv) = inventory {
        let page_count = inv.pages.len();
        eprintln!(
            "{}",
            format!(
                "Checking content ({page_count} page{})...",
                if page_count == 1 { "" } else { "s" }
            )
            .bold()
        );
        let content_diags = checks::content::check_content(project_root, &config, inv);
        print_check_results(&content_diags);
        all.extend(content_diags);
    }

    // E. Output checks
    eprintln!("{}", "Checking output...".bold());
    let output_diags = checks::output::check_output(project_root, &config);
    print_check_results(&output_diags);
    all.extend(output_diags);

    let summary = Summary::from_diagnostics(&all);
    (all, summary)
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
