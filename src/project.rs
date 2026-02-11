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
pub enum NavNode {
    Page {
        label: String,
        slug: String,
    },
    Group {
        label: String,
        slug: Option<String>,
        children: Vec<NavNode>,
    },
    Separator {
        label: Option<String>,
    },
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
        nodes.push(NavNode::Page {
            label: info.title.clone(),
            slug: info.slug.clone(),
        });
    } else {
        // Find or create directory node
        let dir_label = title_from_slug(parts[0]);
        let dir_node = nodes.iter_mut().find(|n| {
            matches!(n, NavNode::Group { label, slug: None, .. } if label == &dir_label)
        });

        if let Some(NavNode::Group { children, .. }) = dir_node {
            insert_nav_node(children, &parts[1..], info);
        } else {
            let mut children = Vec::new();
            insert_nav_node(&mut children, &parts[1..], info);
            nodes.push(NavNode::Group {
                label: dir_label,
                slug: None,
                children,
            });
        }
    }
}

/// Convert a slug segment to a human-readable title.
pub(crate) fn title_from_slug(slug: &str) -> String {
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

/// Check if a nav section contains the active page.
fn section_contains_active(node: &NavNode, current_slug: &str) -> bool {
    match node {
        NavNode::Page { slug, .. } => slug == current_slug,
        NavNode::Group {
            slug, children, ..
        } => {
            slug.as_deref() == Some(current_slug)
                || children
                    .iter()
                    .any(|child| section_contains_active(child, current_slug))
        }
        NavNode::Separator { .. } => false,
    }
}

/// Render a navigation tree as nested HTML `<ul>` lists with collapsible groups.
pub fn render_nav(nodes: &[NavNode], current_slug: &str, base_url: &str) -> String {
    if nodes.is_empty() {
        return String::new();
    }

    let mut html = String::from("<ul>\n");
    for node in nodes {
        match node {
            NavNode::Page { label, slug } => {
                let href = format!("{}{}.html", base_url, slug);
                let class = if slug == current_slug {
                    "nav-item active"
                } else {
                    "nav-item"
                };
                html.push_str(&format!(
                    "  <li class=\"{class}\"><a href=\"{href}\">{label}</a></li>\n",
                ));
            }
            NavNode::Group {
                label,
                slug,
                children,
            } => {
                let is_open = section_contains_active(node, current_slug);
                let open_class = if is_open { " open" } else { "" };
                let aria = if is_open { "true" } else { "false" };
                let header_html = if let Some(s) = slug {
                    let href = format!("{}{}.html", base_url, s);
                    let link_class = if s == current_slug { " active" } else { "" };
                    format!("<a href=\"{href}\" class=\"nav-group-link{link_class}\">{label}</a>")
                } else {
                    label.clone()
                };
                html.push_str(&format!(
                    "  <li class=\"nav-group{open_class}\">\n    <button class=\"nav-group-toggle\" aria-expanded=\"{aria}\">\n      <svg class=\"nav-chevron\" viewBox=\"0 0 16 16\" width=\"12\" height=\"12\"><path d=\"M6 4l4 4-4 4\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></svg>\n      {header_html}\n    </button>\n",
                ));
                html.push_str("    <ul class=\"nav-group-children\">\n");
                for child in children {
                    html.push_str(&render_nav_item(child, current_slug, base_url));
                }
                html.push_str("    </ul>\n  </li>\n");
            }
            NavNode::Separator { label } => {
                if let Some(text) = label {
                    html.push_str(&format!(
                        "  <li class=\"nav-separator nav-separator-labeled\">{text}</li>\n"
                    ));
                } else {
                    html.push_str("  <li class=\"nav-separator\"><hr></li>\n");
                }
            }
        }
    }
    html.push_str("</ul>\n");
    html
}

/// Render a single nav item (leaf or nested group) within a group's children.
fn render_nav_item(node: &NavNode, current_slug: &str, base_url: &str) -> String {
    let mut html = String::new();
    match node {
        NavNode::Page { label, slug } => {
            let href = format!("{}{}.html", base_url, slug);
            let class = if slug == current_slug {
                "nav-item active"
            } else {
                "nav-item"
            };
            html.push_str(&format!(
                "      <li class=\"{class}\"><a href=\"{href}\">{label}</a></li>\n",
            ));
        }
        NavNode::Group {
            label,
            slug,
            children,
        } => {
            let is_open = section_contains_active(node, current_slug);
            let open_class = if is_open { " open" } else { "" };
            let aria = if is_open { "true" } else { "false" };
            let header_html = if let Some(s) = slug {
                let href = format!("{}{}.html", base_url, s);
                let link_class = if s == current_slug { " active" } else { "" };
                format!("<a href=\"{href}\" class=\"nav-group-link{link_class}\">{label}</a>")
            } else {
                label.clone()
            };
            html.push_str(&format!(
                "      <li class=\"nav-group{open_class}\">\n        <button class=\"nav-group-toggle\" aria-expanded=\"{aria}\">\n          <svg class=\"nav-chevron\" viewBox=\"0 0 16 16\" width=\"12\" height=\"12\"><path d=\"M6 4l4 4-4 4\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></svg>\n          {header_html}\n        </button>\n",
            ));
            html.push_str("        <ul class=\"nav-group-children\">\n");
            for child in children {
                html.push_str(&render_nav_item(child, current_slug, base_url));
            }
            html.push_str("        </ul>\n      </li>\n");
        }
        NavNode::Separator { label } => {
            if let Some(text) = label {
                html.push_str(&format!(
                    "      <li class=\"nav-separator nav-separator-labeled\">{text}</li>\n"
                ));
            } else {
                html.push_str("      <li class=\"nav-separator\"><hr></li>\n");
            }
        }
    }
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

    #[test]
    fn render_nav_collapsible_output() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(docs.join("guides")).unwrap();
        fs::write(docs.join("index.md"), "# Home").unwrap();
        fs::write(docs.join("guides/setup.md"), "# Setup").unwrap();
        fs::write(docs.join("guides/config.md"), "# Config").unwrap();

        let inv = PageInventory::scan(&docs).unwrap();
        let tree = inv.nav_tree();

        // When viewing a page outside the group, group should be collapsed
        let html = render_nav(&tree, "index", "/");
        assert!(html.contains("nav-group-toggle"));
        assert!(html.contains("nav-chevron"));
        assert!(html.contains("aria-expanded=\"false\""));
        assert!(html.contains("nav-group-children"));
        assert!(html.contains("class=\"nav-item active\""));

        // When viewing a page inside the group, group should be open
        let html_active = render_nav(&tree, "guides/setup", "/");
        assert!(html_active.contains("nav-group open"));
        assert!(html_active.contains("aria-expanded=\"true\""));
    }
}
