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
    Unlabeled(bool),
}

#[derive(Deserialize, Debug)]
pub struct NavGroupItem {
    pub page: Option<String>,
    pub label: Option<String>,
    pub separator: Option<SeparatorValue>,
    pub group: Option<Vec<NavGroupItem>>,
    pub autodiscover: Option<String>,
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

/// Warn about slugs in nav.toml that don't match any page in the inventory.
pub fn validate(entries: &[NavEntry], inventory: &PageInventory) {
    for entry in entries {
        if let Some(slug) = &entry.page {
            if !inventory.pages.contains_key(slug) {
                diagnostics::warn_nav_missing_page(slug);
            }
        }
        if let Some(folder) = &entry.autodiscover {
            if inventory.nav_tree_for_folder(folder, None).is_empty() {
                diagnostics::warn_nav_autodiscover_empty(folder);
            }
        }
        if let Some(group) = &entry.group {
            validate_group_items(group, inventory);
        }
    }
}

fn validate_group_items(items: &[NavGroupItem], inventory: &PageInventory) {
    for item in items {
        if let Some(slug) = &item.page {
            if !inventory.pages.contains_key(slug) {
                diagnostics::warn_nav_missing_page(slug);
            }
        }
        if let Some(folder) = &item.autodiscover {
            if inventory.nav_tree_for_folder(folder, None).is_empty() {
                diagnostics::warn_nav_autodiscover_empty(folder);
            }
        }
        if let Some(group) = &item.group {
            validate_group_items(group, inventory);
        }
    }
}

/// Convert parsed nav entries into a NavNode tree.
pub fn nav_tree_from_config(entries: &[NavEntry], inventory: &PageInventory) -> Vec<NavNode> {
    entries
        .iter()
        .flat_map(|entry| entry_to_nodes(entry, inventory))
        .collect()
}

fn entry_to_nodes(entry: &NavEntry, inventory: &PageInventory) -> Vec<NavNode> {
    // Separator entry
    if let Some(sep) = &entry.separator {
        return vec![match sep {
            SeparatorValue::Labeled(label) => NavNode::Separator {
                label: Some(label.clone()),
            },
            SeparatorValue::Unlabeled(_) => NavNode::Separator { label: None },
        }];
    }

    // Autodiscover entry
    if let Some(folder) = &entry.autodiscover {
        let exclude = entry.page.as_deref();
        let discovered = inventory.nav_tree_for_folder(folder, exclude);
        if let Some(label) = &entry.label {
            // Wrap in a group
            return vec![NavNode::Group {
                label: label.clone(),
                slug: entry.page.clone(),
                children: discovered,
            }];
        }
        // Inline the discovered nodes
        return discovered;
    }

    // Group entry
    if let Some(group_items) = &entry.group {
        let label = entry.label.clone().unwrap_or_default();
        let slug = entry.page.clone();
        let children = group_items_to_nodes(group_items, inventory);
        return vec![NavNode::Group {
            label,
            slug,
            children,
        }];
    }

    // Page entry
    if let Some(slug) = &entry.page {
        let label = entry
            .label
            .clone()
            .unwrap_or_else(|| resolve_label(slug, inventory));
        return vec![NavNode::Page {
            label,
            slug: slug.clone(),
        }];
    }

    vec![]
}

fn group_items_to_nodes(items: &[NavGroupItem], inventory: &PageInventory) -> Vec<NavNode> {
    items
        .iter()
        .flat_map(|item| group_item_to_nodes(item, inventory))
        .collect()
}

fn group_item_to_nodes(item: &NavGroupItem, inventory: &PageInventory) -> Vec<NavNode> {
    // Separator
    if let Some(sep) = &item.separator {
        return vec![match sep {
            SeparatorValue::Labeled(label) => NavNode::Separator {
                label: Some(label.clone()),
            },
            SeparatorValue::Unlabeled(_) => NavNode::Separator { label: None },
        }];
    }

    // Autodiscover
    if let Some(folder) = &item.autodiscover {
        let exclude = item.page.as_deref();
        let discovered = inventory.nav_tree_for_folder(folder, exclude);
        if let Some(label) = &item.label {
            return vec![NavNode::Group {
                label: label.clone(),
                slug: item.page.clone(),
                children: discovered,
            }];
        }
        return discovered;
    }

    // Nested group
    if let Some(group_items) = &item.group {
        let label = item.label.clone().unwrap_or_default();
        let slug = item.page.clone();
        let children = group_items_to_nodes(group_items, inventory);
        return vec![NavNode::Group {
            label,
            slug,
            children,
        }];
    }

    // Page
    if let Some(slug) = &item.page {
        let label = item
            .label
            .clone()
            .unwrap_or_else(|| resolve_label(slug, inventory));
        return vec![NavNode::Page {
            label,
            slug: slug.clone(),
        }];
    }

    vec![]
}

/// Resolve a label for a page slug — use the page title from inventory, or derive from slug.
fn resolve_label(slug: &str, inventory: &PageInventory) -> String {
    inventory
        .pages
        .get(slug)
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

        let inventory = PageInventory::scan(&docs).unwrap();

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
