//! Plan reference pages without materializing generated Markdown on disk.
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::Api;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::pipeline::frontmatter::FrontMatter;
use crate::project::{GeneratedSource, NavNode, PageInfo, PageInventory};

/// Append generated pages after authored front matter has established its URLs.
/// Specifications are shared across locales; their descriptions are not translated.
pub(crate) fn extend(
    inventory: &mut PageInventory,
    sources: &mut HashMap<String, String>,
    front_matters: &mut HashMap<String, FrontMatter>,
    config: &Config,
    content_dir: &Path,
    version: Option<&str>,
    boundary: Option<&Path>,
) -> Result<()> {
    for entry in &config.openapi {
        if !entry.versions.is_empty() {
            if !config.is_versioning_enabled()
                || entry
                    .versions
                    .iter()
                    .any(|v| !config.version.enabled.contains(v))
            {
                return Err(Error::General(
                    "openapi.versions must name enabled documentation versions".into(),
                ));
            }
            if version.is_none_or(|v| !entry.versions.iter().any(|selected| selected == v)) {
                continue;
            }
        }
        if entry.spec.is_absolute() || entry.spec.as_os_str().is_empty() {
            return Err(Error::General(
                "openapi.spec must be a nonempty relative path from the content directory".into(),
            ));
        }
        if entry.output.is_empty()
            || entry.output.split('/').any(|s| {
                s.is_empty()
                    || s == "."
                    || s == ".."
                    || !s
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
            })
        {
            return Err(Error::General(format!(
                "invalid OpenAPI output namespace {:?}; use slash-separated letters, digits, '-' or '_'",
                entry.output
            )));
        }
        let api = super::load(&content_dir.join(&entry.spec))
            .map_err(|e| Error::General(e.to_string()))?;
        let dependencies: BTreeSet<_> = api.graph.dependencies().map(Path::to_path_buf).collect();
        if let Some(boundary) = boundary {
            let boundary = boundary.canonicalize()?;
            if dependencies.iter().any(|file| !file.starts_with(&boundary)) {
                return Err(Error::General("OpenAPI specifications and references in Git versions must stay inside the repository snapshot".into()));
            }
        }
        let pages = plan(&api, &entry.output)?;
        for (operation, page) in api.operations.iter().zip(pages.iter().skip(1)) {
            inventory
                .operation_methods
                .insert(page.slug.clone(), operation.method.clone());
        }
        let tags = super::render::tag_groups(&api)?;
        if !tags.is_empty() {
            let mut groups: Vec<_> = tags
                .iter()
                .map(|(name, _)| NavNode::Group {
                    label: (*name).to_owned(),
                    slug: None,
                    children: Vec::new(),
                    open: false,
                })
                .collect();
            let mut untagged = Vec::new();
            // Planned operation pages precede schema pages (after the overview).
            for (operation, page) in api.operations.iter().zip(pages.iter().skip(1)) {
                let node = NavNode::Page {
                    label: page.title.clone(),
                    slug: page.slug.clone(),
                };
                if operation.tags.is_empty() {
                    untagged.push(node);
                } else {
                    for ((name, _), group) in tags.iter().zip(&mut groups) {
                        if operation.tags.iter().any(|tag| tag == name)
                            && let NavNode::Group { children, .. } = group
                        {
                            children.push(node.clone());
                        }
                    }
                }
            }
            if !untagged.is_empty() {
                groups.push(NavNode::Group {
                    label: "Untagged operations".into(),
                    slug: None,
                    children: untagged,
                    open: false,
                });
            }
            inventory
                .generated_nav
                .insert(format!("{}/operations", entry.output), groups);
        }
        inventory.generation_warnings.extend(api.warnings);
        let locales: Vec<Option<String>> = if config.is_i18n_enabled() {
            config.locale.enabled.iter().cloned().map(Some).collect()
        } else {
            vec![None]
        };
        for locale in locales {
            for page in &pages {
                let key = locale
                    .as_ref()
                    .map(|l| format!("{l}:{}", page.slug))
                    .unwrap_or_else(|| page.slug.clone());
                let prefix = [version, locale.as_deref()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join("/");
                let output_path = PathBuf::from(if prefix.is_empty() {
                    format!("{}.html", page.slug)
                } else {
                    format!("{prefix}/{}.html", page.slug)
                });
                if let Some(existing) = inventory
                    .pages
                    .get(&key)
                    .or_else(|| inventory.drafts.get(&key))
                {
                    return Err(Error::General(format!(
                        "OpenAPI page {} collides with {}",
                        page.slug,
                        existing.source_path.display()
                    )));
                }
                if inventory
                    .pages
                    .values()
                    .any(|p| p.output_path == output_path)
                    || inventory.slug_aliases.contains_key(&key)
                {
                    return Err(Error::General(format!(
                        "OpenAPI output collision at {}",
                        output_path.display()
                    )));
                }
                inventory.pages.insert(
                    key.clone(),
                    PageInfo {
                        source_path: page.source.clone(),
                        output_path,
                        title: page.title.clone(),
                        slug: page.slug.clone(),
                        locale: locale.clone(),
                        version: version.map(str::to_owned),
                    },
                );
                inventory.ordered.push(key.clone());
                inventory.generated.insert(
                    key.clone(),
                    GeneratedSource {
                        markdown: page.markdown.clone(),
                        html: page.html.clone(),
                        dependencies: dependencies.clone(),
                    },
                );
                sources.insert(key.clone(), page.markdown.clone());
                front_matters.insert(
                    key,
                    FrontMatter {
                        title: Some(page.title.clone()),
                        ..Default::default()
                    },
                );
            }
        }
    }
    Ok(())
}

struct Page {
    slug: String,
    title: String,
    source: PathBuf,
    markdown: String,
    html: String,
}

fn segment(value: &str) -> String {
    let slug = slug::slugify(value);
    if slug.is_empty() || slug.len() > 96 {
        format!("item-{}", hash(value))
    } else {
        slug
    }
}

fn hash(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .take(6)
        .map(|b| format!("{b:02x}"))
        .collect()
}

// Escape inline Markdown, including table delimiters and DocAnvil wiki syntax.
pub(super) fn inline(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace(['\r', '\n'], " ")
        .chars()
        .flat_map(|c| {
            if "\\`*_{}[]()#+-.!|".contains(c) {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect()
}

pub(super) fn code(value: &Value) -> String {
    let json = serde_json::to_string_pretty(value).unwrap();
    let count = 3.max(json.split(|c| c != '`').map(str::len).max().unwrap_or(0) + 1);
    let fence = "`".repeat(count);
    format!("\n{fence}json\n{json}\n{fence}\n\n")
}

pub(super) fn link(slug: &str, title: &str) -> String {
    // Labels originate in a spec and can contain wiki delimiters.
    format!(
        "[[{slug}|{}]]",
        title
            .replace('&', "&amp;")
            .replace('[', "&#91;")
            .replace(']', "&#93;")
            .replace('|', "&#124;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace(['\r', '\n'], " ")
    )
}

fn relative_identity(base: &Path, target: &Path) -> String {
    let base: Vec<_> = base.components().collect();
    let target: Vec<_> = target.components().collect();
    let shared = base.iter().zip(&target).take_while(|(a, b)| a == b).count();
    std::iter::repeat_n("..".to_string(), base.len() - shared)
        .chain(
            target[shared..]
                .iter()
                .map(|p| p.as_os_str().to_string_lossy().into_owned()),
        )
        .collect::<Vec<_>>()
        .join("/")
}

fn plan(api: &Api, namespace: &str) -> Result<Vec<Page>> {
    let mut schema_routes = BTreeMap::new();
    let mut schemas = Vec::new();
    for (name, source) in &api.schemas {
        let slug = format!("{namespace}/schemas/{}", segment(name));
        schema_routes.insert(source.clone(), slug.clone());
        // Map a component alias to its concrete external schema too.
        if let Ok(target) = api.graph.dereference(source) {
            schema_routes.entry(target).or_insert_with(|| slug.clone());
        }
        schemas.push((name.clone(), source.clone(), slug));
    }
    for reference in api.graph.references() {
        if api.graph.is_schema(&reference.target) && !schema_routes.contains_key(&reference.target)
        {
            let identity = format!(
                "{}#{}",
                relative_identity(
                    api.graph.root.file.parent().unwrap(),
                    &reference.target.file
                ),
                reference.target.pointer
            );
            let name = reference
                .target
                .pointer
                .rsplit('/')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    reference
                        .target
                        .file
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("schema")
                });
            let slug = format!("{namespace}/schemas/{}-{}", segment(name), hash(&identity));
            schema_routes.insert(reference.target.clone(), slug.clone());
            schemas.push((name.to_owned(), reference.target.clone(), slug));
        }
    }
    let operations: Vec<_> = api
        .operations
        .iter()
        .map(|operation| {
            let route = operation
                .operation_id
                .as_ref()
                .map(|id| segment(id))
                .unwrap_or_else(|| {
                    format!(
                        "{}-{}-{}",
                        operation.method.to_lowercase(),
                        segment(&operation.path),
                        hash(&format!("{} {}", operation.method, operation.path))
                    )
                });
            (format!("{namespace}/operations/{route}"), operation)
        })
        .collect();
    let mut pages = Vec::new();
    for (slug, operation) in &operations {
        let content = super::render::operation(api, operation, &schema_routes, slug)?;
        pages.push(Page {
            slug: slug.clone(),
            title: operation
                .summary
                .clone()
                .unwrap_or_else(|| format!("{} {}", operation.method, operation.path)),
            source: operation.source.file.clone(),
            markdown: content.markdown,
            html: content.html,
        });
    }
    for (name, source, slug) in &schemas {
        let content = super::render::schema(api, name, source, &schema_routes)?;
        pages.push(Page {
            slug: slug.clone(),
            title: name.clone(),
            source: source.file.clone(),
            markdown: content.markdown,
            html: content.html,
        });
    }
    let overview = super::render::overview(api, &operations, &schemas)?;
    pages.insert(
        0,
        Page {
            slug: format!("{namespace}/index"),
            title: api.title.clone(),
            source: api.graph.root.file.clone(),
            markdown: overview.markdown,
            html: overview.html,
        },
    );
    let mut seen = BTreeSet::new();
    for page in &mut pages {
        // Assign once across the complete page, including all media branches.
        // Link validation and every publishing consumer then share these IDs.
        page.html = crate::pipeline::headings::inject_heading_ids(&page.html);
        if !seen.insert(&page.slug) {
            return Err(Error::General(format!(
                "OpenAPI route collision at {}; use distinct operation IDs/schema names",
                page.slug
            )));
        }
    }
    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{build_breadcrumb_map, flatten_nav_pages};
    use serde_json::json;

    fn tagged_inventory(localized: bool) -> PageInventory {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        std::fs::create_dir_all(docs.join("api/operations/nested")).unwrap();
        std::fs::write(docs.join("api/operations/guide.md"), "# Operation guide").unwrap();
        std::fs::write(docs.join("api/operations/nested/extra.md"), "# Extra").unwrap();
        let mut spec: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/openapi/petstore.json"))
                .unwrap();
        spec["tags"] = json!([{"name": "Zebra"}, {"name": "Empty"}, {"name": "Alpha"}]);
        spec["paths"]["/pets"]["get"]["tags"] = json!(["Alpha", "Zebra", "Alpha", "A/B", "A-B"]);
        std::fs::write(docs.join("openapi.json"), spec.to_string()).unwrap();
        let locale_config = if localized {
            "\n[locale]\ndefault = 'en'\nenabled = ['en', 'fr']\n"
        } else {
            ""
        };
        std::fs::write(
            dir.path().join("docanvil.toml"),
            format!("[project]\nname = 'API'\n[[openapi]]\nspec = 'openapi.json'\n{locale_config}"),
        )
        .unwrap();
        let config = Config::load(dir.path()).unwrap();
        let locales = localized.then_some(config.locale.enabled.as_slice());
        let mut inventory =
            PageInventory::scan(&docs, locales, localized.then_some("en"), Some("v2")).unwrap();
        crate::cli::build::assemble_sources(&mut inventory, &config, &docs, Some("v2"), None)
            .unwrap();
        inventory
    }

    fn assert_tag_groups(nodes: &[NavNode]) {
        let groups: Vec<_> = nodes
            .iter()
            .filter_map(|node| match node {
                NavNode::Group { label, .. } if label != "Nested" => Some(label.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            groups,
            [
                "Zebra",
                "Empty",
                "Alpha",
                "A-B",
                "A/B",
                "Untagged operations"
            ]
        );
        let html = crate::project::render_nav(nodes, "api/operations/listpets", "/manual/v2/fr/");
        assert_eq!(
            html.matches("href=\"/manual/v2/fr/api/operations/listpets.html\"")
                .count(),
            4
        );
        let flat = flatten_nav_pages(nodes);
        assert_eq!(
            flat.iter()
                .filter(|(slug, _)| slug == "api/operations/listpets")
                .count(),
            1
        );
        assert!(
            flat.iter()
                .any(|(slug, _)| slug == "api/operations/createpet")
        );
        let breadcrumbs = build_breadcrumb_map(nodes);
        assert_eq!(breadcrumbs["api/operations/listpets"][0].label, "Zebra");
    }

    #[test]
    fn tag_navigation_handles_discovery_scopes_locales_and_authored_siblings() {
        for localized in [false, true] {
            let inventory = tagged_inventory(localized);
            let full = if localized {
                inventory.nav_tree_for_locale("fr")
            } else {
                inventory.nav_tree()
            };
            let operations = if localized {
                inventory.nav_tree_for_folder_in_locale("api/operations", None, "fr")
            } else {
                inventory.nav_tree_for_folder("api/operations", None)
            };
            assert_tag_groups(&operations);
            let flat = flatten_nav_pages(&full);
            assert_eq!(
                flat.iter()
                    .filter(|(slug, _)| slug == "api/operations/listpets")
                    .count(),
                1
            );
            // Default-locale authored pages are preserved alongside generated tags.
            let default = if localized {
                inventory.nav_tree_for_folder_in_locale("api/operations", None, "en")
            } else {
                inventory.nav_tree_for_folder("api/operations", None)
            };
            let flat = flatten_nav_pages(&default);
            assert!(flat.iter().any(|(slug, _)| slug == "api/operations/guide"));
            assert!(
                flat.iter()
                    .any(|(slug, _)| slug == "api/operations/nested/extra")
            );
            let filtered = if localized {
                inventory.nav_tree_for_folder_in_locale(
                    "api",
                    Some("api/operations/listpets"),
                    "en",
                )
            } else {
                inventory.nav_tree_for_folder("api", Some("api/operations/listpets"))
            };
            assert!(
                !flatten_nav_pages(&filtered)
                    .iter()
                    .any(|(slug, _)| slug == "api/operations/listpets")
            );
            assert!(inventory.nav_tree_for_folder("unrelated", None).is_empty());
            assert!(inventory.nav_tree_for_locale("unknown").is_empty());
        }
    }

    #[test]
    fn explicit_navigation_remains_author_controlled() {
        let inventory = tagged_inventory(false);
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("nav.toml"),
            "[[nav]]\nlabel = 'Chosen operation'\npage = 'api/operations/listpets'\n",
        )
        .unwrap();
        let entries = crate::nav::load_nav(dir.path()).unwrap().unwrap();
        let nodes = crate::nav::nav_tree_from_config(&entries, &inventory);
        assert!(
            matches!(&nodes[..], [NavNode::Page { label, slug }] if label == "Chosen operation" && slug == "api/operations/listpets")
        );
    }
}
