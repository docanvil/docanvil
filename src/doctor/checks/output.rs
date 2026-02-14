use std::path::Path;

use crate::config::Config;
use crate::doctor::{Diagnostic, Severity};

/// Check output directory: parent writability.
pub fn check_output(project_root: &Path, config: &Config) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let output_dir = project_root.join(&config.build.output_dir);

    // Check if the output directory's parent exists and is writable
    let parent = output_dir.parent().unwrap_or(project_root);
    if parent.exists() {
        // Try to check writability by checking metadata
        match parent.metadata() {
            Ok(meta) => {
                if meta.permissions().readonly() {
                    diags.push(Diagnostic {
                        check: "output-dir-not-writable",
                        category: "output",
                        severity: Severity::Error,
                        message: format!(
                            "Output directory parent is not writable: {}",
                            parent.display()
                        ),
                        file: Some(parent.to_path_buf()),
                        line: None,
                        fix: None,
                    });
                }
            }
            Err(e) => {
                diags.push(Diagnostic {
                    check: "output-dir-not-accessible",
                    category: "output",
                    severity: Severity::Error,
                    message: format!(
                        "Cannot access output directory parent {}: {e}",
                        parent.display()
                    ),
                    file: Some(parent.to_path_buf()),
                    line: None,
                    fix: None,
                });
            }
        }
    } else {
        diags.push(Diagnostic {
            check: "output-dir-parent-missing",
            category: "output",
            severity: Severity::Error,
            message: format!(
                "Output directory parent does not exist: {}",
                parent.display()
            ),
            file: Some(parent.to_path_buf()),
            line: None,
            fix: None,
        });
    }

    diags
}
