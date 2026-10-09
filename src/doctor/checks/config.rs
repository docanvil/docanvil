use std::path::Path;

use crate::config::{Config, LastUpdatedSource};
use crate::doctor::{Diagnostic, Severity};
use crate::edit::EditLinks;
use crate::last_updated::GitDates;
use crate::nav;
use crate::project::PageInventory;

/// Check configuration validity: file references, nav.toml, URLs.
pub fn check_config(
    project_root: &Path,
    config: &Config,
    inventory: Option<&PageInventory>,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Check logo file exists
    if let Some(ref logo) = config.project.logo {
        let logo_path = project_root.join(logo);
        if !logo_path.exists() {
            diags.push(Diagnostic {
                check: "logo-not-found",
                category: "config",
                severity: Severity::Warning,
                message: format!("Logo file not found: {logo}"),
                file: Some(logo_path),
                line: None,
                fix: None,
            });
        }
    }

    // Check favicon file exists
    if let Some(ref favicon) = config.project.favicon {
        let favicon_path = project_root.join(favicon);
        if !favicon_path.exists() {
            diags.push(Diagnostic {
                check: "favicon-not-found",
                category: "config",
                severity: Severity::Warning,
                message: format!("Favicon file not found: {favicon}"),
                file: Some(favicon_path),
                line: None,
                fix: None,
            });
        }
    }

    // Check site_url
    match &config.build.site_url {
        Some(url) => {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                diags.push(Diagnostic {
                    check: "site-url-no-scheme",
                    category: "config",
                    severity: Severity::Warning,
                    message: format!("site_url should include scheme (http:// or https://): {url}"),
                    file: None,
                    line: None,
                    fix: None,
                });
            }
        }
        None => {
            diags.push(Diagnostic {
                check: "site-url-missing",
                category: "config",
                severity: Severity::Info,
                message: "site_url not set — sitemap.xml will use relative URLs".to_string(),
                file: None,
                line: None,
                fix: None,
            });
        }
    }

    // Check [edit] can produce "Edit this page" links
    if let Err(message) = EditLinks::from_config(&config.edit, project_root) {
        diags.push(Diagnostic {
            check: "edit-link-config",
            category: "config",
            severity: Severity::Warning,
            message,
            file: None,
            line: None,
            fix: None,
        });
    }

    // Check [last_updated] can read Git history
    if config.last_updated.enabled && config.last_updated.source == LastUpdatedSource::Git {
        match GitDates::collect(project_root) {
            Err(message) => diags.push(Diagnostic {
                check: "last-updated-no-git",
                category: "config",
                severity: Severity::Warning,
                message: format!(
                    "{message}. Pages only show front matter dates; set source = \"front-matter\" to skip Git"
                ),
                file: None,
                line: None,
                fix: None,
            }),
            Ok(git) if git.is_shallow() => diags.push(Diagnostic {
                check: "last-updated-shallow-clone",
                category: "config",
                severity: Severity::Warning,
                message: "[last_updated] this is a shallow Git clone, so every page shows the date of the latest commit. Fetch the full history (e.g. `fetch-depth: 0` on actions/checkout)".to_string(),
                file: None,
                line: None,
                fix: None,
            }),
            Ok(_) => {}
        }
    }

    // llms.txt links are only useful to AI tools when they're absolute
    if config.llms.enabled && config.site_url().is_none() {
        diags.push(Diagnostic {
            check: "llms-no-site-url",
            category: "config",
            severity: Severity::Info,
            message: "[llms] is on but [build] site_url isn't set, so links in llms.txt are relative. Set site_url so AI tools can follow them".to_string(),
            file: None,
            line: None,
            fix: None,
        });
    }

    // Validate nav.toml
    let nav_path = project_root.join("nav.toml");
    if nav_path.exists() {
        match nav::load_nav(project_root) {
            Ok(Some(entries)) => {
                // Check for nav references to nonexistent pages
                if let Some(inv) = inventory {
                    check_nav_entries(&entries, inv, &mut diags);
                }
            }
            Ok(None) => {}
            Err(e) => {
                diags.push(Diagnostic {
                    check: "nav-parse-error",
                    category: "config",
                    severity: Severity::Error,
                    message: format!("nav.toml parse error: {e}"),
                    file: Some(nav_path),
                    line: None,
                    fix: None,
                });
            }
        }
    }

    diags
}

