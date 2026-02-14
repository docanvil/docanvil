use std::path::Path;

use crate::config::Config;
use crate::doctor::{Diagnostic, Fix, Severity};

/// Check theme: custom CSS existence, layout template validity.
pub fn check_theme(project_root: &Path, config: &Config) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Check custom CSS file
    if let Some(ref css_path) = config.theme.custom_css {
        let full_path = project_root.join(css_path);
        if !full_path.exists() {
            diags.push(Diagnostic {
                check: "custom-css-not-found",
                category: "theme",
                severity: Severity::Warning,
                message: format!("Custom CSS file not found: {css_path}"),
                file: Some(full_path.clone()),
                line: None,
                fix: Some(Fix::CreateFile {
                    path: full_path,
                    content: String::new(),
                }),
            });
        }
    }

    // Check user layout template for Tera errors
    let user_layout_path = project_root.join("theme/templates/layout.html");
    if user_layout_path.exists() {
        let template_content = match std::fs::read_to_string(&user_layout_path) {
            Ok(content) => content,
            Err(e) => {
                diags.push(Diagnostic {
                    check: "layout-read-error",
                    category: "theme",
                    severity: Severity::Error,
                    message: format!("Cannot read layout template: {e}"),
                    file: Some(user_layout_path),
                    line: None,
                    fix: None,
                });
                return diags;
            }
        };

        let mut tera = tera::Tera::default();
        if let Err(e) = tera.add_raw_template("layout.html", &template_content) {
            diags.push(Diagnostic {
                check: "layout-tera-error",
                category: "theme",
                severity: Severity::Error,
                message: format!("Layout template has Tera errors: {e}"),
                file: Some(user_layout_path),
                line: None,
                fix: None,
            });
        }
    }

    diags
}
