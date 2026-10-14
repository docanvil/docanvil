use std::path::Path;

use crate::config::Config;
use crate::doctor::{Diagnostic, Fix, Severity};

/// Run version-related checks (only when versioning is enabled).
pub fn check_version(project_root: &Path, config: &Config) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    if !config.is_versioning_enabled() {
        return diags;
    }

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
        let resolved =
            match crate::git_versions::VersionSource::resolve(project_root, config, version) {
                Ok(source) => source,
                Err(error) => {
                    diags.push(Diagnostic {
                        check: "version-source-invalid",
                        category: "version",
                        severity: Severity::Error,
                        message: format!("Version '{version}': {error}"),
                        file: Some(project_root.join("docanvil.toml")),
                        line: None,
                        fix: None,
                    });
                    continue;
                }
            };
        let version_dir = resolved.content_dir.clone();

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
                fix: if config.version.git.refs.contains_key(version) {
                    None
                } else {
                    Some(Fix::CreateDir(version_dir))
                },
            });
            continue;
        }

        // The normal doctor inventory covers working-tree pages. Historical
        // snapshots need their own inventory and include root while still using
        // the combined site's locale and diagnostics settings.
        if resolved.snapshot.is_some() {
            let result = (|| -> crate::error::Result<Vec<Diagnostic>> {
                let mut source_config = Config::load(project_root)?;
                source_config.project.content_dir = resolved
                    .content_dir
                    .strip_prefix(&resolved.project_root)
                    .map_err(|e| crate::error::Error::General(e.to_string()))?
                    .to_owned();
                let inventory = crate::project::PageInventory::scan(
                    &resolved.content_dir,
                    config
                        .is_i18n_enabled()
                        .then_some(config.locale.enabled.as_slice()),
                    config.default_locale(),
                    Some(version),
                )?;
                let mut checks = super::content::check_content(
                    &resolved.project_root,
                    &source_config,
                    &inventory,
                );
                checks.extend(super::readability::check_readability(
                    &resolved.project_root,
                    &source_config,
                    &inventory,
                ));
                if config.is_i18n_enabled() {
                    checks.extend(super::locale::check_locale(
                        &resolved.project_root,
                        &source_config,
                        Some(&inventory),
                    ));
                }
                Ok(checks)
            })();
            match result {
                Ok(checks) => diags.extend(checks.into_iter().map(|mut diagnostic| {
                    let location = diagnostic
                        .file
                        .as_ref()
                        .map(|file| {
                            let relative =
                                file.strip_prefix(&resolved.project_root).unwrap_or(file);
                            match diagnostic.line {
                                Some(line) => format!(" ({}:{line})", relative.display()),
                                None => format!(" ({})", relative.display()),
                            }
                        })
                        .unwrap_or_default();
                    diagnostic.message =
                        format!("Version '{version}'{location}: {}", diagnostic.message);
                    // Temp paths disappear after this check; retain the source ref
                    // in the message and link to the version's configuration.
                    diagnostic.file = Some(project_root.join("docanvil.toml"));
                    diagnostic.line = None;
                    diagnostic.fix = None;
                    diagnostic
                })),
                Err(error) => diags.push(Diagnostic {
                    check: "version-source-invalid",
                    category: "version",
                    severity: Severity::Error,
                    message: format!("Version '{version}': {error}"),
                    file: Some(project_root.join("docanvil.toml")),
                    line: None,
                    fix: None,
                }),
            }
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
