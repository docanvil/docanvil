use std::path::Path;

use crate::cli::OutputFormat;
use crate::doctor;
use crate::error::Result;

/// Run the doctor command from CLI.
pub fn run(
    project_root: &Path,
    fix: bool,
    strict: bool,
    quiet: bool,
    format: &OutputFormat,
) -> Result<()> {
    let silent = !matches!(format, OutputFormat::Human);
    let (diagnostics, summary) = doctor::run_checks(project_root, silent);

    if fix {
        let fixed = doctor::apply_fixes(&diagnostics);
        if !silent && !quiet && fixed > 0 {
            eprintln!(
                "\nFixed {fixed} issue{}. Run `docanvil doctor` again to verify.",
                if fixed == 1 { "" } else { "s" }
            );
        }
    }

    match format {
        OutputFormat::Checkstyle => {
            print!("{}", doctor::format_checkstyle(&diagnostics, project_root));
        }
        OutputFormat::Junit => {
            print!("{}", doctor::format_junit(&diagnostics, project_root));
        }
        OutputFormat::Human => {
            if !quiet {
                eprintln!();
                summary.print();
            }
        }
    }

    if strict && (summary.warnings > 0 || summary.errors > 0) {
        return Err(crate::error::Error::DoctorFailed {
            warnings: summary.warnings,
            errors: summary.errors,
        });
    }

    Ok(())
}
