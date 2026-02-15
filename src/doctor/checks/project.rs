use std::path::Path;

use crate::config::Config;
use crate::doctor::{Diagnostic, Fix, Severity};

/// Check project structure: config file, content directory, index page.
pub fn check_project(project_root: &Path) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Check config file exists
    let config_path = project_root.join("docanvil.toml");
    if !config_path.exists() {
        diags.push(Diagnostic {
            check: "config-missing",
            category: "project",
            severity: Severity::Error,
            message: "Config file docanvil.toml not found".to_string(),
            file: Some(config_path),
            line: None,
            fix: None,
        });
        return diags;
    }

    // Try loading config to check for parse errors
    let config = match Config::load(project_root) {
        Ok(c) => c,
        Err(e) => {
            diags.push(Diagnostic {
                check: "config-parse",
                category: "project",
                severity: Severity::Error,
                message: format!("Config parse error: {e}"),
                file: Some(config_path),
                line: None,
                fix: None,
            });
            return diags;
        }
    };

    // Check content directory exists
    let content_dir = project_root.join(&config.project.content_dir);
    if !content_dir.is_dir() {
        diags.push(Diagnostic {
            check: "content-dir-missing",
            category: "project",
            severity: Severity::Error,
            message: format!("Content directory not found: {}", content_dir.display()),
            file: None,
            line: None,
            fix: Some(Fix::CreateDir(content_dir.clone())),
        });
        return diags;
    }

    // Check for .md files in content dir
    let has_md = walkdir::WalkDir::new(&content_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .any(|e| e.path().extension().is_some_and(|ext| ext == "md") && e.file_type().is_file());

    if !has_md {
        diags.push(Diagnostic {
            check: "no-md-files",
            category: "project",
            severity: Severity::Warning,
            message: format!("No .md files found in {}", content_dir.display()),
            file: None,
            line: None,
            fix: None,
        });
    }

    // Check for index.md at content root
    let index_path = content_dir.join("index.md");
    if !index_path.exists() {
        diags.push(Diagnostic {
            check: "no-index",
            category: "project",
            severity: Severity::Warning,
            message: "No index.md at content root".to_string(),
            file: None,
            line: None,
            fix: Some(Fix::CreateFile {
                path: index_path,
                content: format!(
                    "---\ntitle: Home\n---\n\n# {}\n\nWelcome to your documentation site.\n",
                    config.project.name
                ),
            }),
        });
    }

    diags
}
