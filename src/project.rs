use std::collections::HashMap;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::Result;

/// Metadata for a single documentation page.
#[derive(Debug, Clone)]
pub struct PageInfo {
    /// Absolute path to the source .md file.
    pub source_path: PathBuf,
    /// Relative output path (e.g. `guides/setup.html`).
    pub output_path: PathBuf,
    /// Page title (from first heading or filename).
    pub title: String,
    /// URL-friendly slug used for wiki-link resolution (e.g. `guides/setup`).
    pub slug: String,
}

/// Ordered collection of all pages in the docs directory.
#[derive(Debug)]
pub struct PageInventory {
    /// Slug → PageInfo lookup.
    pub pages: HashMap<String, PageInfo>,
    /// Pages in directory-walk order.
    pub ordered: Vec<String>,
}

/// A node in the navigation tree.
#[derive(Debug, Clone)]
pub struct NavNode {
    pub label: String,
    pub slug: Option<String>,
    pub children: Vec<NavNode>,
}

impl PageInventory {
    /// Scan the content directory and build the page inventory.
    pub fn scan(content_dir: &Path) -> Result<Self> {
        let mut pages = HashMap::new();
        let mut ordered = Vec::new();

        let mut entries: Vec<_> = WalkDir::new(content_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().is_some_and(|ext| ext == "md") && e.file_type().is_file()
            })
            .collect();

        // Sort for deterministic order
        entries.sort_by(|a, b| a.path().cmp(b.path()));

        for entry in entries {
            let path = entry.path().to_path_buf();
            let relative = path.strip_prefix(content_dir).unwrap();

            // Build slug: drop .md extension, use forward slashes
            let slug = relative
                .with_extension("")
                .to_string_lossy()
                .replace('\\', "/");

            // Output path: same structure but .html
            let output_path = relative.with_extension("html");

            // Title: derive from filename (improved later with front-matter/heading extraction)
            let title = title_from_slug(&slug);

            let info = PageInfo {
                source_path: path,
                output_path,
                title,
                slug: slug.clone(),
            };

            ordered.push(slug.clone());
            pages.insert(slug, info);
        }

        Ok(Self { pages, ordered })
    }

    /// Resolve a wiki-link target to a page slug.
    /// Tries exact match first, then basename match.
    pub fn resolve_link(&self, target: &str) -> Option<&PageInfo> {
        let normalized = target.trim().replace('\\', "/");

        // Exact match
        if let Some(page) = self.pages.get(&normalized) {
            return Some(page);
        }

        // Try matching just the last component (basename)
        self.pages.values().find(|p| {
            p.slug
                .rsplit('/')
                .next()
                .is_some_and(|base| base == normalized)
        })
    }

    /// Build a navigation tree from the page inventory.
    pub fn nav_tree(&self) -> Vec<NavNode> {
        let mut root: Vec<NavNode> = Vec::new();

        for slug in &self.ordered {
            let info = &self.pages[slug];
            let parts: Vec<&str> = slug.split('/').collect();
            insert_nav_node(&mut root, &parts, info);
        }

        root
    }
}

fn insert_nav_node(nodes: &mut Vec<NavNode>, parts: &[&str], info: &PageInfo) {
    if parts.is_empty() {
        return;
    }

    if parts.len() == 1 {
        // Leaf page
        nodes.push(NavNode {
            label: info.title.clone(),
            slug: Some(info.slug.clone()),
            children: Vec::new(),
        });
    } else {
        // Find or create directory node
        let dir_label = title_from_slug(parts[0]);
        let dir_node = nodes
            .iter_mut()
            .find(|n| n.label == dir_label && n.slug.is_none());

        if let Some(node) = dir_node {
            insert_nav_node(&mut node.children, &parts[1..], info);
        } else {
            let mut new_node = NavNode {
                label: dir_label,
                slug: None,
                children: Vec::new(),
            };
            insert_nav_node(&mut new_node.children, &parts[1..], info);
            nodes.push(new_node);
        }
    }
}

/// Convert a slug segment to a human-readable title.
fn title_from_slug(slug: &str) -> String {
    let base = slug.rsplit('/').next().unwrap_or(slug);
    if base == "index" {
        return "Home".to_string();
    }
    base.split(['-', '_'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{upper}{}", chars.collect::<String>())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Render a navigation tree as nested HTML `<ul>` lists.
pub fn render_nav(nodes: &[NavNode], current_slug: &str) -> String {
    if nodes.is_empty() {
        return String::new();
    }

    let mut html = String::from("<ul>\n");
    for node in nodes {
        if let Some(slug) = &node.slug {
            let href = format!("/{}.html", slug);
            let active = if slug == current_slug {
                " class=\"active\""
            } else {
                ""
            };
            html.push_str(&format!(
                "  <li{active}><a href=\"{href}\">{}</a></li>\n",
                node.label
            ));
        } else {
            html.push_str(&format!("  <li><span>{}</span>\n", node.label));
            html.push_str(&render_nav(&node.children, current_slug));
            html.push_str("  </li>\n");
        }
    }
    html.push_str("</ul>\n");
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn title_from_slug_basic() {
        assert_eq!(title_from_slug("getting-started"), "Getting Started");
        assert_eq!(title_from_slug("index"), "Home");
        assert_eq!(title_from_slug("api_reference"), "Api Reference");
    }

    #[test]
    fn scan_and_resolve() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(docs.join("guides")).unwrap();
        fs::write(docs.join("index.md"), "# Home").unwrap();
        fs::write(docs.join("guides/setup.md"), "# Setup Guide").unwrap();

        let inv = PageInventory::scan(&docs).unwrap();
        assert_eq!(inv.pages.len(), 2);

        // Exact match
        assert!(inv.resolve_link("guides/setup").is_some());
        // Basename match
        assert!(inv.resolve_link("setup").is_some());
        // Missing
        assert!(inv.resolve_link("nonexistent").is_none());
    }

    #[test]
    fn nav_tree_structure() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(docs.join("guides")).unwrap();
        fs::write(docs.join("index.md"), "# Home").unwrap();
        fs::write(docs.join("guides/setup.md"), "# Setup").unwrap();

        let inv = PageInventory::scan(&docs).unwrap();
        let tree = inv.nav_tree();

        // Should have "Guides" directory node and "Home" page node
        assert!(tree.len() >= 2);
    }
}
