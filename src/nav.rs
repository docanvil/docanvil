use std::path::Path;

use serde::Deserialize;

use crate::diagnostics;
use crate::error::Result;
use crate::project::{NavNode, PageInventory};

#[derive(Deserialize)]
struct NavFile {
    nav: Vec<NavEntry>,
}

#[derive(Deserialize, Debug)]
pub struct NavEntry {
    pub page: Option<String>,
    pub label: Option<String>,
    pub separator: Option<SeparatorValue>,
    pub group: Option<Vec<NavGroupItem>>,
    pub autodiscover: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum SeparatorValue {
    Labeled(String),
    Unlabeled(#[allow(dead_code)] bool),
}

#[derive(Deserialize, Debug)]
pub struct NavGroupItem {
    pub page: Option<String>,
    pub label: Option<String>,
    pub separator: Option<SeparatorValue>,
    pub group: Option<Vec<NavGroupItem>>,
    pub autodiscover: Option<String>,
}

/// Shared accessors for the identical fields on `NavEntry` and `NavGroupItem`.
trait NavItem {
    fn page(&self) -> Option<&str>;
    fn label(&self) -> Option<&str>;
    fn separator(&self) -> Option<&SeparatorValue>;
    fn group(&self) -> Option<&[NavGroupItem]>;
    fn autodiscover(&self) -> Option<&str>;
}

impl NavItem for NavEntry {
    fn page(&self) -> Option<&str> {
        self.page.as_deref()
    }
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    fn separator(&self) -> Option<&SeparatorValue> {
        self.separator.as_ref()
    }
    fn group(&self) -> Option<&[NavGroupItem]> {
        self.group.as_deref()
    }
    fn autodiscover(&self) -> Option<&str> {
        self.autodiscover.as_deref()
    }
}

impl NavItem for NavGroupItem {
    fn page(&self) -> Option<&str> {
        self.page.as_deref()
    }
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    fn separator(&self) -> Option<&SeparatorValue> {
        self.separator.as_ref()
    }
    fn group(&self) -> Option<&[NavGroupItem]> {
        self.group.as_deref()
    }
    fn autodiscover(&self) -> Option<&str> {
        self.autodiscover.as_deref()
    }
}

/// Read and parse `nav.toml` from the project root, if it exists.
pub fn load_nav(project_root: &Path) -> Result<Option<Vec<NavEntry>>> {
    let path = project_root.join("nav.toml");
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)?;
    let nav_file: NavFile =
        toml::from_str(&content).map_err(|e| crate::error::Error::ConfigParse {
            path: path.clone(),
            source: e,
        })?;

    Ok(Some(nav_file.nav))
}

/// Load locale-specific nav file: tries `nav.{locale}.toml` first, falls back to `nav.toml`.
pub fn load_nav_for_locale(project_root: &Path, locale: &str) -> Result<Option<Vec<NavEntry>>> {
    let locale_path = project_root.join(format!("nav.{locale}.toml"));
    if locale_path.exists() {
        let content = std::fs::read_to_string(&locale_path)?;
        let nav_file: NavFile =
            toml::from_str(&content).map_err(|e| crate::error::Error::ConfigParse {
                path: locale_path.clone(),
                source: e,
            })?;
        return Ok(Some(nav_file.nav));
    }

    // Fall back to the default nav.toml
    load_nav(project_root)
}

/// Warn about slugs in nav.toml that don't match any page in the inventory.
pub fn validate(entries: &[NavEntry], inventory: &PageInventory) {
    validate_items(entries, inventory, None);
}

/// Warn about slugs in nav.toml that don't match any page for the given locale.
pub fn validate_for_locale(entries: &[NavEntry], inventory: &PageInventory, locale: &str) {
    validate_items(entries, inventory, Some(locale));
}

fn validate_items<T: NavItem>(items: &[T], inventory: &PageInventory, locale: Option<&str>) {
    for item in items {
        if let Some(slug) = item.page() {
            let key = match locale {
                Some(l) => format!("{l}:{slug}"),
                None => slug.to_string(),
            };
            if !inventory.pages.contains_key(&key) {
                diagnostics::warn_nav_missing_page(slug);
            }
        }
        if let Some(folder) = item.autodiscover()
            && inventory.nav_tree_for_folder(folder, None).is_empty()
        {
            diagnostics::warn_nav_autodiscover_empty(folder);
        }
        if let Some(group) = item.group() {
            validate_items(group, inventory, locale);
        }
    }
}

/// Convert parsed nav entries into a NavNode tree.
pub fn nav_tree_from_config(entries: &[NavEntry], inventory: &PageInventory) -> Vec<NavNode> {
    entries
        .iter()
        .flat_map(|entry| item_to_nodes(entry, inventory, None))
        .collect()
}

/// Convert parsed nav entries into a NavNode tree for a specific locale.
/// Nav entries use base slugs; inventory lookup uses `{locale}:{slug}` composite keys.
pub fn nav_tree_from_config_for_locale(
    entries: &[NavEntry],
    inventory: &PageInventory,
    locale: &str,
) -> Vec<NavNode> {
    entries
        .iter()
        .flat_map(|entry| item_to_nodes(entry, inventory, Some(locale)))
        .collect()
}

fn item_to_nodes(
    item: &dyn NavItem,
    inventory: &PageInventory,
    locale: Option<&str>,
) -> Vec<NavNode> {
    // Separator entry
    if let Some(sep) = item.separator() {
        return vec![match sep {
            SeparatorValue::Labeled(label) => NavNode::Separator {
                label: Some(label.clone()),
            },
            SeparatorValue::Unlabeled(_) => NavNode::Separator { label: None },
        }];
    }

    // Autodiscover entry
    if let Some(folder) = item.autodiscover() {
        let exclude = item.page();
        let discovered = if let Some(locale) = locale {
            inventory.nav_tree_for_folder_in_locale(folder, exclude, locale)
        } else {
            inventory.nav_tree_for_folder(folder, exclude)
        };
        if let Some(label) = item.label() {
            return vec![NavNode::Group {
                label: label.to_string(),
                slug: item.page().map(String::from),
                children: discovered,
            }];
        }
        return discovered;
    }

    // Group entry
    if let Some(group_items) = item.group() {
        let label = item.label().unwrap_or_default().to_string();
        let slug = item.page().map(String::from);
        let children: Vec<NavNode> = group_items
            .iter()
            .flat_map(|child| item_to_nodes(child, inventory, locale))
            .collect();
        return vec![NavNode::Group {
            label,
            slug,
            children,
        }];
    }

    // Page entry
    if let Some(slug) = item.page() {
        let label = item
            .label()
            .map(String::from)
            .unwrap_or_else(|| resolve_label(slug, inventory, locale));
        return vec![NavNode::Page {
            label,
            slug: slug.to_string(),
        }];
    }

    vec![]
}

/// Resolve a label for a page slug — use the page title from inventory, or derive from slug.
fn resolve_label(slug: &str, inventory: &PageInventory, locale: Option<&str>) -> String {
    let key = match locale {
        Some(l) => format!("{l}:{slug}"),
        None => slug.to_string(),
    };
    inventory
        .pages
        .get(&key)
        .map(|p| p.title.clone())
        .unwrap_or_else(|| crate::project::title_from_slug(slug))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_nav_toml() {
        let toml_str = r#"
[[nav]]
page = "index"

[[nav]]
separator = "Guides"

[[nav]]
page = "guides/getting-started"
label = "Installation"

[[nav]]
separator = true

[[nav]]
label = "API Reference"
group = [
  { page = "api/overview" },
  { page = "api/endpoints", label = "REST Endpoints" },
]

[[nav]]
label = "Advanced"
page = "advanced/index"
group = [
  { page = "advanced/plugins" },
  { page = "advanced/deployment" },
]
"#;
        let nav_file: NavFile = toml::from_str(toml_str).unwrap();
        assert_eq!(nav_file.nav.len(), 6);

        // First entry: page
        assert_eq!(nav_file.nav[0].page.as_deref(), Some("index"));

        // Second entry: labeled separator
        assert!(matches!(
            &nav_file.nav[1].separator,
            Some(SeparatorValue::Labeled(s)) if s == "Guides"
        ));

        // Third entry: page with label override
        assert_eq!(
            nav_file.nav[2].page.as_deref(),
            Some("guides/getting-started")
        );
        assert_eq!(nav_file.nav[2].label.as_deref(), Some("Installation"));

        // Fourth entry: unlabeled separator
        assert!(matches!(
            &nav_file.nav[3].separator,
            Some(SeparatorValue::Unlabeled(true))
        ));

        // Fifth entry: group without header page
        assert_eq!(nav_file.nav[4].label.as_deref(), Some("API Reference"));
        assert!(nav_file.nav[4].group.is_some());
        assert!(nav_file.nav[4].page.is_none());

        // Sixth entry: group with header page
        assert_eq!(nav_file.nav[5].label.as_deref(), Some("Advanced"));
        assert_eq!(nav_file.nav[5].page.as_deref(), Some("advanced/index"));
        assert!(nav_file.nav[5].group.is_some());
    }

    #[test]
    fn load_nav_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let result = load_nav(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn load_nav_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let nav_content = r#"
[[nav]]
page = "index"

[[nav]]
separator = "Section"

[[nav]]
page = "guide"
"#;
        std::fs::write(dir.path().join("nav.toml"), nav_content).unwrap();
        let result = load_nav(dir.path()).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn nav_tree_from_config_basic() {
        use std::fs;

        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.md"), "# Home").unwrap();
        fs::write(docs.join("guide.md"), "# Guide").unwrap();

        let inventory = PageInventory::scan(&docs, None, None).unwrap();

        let entries = vec![
            NavEntry {
                page: Some("guide".to_string()),
                label: Some("My Guide".to_string()),
                separator: None,
                group: None,
                autodiscover: None,
            },
            NavEntry {
                page: None,
                label: None,
                separator: Some(SeparatorValue::Labeled("Section".to_string())),
                group: None,
                autodiscover: None,
            },
            NavEntry {
                page: Some("index".to_string()),
                label: None,
                separator: None,
                group: None,
                autodiscover: None,
            },
        ];

        let tree = nav_tree_from_config(&entries, &inventory);
        assert_eq!(tree.len(), 3);

        assert!(
            matches!(&tree[0], NavNode::Page { label, slug } if label == "My Guide" && slug == "guide")
        );
        assert!(matches!(&tree[1], NavNode::Separator { label: Some(l) } if l == "Section"));
        assert!(
            matches!(&tree[2], NavNode::Page { label, slug } if label == "Home" && slug == "index")
        );
    }
}
