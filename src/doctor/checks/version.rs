use std::path::Path;

use crate::config::Config;
use crate::doctor::{Diagnostic, Fix, Severity};

/// Run version-related checks (only when versioning is enabled).
pub fn check_version(project_root: &Path, config: &Config) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    if !config.is_versioning_enabled() {
        return diags;
    }

    let content_dir = project_root.join(&config.project.content_dir);

    // Check: current-not-in-enabled — version.current is not listed in version.enabled
    if let Some(ref current) = config.version.current
        && !config.version.enabled.contains(current)
    {
        diags.push(Diagnostic {
            check: "current-not-in-enabled",
            category: "version",
            severity: Severity::Error,
            message: format!(
                "version.current '{}' is not in version.enabled {:?} — add it or remove the current setting",
                current, config.version.enabled
            ),
            file: Some(project_root.join("docanvil.toml")),
            line: None,
            fix: None,
        });
    }

    for version in &config.version.enabled {
        let version_dir = content_dir.join(version);

        // Check: version-dir-missing — no subdirectory in the content directory
        if !version_dir.exists() {
            diags.push(Diagnostic {
                check: "version-dir-missing",
                category: "version",
                severity: Severity::Error,
                message: format!(
                    "Version '{}' is enabled but the directory '{}' doesn't exist",
                    version,
                    version_dir.display()
                ),
                file: None,
                line: None,
                fix: Some(Fix::CreateDir(version_dir)),
            });
            continue;
        }

        // Check: empty-version — version directory has no .md files
        let has_md = walkdir::WalkDir::new(&version_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| {
                e.path().extension().is_some_and(|ext| ext == "md") && e.file_type().is_file()
            });

        if !has_md {
            diags.push(Diagnostic {
                check: "empty-version",
                category: "version",
                severity: Severity::Warning,
                message: format!(
                    "Version '{}' directory '{}' contains no Markdown files",
                    version,
                    version_dir.display()
                ),
                file: None,
                line: None,
                fix: None,
            });
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_config(toml: &str) -> Config {
        toml::from_str(toml).unwrap()
    }

    #[test]
    fn no_versioning_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let config = make_config("");
        let diags = check_version(dir.path(), &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn current_not_in_enabled_emits_error() {
        let dir = tempfile::tempdir().unwrap();
        // Bypass config validation by constructing directly
        let mut config = Config::default();
        config.version.current = Some("v3".to_string());
        config.version.enabled = vec!["v1".to_string(), "v2".to_string()];

        let diags = check_version(dir.path(), &config);
        let d = diags.iter().find(|d| d.check == "current-not-in-enabled");
        assert!(d.is_some(), "expected current-not-in-enabled diagnostic");
        assert_eq!(d.unwrap().severity, Severity::Error);
    }

    #[test]
    fn version_dir_missing_emits_error_with_fix() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("docs")).unwrap();

        let mut config = Config::default();
        config.version.enabled = vec!["v1".to_string()];

        let diags = check_version(dir.path(), &config);
        let d = diags.iter().find(|d| d.check == "version-dir-missing");
        assert!(d.is_some(), "expected version-dir-missing diagnostic");
        assert_eq!(d.unwrap().severity, Severity::Error);
        assert!(d.unwrap().fix.is_some(), "should have a fixable CreateDir");
    }

    #[test]
    fn empty_version_dir_emits_warning() {
        let dir = tempfile::tempdir().unwrap();
        let version_dir = dir.path().join("docs/v1");
        fs::create_dir_all(&version_dir).unwrap();

        let mut config = Config::default();
        config.version.enabled = vec!["v1".to_string()];

        let diags = check_version(dir.path(), &config);
        let d = diags.iter().find(|d| d.check == "empty-version");
        assert!(d.is_some(), "expected empty-version diagnostic");
        assert_eq!(d.unwrap().severity, Severity::Warning);
    }

    #[test]
    fn valid_version_dir_with_pages_passes() {
        let dir = tempfile::tempdir().unwrap();
        let version_dir = dir.path().join("docs/v1");
        fs::create_dir_all(&version_dir).unwrap();
        fs::write(version_dir.join("index.md"), "# Home").unwrap();

        let mut config = Config::default();
        config.version.enabled = vec!["v1".to_string()];

        let diags = check_version(dir.path(), &config);
        assert!(
            diags.is_empty(),
            "expected no diagnostics for valid version: {diags:?}"
        );
    }
}
