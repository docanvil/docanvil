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

    /// Build a navigation subtree for pages within a specific folder.
    ///
    /// Filters pages whose slugs start with `{folder}/`, strips the folder prefix,
    /// and builds a subtree using the same logic as `nav_tree`. Optionally excludes
    /// a specific slug (e.g. when a group's header page should not appear as a child).
    pub fn nav_tree_for_folder(&self, folder: &str, exclude_slug: Option<&str>) -> Vec<NavNode> {
        let prefix = format!("{}/", folder.trim_end_matches('/'));
        let mut root: Vec<NavNode> = Vec::new();

        for slug in &self.ordered {
            if let Some(excluded) = exclude_slug
                && slug == excluded
            {
                continue;
            }
            if let Some(rest) = slug.strip_prefix(&prefix) {
                if rest.is_empty() {
                    continue;
                }
                let info = &self.pages[slug];
                let parts: Vec<&str> = rest.split('/').collect();
                insert_nav_node(&mut root, &parts, info);
            }
        }

        root
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
        let dir_node = nodes
            .iter_mut()
            .find(|n| matches!(n, NavNode::Group { label, slug: None, .. } if label == &dir_label));

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

/// Build a map from page slug to its full breadcrumb trail (ancestor group labels + page label).
pub fn build_breadcrumb_map(nodes: &[NavNode]) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    collect_breadcrumbs(nodes, &[], &mut map);
    map
}

fn collect_breadcrumbs(
    nodes: &[NavNode],
    ancestors: &[String],
    map: &mut HashMap<String, Vec<String>>,
) {
    for node in nodes {
        match node {
            NavNode::Page { label, slug } => {
                let mut trail = ancestors.to_vec();
                trail.push(label.clone());
                map.insert(slug.clone(), trail);
            }
            NavNode::Group {
                label,
                slug,
                children,
            } => {
                let mut trail = ancestors.to_vec();
                trail.push(label.clone());
                if let Some(s) = slug {
                    map.insert(s.clone(), trail.clone());
                }
                collect_breadcrumbs(children, &trail, map);
            }
            NavNode::Separator { .. } => {}
        }
    }
}

/// Flatten the nav tree into an ordered list of (slug, label) pairs for prev/next navigation.
pub fn flatten_nav_pages(nodes: &[NavNode]) -> Vec<(String, String)> {
    let mut pages = Vec::new();
    collect_nav_pages(nodes, &mut pages);
    pages
}

fn collect_nav_pages(nodes: &[NavNode], out: &mut Vec<(String, String)>) {
    for node in nodes {
        match node {
            NavNode::Page { label, slug } => {
                out.push((slug.clone(), label.clone()));
            }
            NavNode::Group {
                label,
                slug,
                children,
            } => {
                if let Some(s) = slug {
                    out.push((s.clone(), label.clone()));
                }
                collect_nav_pages(children, out);
            }
            NavNode::Separator { .. } => {}
        }
    }
}

/// Check if a nav section contains the active page.
fn section_contains_active(node: &NavNode, current_slug: &str) -> bool {
    match node {
        NavNode::Page { slug, .. } => slug == current_slug,
        NavNode::Group { slug, children, .. } => {
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
        render_nav_node(node, current_slug, base_url, 0, &mut html);
    }
    html.push_str("</ul>\n");
    html
}

/// Render a single nav node at the given nesting depth.
/// Depth 0 = top-level items (2-space base indent), depth 1+ = nested children (6-space base).
fn render_nav_node(
    node: &NavNode,
    current_slug: &str,
    base_url: &str,
    depth: usize,
    html: &mut String,
) {
    // Compute indentation: depth 0 → 2 spaces, depth 1+ → 2 + 4*depth spaces
    let base = 2 + 4 * depth;
    let indent: String = " ".repeat(base);
    let inner: String = " ".repeat(base + 2);
    let deep: String = " ".repeat(base + 4);

    match node {
        NavNode::Page { label, slug } => {
            let href = format!("{}{}.html", base_url, slug);
            let class = if slug == current_slug {
                "nav-item active"
            } else {
                "nav-item"
            };
            html.push_str(&format!(
                "{indent}<li class=\"{class}\"><a href=\"{href}\">{label}</a></li>\n",
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
                "{indent}<li class=\"nav-group{open_class}\">\n{inner}<button class=\"nav-group-toggle\" aria-expanded=\"{aria}\">\n{deep}<svg class=\"nav-chevron\" viewBox=\"0 0 16 16\" width=\"12\" height=\"12\"><path d=\"M6 4l4 4-4 4\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></svg>\n{deep}{header_html}\n{inner}</button>\n",
            ));
            html.push_str(&format!("{inner}<ul class=\"nav-group-children\">\n"));
            for child in children {
                render_nav_node(child, current_slug, base_url, depth + 1, html);
            }
            html.push_str(&format!("{inner}</ul>\n{indent}</li>\n"));
        }
        NavNode::Separator { label } => {
            if let Some(text) = label {
                html.push_str(&format!(
                    "{indent}<li class=\"nav-separator nav-separator-labeled\">{text}</li>\n"
                ));
            } else {
                html.push_str(&format!("{indent}<li class=\"nav-separator\"><hr></li>\n"));
            }
        }
    }
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

    #[test]
    fn breadcrumb_map_flat_pages() {
        let nodes = vec![
            NavNode::Page {
                label: "Home".into(),
                slug: "index".into(),
            },
            NavNode::Page {
                label: "About".into(),
                slug: "about".into(),
            },
        ];
        let map = build_breadcrumb_map(&nodes);
        assert_eq!(map.get("index").unwrap(), &vec!["Home"]);
        assert_eq!(map.get("about").unwrap(), &vec!["About"]);
    }

    #[test]
    fn breadcrumb_map_nested_groups() {
        let nodes = vec![NavNode::Group {
            label: "Guides".into(),
            slug: None,
            children: vec![NavNode::Group {
                label: "Advanced".into(),
                slug: None,
                children: vec![NavNode::Page {
                    label: "Setup".into(),
                    slug: "guides/advanced/setup".into(),
                }],
            }],
        }];
        let map = build_breadcrumb_map(&nodes);
        assert_eq!(
            map.get("guides/advanced/setup").unwrap(),
            &vec!["Guides", "Advanced", "Setup"]
        );
    }

    #[test]
    fn breadcrumb_map_group_with_header_page() {
        let nodes = vec![NavNode::Group {
            label: "Guides".into(),
            slug: Some("guides/index".into()),
            children: vec![NavNode::Page {
                label: "Setup".into(),
                slug: "guides/setup".into(),
            }],
        }];
        let map = build_breadcrumb_map(&nodes);
        assert_eq!(map.get("guides/index").unwrap(), &vec!["Guides"]);
        assert_eq!(map.get("guides/setup").unwrap(), &vec!["Guides", "Setup"]);
    }

    #[test]
    fn breadcrumb_map_ignores_separators() {
        let nodes = vec![
            NavNode::Separator {
                label: Some("Section".into()),
            },
            NavNode::Page {
                label: "Home".into(),
                slug: "index".into(),
            },
            NavNode::Separator { label: None },
        ];
        let map = build_breadcrumb_map(&nodes);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("index").unwrap(), &vec!["Home"]);
    }
}
