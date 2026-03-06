use std::path::Path;

use crate::config::Config;
use crate::doctor::{Diagnostic, Severity};
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
