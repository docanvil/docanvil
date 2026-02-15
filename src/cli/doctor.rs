use std::path::Path;

use crate::doctor;
use crate::error::Result;

/// Run the doctor command from CLI.
pub fn run(project_root: &Path, fix: bool, strict: bool, quiet: bool) -> Result<()> {
    let (diagnostics, summary) = doctor::run_checks(project_root);

    if !quiet {
        doctor::print_diagnostics(&diagnostics);
    }

    if fix {
        let fixed = doctor::apply_fixes(&diagnostics);
        if !quiet && fixed > 0 {
            eprintln!();
            eprintln!(
                "Fixed {fixed} issue{}. Run `docanvil doctor` again to verify.",
                if fixed == 1 { "" } else { "s" }
            );
        }
    }

    if !quiet {
        eprintln!();
        summary.print();
    }

    if strict && (summary.warnings > 0 || summary.errors > 0) {
        std::process::exit(1);
    }

    Ok(())
}
