//! Redirect checks: the problems `docanvil build` would warn about, found up front.

use std::path::Path;

use crate::cli::build::scan_site;
use crate::config::Config;
use crate::doctor::{Diagnostic, Severity};
use crate::redirects::{self, CHECK_INVALID, Origin, PageSet};

/// Check front matter `redirect_from` and `[redirects]` against the site as a
/// production build sees it (drafts left out).
pub fn check_redirects(project_root: &Path, config: &Config) -> Vec<Diagnostic> {
    let Ok(sites) = scan_site(project_root, config) else {
        return Vec::new();
    };
    let sets: Vec<PageSet> = sites
        .iter()
        .map(|(version, inventory, front_matters)| PageSet {
            version: version.as_deref(),
            inventory,
            front_matters,
        })
        .collect();

    let config_path = project_root.join("docanvil.toml");
    redirects::plan(config, &sets)
        .problems
        .into_iter()
        .map(|problem| {
            let (category, file, line) = match &problem.origin {
                Origin::FrontMatter(path) => (
                    "content",
                    path.clone(),
                    line_of(path, |l| l.contains("\"redirect_from\"")),
                ),
                Origin::Table(key) => (
                    "config",
                    config_path.clone(),
                    line_of(&config_path, |l| is_toml_key(l, key)),
                ),
                Origin::Unprefixed => (
                    "config",
                    config_path.clone(),
                    line_of(&config_path, |l| is_toml_key(l, "unprefixed")),
                ),
            };
            Diagnostic {
                check: problem.check,
                category,
                severity: if problem.check == CHECK_INVALID {
                    Severity::Error
                } else {
                    Severity::Warning
                },
                message: problem.message,
                file: Some(file),
                line,
                fix: None,
            }
        })
        .collect()
}

/// The 1-based number of the first line of `path` matching `matches`.
fn line_of(path: &Path, matches: impl Fn(&str) -> bool) -> Option<usize> {
    let source = std::fs::read_to_string(path).ok()?;
    source.lines().position(matches).map(|i| i + 1)
}

/// Whether a TOML line defines `key` (quoted or bare).
fn is_toml_key(line: &str, key: &str) -> bool {
    let line = line.trim_start();
    let rest = line
        .strip_prefix(&format!("\"{key}\""))
        .or_else(|| line.strip_prefix(&format!("'{key}'")))
        .or_else(|| line.strip_prefix(key));
    rest.is_some_and(|rest| rest.trim_start().starts_with('='))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doctor(config: &str, files: &[(&str, &str)]) -> (tempfile::TempDir, Vec<Diagnostic>) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("docanvil.toml"), config).unwrap();
        for (path, content) in files {
            let p = dir.path().join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, content).unwrap();
        }
        let config = Config::load(dir.path()).unwrap();
        let diags = check_redirects(dir.path(), &config);
        (dir, diags)
    }

    #[test]
    fn clean_redirects_have_no_diagnostics() {
        let (_dir, diags) = doctor(
            "[project]\nname = \"T\"\n\n[redirects]\n\"old\" = \"guide\"\n",
            &[
                ("docs/guide.md", "# Guide"),
                (
                    "docs/new.md",
                    "---\n{\"redirect_from\": [\"older\"]}\n---\n# New",
                ),
            ],
        );
        assert!(diags.is_empty(), "{diags:?}");
    }

    #[test]
    fn table_problem_points_at_its_line_in_docanvil_toml() {
        let (dir, diags) = doctor(
            "[project]\nname = \"T\"\n\n[redirects]\n\"old\" = \"nowhere\"\n",
            &[("docs/index.md", "# Home")],
        );
        assert_eq!(diags.len(), 1, "{diags:?}");
        let d = &diags[0];
        assert_eq!(d.check, "redirect-target-missing");
        assert_eq!(d.category, "config");
        assert!(matches!(d.severity, Severity::Warning));
        assert_eq!(
            d.file.as_deref(),
            Some(dir.path().join("docanvil.toml").as_path())
        );
        assert_eq!(d.line, Some(5));
    }

    #[test]
    fn invalid_front_matter_is_an_error_at_its_line() {
        let (_dir, diags) = doctor(
            "[project]\nname = \"T\"\n",
            &[(
                "docs/a.md",
                "---\n{\n  \"title\": \"A\",\n  \"redirect_from\": 5\n}\n---\n# A",
            )],
        );
        assert_eq!(diags.len(), 1, "{diags:?}");
        let d = &diags[0];
        assert_eq!(d.check, "redirect-invalid");
        assert_eq!(d.category, "content");
        assert!(matches!(d.severity, Severity::Error));
        assert!(d.file.as_ref().unwrap().ends_with("docs/a.md"));
        assert_eq!(d.line, Some(4));
    }

    #[test]
    fn versioned_sites_are_checked_per_version() {
        let (_dir, diags) = doctor(
            "[project]\nname = \"T\"\n\n[version]\nenabled = [\"v1\", \"v2\"]\n",
            &[
                ("docs/v1/setup.md", "# Setup"),
                ("docs/v2/setup.md", "# Setup"),
                (
                    "docs/v2/install.md",
                    "---\n{\"redirect_from\": [\"setup\"]}\n---\n# Install",
                ),
            ],
        );
        // v2/setup.html is a real page; v1 is unaffected.
        assert_eq!(diags.len(), 1, "{diags:?}");
        assert_eq!(diags[0].check, "redirect-shadowed");
    }
}
