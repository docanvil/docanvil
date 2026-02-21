use std::collections::HashSet;
use std::path::Path;

use crate::config::Config;
use crate::doctor::{Diagnostic, Severity};
use crate::project::PageInventory;

/// Run locale-related checks (only when i18n is enabled).
pub fn check_locale(
    project_root: &Path,
    config: &Config,
    inventory: Option<&PageInventory>,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    if !config.is_i18n_enabled() {
        return diags;
    }

    let default = config.default_locale().unwrap_or("en");
    let enabled: &[String] = &config.locale.enabled;

    // Check: default locale not in enabled list (also caught in config validation,
    // but doctor provides a more detailed diagnostic)
    if !enabled.iter().any(|l| l == default) {
        diags.push(Diagnostic {
            check: "default-not-in-enabled",
            category: "locale",
            severity: Severity::Error,
            message: format!(
                "Default locale '{}' is not in the enabled list {:?}",
                default, enabled
            ),
            file: Some(project_root.join("docanvil.toml")),
            line: None,
            fix: None,
        });
    }

    let Some(inventory) = inventory else {
        return diags;
    };

    // Check: missing-default-locale — default locale has no pages at all
    let default_keys = inventory.ordered_for_locale(default);
    if default_keys.is_empty() {
        diags.push(Diagnostic {
            check: "missing-default-locale",
            category: "locale",
            severity: Severity::Error,
            message: format!(
                "Default locale '{}' has no pages — add content files with '.{}.md' suffix",
                default, default
            ),
            file: None,
            line: None,
            fix: None,
        });
    }

    // Check: missing-translation — page exists in some locales but not all
    let coverage = inventory.slug_locale_coverage();
    let enabled_set: HashSet<&str> = enabled.iter().map(|s| s.as_str()).collect();

    for (slug, locales_with_page) in &coverage {
        for locale in &enabled_set {
            if !locales_with_page.contains(*locale) {
                let content_dir = &config.project.content_dir;
                let suggested_file = format!("{}/{}.{}.md", content_dir.display(), slug, locale);
                diags.push(Diagnostic {
                    check: "missing-translation",
                    category: "locale",
                    severity: Severity::Warning,
                    message: format!(
                        "Page '{}' has no translation for locale '{}'. Create '{}' to add one.",
                        slug, locale, suggested_file
                    ),
                    file: None,
                    line: None,
                    fix: None,
                });
            }
        }
    }

    // Check: orphaned-locale — files with locale suffixes not in the enabled list
    for key in &inventory.ordered {
        if let Some(page) = inventory.pages.get(key)
            && let Some(ref locale) = page.locale
            && !enabled_set.contains(locale.as_str())
        {
            diags.push(Diagnostic {
                check: "orphaned-locale",
                category: "locale",
                severity: Severity::Warning,
                message: format!(
                    "File '{}' has locale suffix '{}' which is not in the enabled list",
                    page.source_path.display(),
                    locale
                ),
                file: Some(page.source_path.clone()),
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

    #[test]
    fn no_diags_when_i18n_disabled() {
        let config = Config::default();
        let diags = check_locale(Path::new("."), &config, None);
        assert!(diags.is_empty());
    }

    #[test]
    fn missing_translation_detected() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.en.md"), "# Home").unwrap();
        fs::write(docs.join("index.fr.md"), "# Accueil").unwrap();
        fs::write(docs.join("guide.en.md"), "# Guide").unwrap();
        // guide.fr.md is missing

        let locales = vec!["en".to_string(), "fr".to_string()];
        let inv = PageInventory::scan(&docs, Some(&locales), Some("en")).unwrap();

        let config: Config = toml::from_str(
            r#"
[locale]
default = "en"
enabled = ["en", "fr"]
"#,
        )
        .unwrap();

        let diags = check_locale(dir.path(), &config, Some(&inv));
        let missing: Vec<_> = diags
            .iter()
            .filter(|d| d.check == "missing-translation")
            .collect();
        assert_eq!(missing.len(), 1);
        assert!(missing[0].message.contains("guide"));
        assert!(missing[0].message.contains("fr"));
    }

    #[test]
    fn missing_default_locale_detected() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.fr.md"), "# Accueil").unwrap();
        // No .en.md files at all

        let locales = vec!["en".to_string(), "fr".to_string()];
        let inv = PageInventory::scan(&docs, Some(&locales), Some("en")).unwrap();

        let config: Config = toml::from_str(
            r#"
[locale]
default = "en"
enabled = ["en", "fr"]
"#,
        )
        .unwrap();

        let diags = check_locale(dir.path(), &config, Some(&inv));
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.check == "missing-default-locale")
            .collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].severity, Severity::Error);
    }
}