/// Check whether a nav slug refers to a page that exists in the inventory.
///
/// With i18n enabled, inventory keys are `"{locale}:{slug}"` — not bare slugs — so
/// a plain `contains_key` check always misses. We fall back to scanning page base
/// slugs so that locale-suffixed projects don't produce false "missing page" warnings.
fn nav_slug_exists(slug: &str, inventory: &PageInventory) -> bool {
    inventory.pages.contains_key(slug) || inventory.pages.values().any(|p| p.slug == slug)
}

fn check_nav_entries(
    entries: &[nav::NavEntry],
    inventory: &PageInventory,
    diags: &mut Vec<Diagnostic>,
) {
    for entry in entries {
        if let Some(slug) = &entry.page
            && !nav_slug_exists(slug, inventory)
        {
            diags.push(Diagnostic {
                check: "nav-missing-page",
                category: "config",
                severity: Severity::Warning,
                message: format!("nav.toml references nonexistent page: {slug}"),
                file: None,
                line: None,
                fix: None,
            });
        }
        if let Some(group) = &entry.group {
            check_nav_group_items(group, inventory, diags);
        }
    }
}

fn check_nav_group_items(
    items: &[nav::NavGroupItem],
    inventory: &PageInventory,
    diags: &mut Vec<Diagnostic>,
) {
    for item in items {
        if let Some(slug) = &item.page
            && !nav_slug_exists(slug, inventory)
        {
            diags.push(Diagnostic {
                check: "nav-missing-page",
                category: "config",
                severity: Severity::Warning,
                message: format!("nav.toml references nonexistent page: {slug}"),
                file: None,
                line: None,
                fix: None,
            });
        }
        if let Some(group) = &item.group {
            check_nav_group_items(group, inventory, diags);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit_diags(edit_toml: &str) -> Vec<Diagnostic> {
        let dir = tempfile::tempdir().unwrap();
        let config: Config = toml::from_str(edit_toml).unwrap();
        check_config(dir.path(), &config, None)
            .into_iter()
            .filter(|d| d.check == "edit-link-config")
            .collect()
    }

    #[test]
    fn edit_link_unknown_host_warns() {
        let diags = edit_diags("[edit]\nrepo = \"https://git.example.com/team/docs\"\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Warning);
        assert!(diags[0].message.contains("provider"));
    }

    #[test]
    fn edit_link_ssh_remote_warns() {
        let diags = edit_diags("[edit]\nrepo = \"git@github.com:org/repo.git\"\n");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn edit_link_valid_or_unset_is_clean() {
        assert!(edit_diags("[edit]\nrepo = \"https://github.com/org/repo\"\n").is_empty());
        assert!(edit_diags("").is_empty());
    }

    fn last_updated_diags(config_toml: &str, project_root: &Path) -> Vec<Diagnostic> {
        let config: Config = toml::from_str(config_toml).unwrap();
        check_config(project_root, &config, None)
            .into_iter()
            .filter(|d| d.check.starts_with("last-updated"))
            .collect()
    }

    #[test]
    fn last_updated_without_repo_warns() {
        if std::process::Command::new("git")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let diags = last_updated_diags("[last_updated]\nenabled = true\n", dir.path());
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "last-updated-no-git");
        assert_eq!(diags[0].severity, Severity::Warning);
    }

    #[test]
    fn last_updated_off_or_front_matter_needs_no_repo() {
        let dir = tempfile::tempdir().unwrap();
        assert!(last_updated_diags("", dir.path()).is_empty());
        assert!(
            last_updated_diags(
                "[last_updated]\nenabled = true\nsource = \"front-matter\"\n",
                dir.path()
            )
            .is_empty()
        );
    }

    fn llms_diags(config_toml: &str) -> Vec<Diagnostic> {
        let config: Config = toml::from_str(config_toml).unwrap();
        let dir = tempfile::tempdir().unwrap();
        check_config(dir.path(), &config, None)
            .into_iter()
            .filter(|d| d.check.starts_with("llms"))
            .collect()
    }

    #[test]
    fn llms_without_site_url_is_info() {
        let diags = llms_diags("[llms]\nenabled = true\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check, "llms-no-site-url");
        assert_eq!(diags[0].severity, Severity::Info);
    }

    #[test]
    fn llms_with_site_url_or_off_is_clean() {
        assert!(llms_diags("").is_empty());
        assert!(
            llms_diags(
                "[build]\nsite_url = \"https://docs.example.com\"\n\n[llms]\nenabled = true\n"
            )
            .is_empty()
        );
    }
}
