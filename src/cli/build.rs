use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::Instant;

use crate::components::ComponentRegistry;
use crate::config::Config;
use crate::diagnostics::{reset_warnings, warning_count};
use crate::error::{Error, Result};
use crate::nav;
use crate::pipeline;
use crate::pipeline::frontmatter::{self, FrontMatter};
use crate::pipeline::syntax::SyntaxHighlighter;
use crate::project::{self, PageInventory};
use crate::render::assets;
use crate::render::templates::{LocaleInfo, PageContext, PageLink, TemplateRenderer, VersionInfo};
use crate::search;
use crate::seo;
use crate::theme::Theme;

/// Wrap an IO error with the file path that caused it.
fn io_context(path: &Path) -> impl FnOnce(std::io::Error) -> Error + '_ {
    move |e| Error::General(format!("{}: {e}", path.display()))
}

/// Run the build command from CLI.
pub fn run(project_root: &Path, out: &Path, clean: bool, quiet: bool, strict: bool) -> Result<()> {
    let start = Instant::now();
    let config = Config::load(project_root)?;

    // Resolve output directory
    let output_dir = if out != Path::new("dist") {
        out.to_path_buf()
    } else {
        project_root.join(&config.build.output_dir)
    };

    // Clean output directory if requested
    if clean && output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    reset_warnings();
    crate::pipeline::popovers::reset_popover_ids();

    let count = build_site(project_root, &config, &output_dir, false)?;

    if strict && warning_count() > 0 {
        return Err(Error::StrictWarnings(warning_count()));
    }

    if !quiet {
        let elapsed = start.elapsed();
        eprintln!(
            "Built {count} page{} in {:.0?}",
            if count == 1 { "" } else { "s" },
            elapsed
        );
    }

    Ok(())
}

/// Build with live_reload enabled (used by the dev server).
pub fn run_with_options(project_root: &Path, live_reload: bool) -> Result<()> {
    let config = Config::load(project_root)?;
    let output_dir = project_root.join(&config.build.output_dir);

    reset_warnings();
    crate::pipeline::popovers::reset_popover_ids();

    let count = build_site(project_root, &config, &output_dir, live_reload)?;
    eprintln!("Built {count} page{}", if count == 1 { "" } else { "s" });
    Ok(())
}

/// Core build logic shared between CLI and serve.
fn build_site(
    project_root: &Path,
    config: &Config,
    output_dir: &Path,
    live_reload: bool,
) -> Result<usize> {
    let content_dir = project_root.join(&config.project.content_dir);
    if !content_dir.exists() {
        return Err(Error::ContentDirNotFound(content_dir));
    }

    // Resolve theme and create template renderer
    let theme = Theme::resolve(config, project_root);
    let renderer = TemplateRenderer::new(&theme)?;

    // Build page inventory for wiki-link resolution and navigation
    let enabled_locales = if config.is_i18n_enabled() {
        Some(config.locale.enabled.as_slice())
    } else {
        None
    };
    let mut inventory =
        PageInventory::scan(&content_dir, enabled_locales, config.default_locale(), None)?;

    // Pre-pass: read all sources and extract front matter.
    // Override page titles and slugs from front matter before nav/search are built.
    let mut sources: HashMap<String, String> = HashMap::new();
    let mut front_matters: HashMap<String, FrontMatter> = HashMap::new();
    let mut slug_updates: Vec<(String, String)> = Vec::new();

    for slug in &inventory.ordered {
        let page = &inventory.pages[slug];
        let source =
            std::fs::read_to_string(&page.source_path).map_err(io_context(&page.source_path))?;
        let fm = frontmatter::extract(&source);
        if let Some(ref title) = fm.title
            && let Some(page) = inventory.pages.get_mut(slug)
        {
            page.title = title.clone();
        }

        // Determine slug override: explicit slug field takes priority, then title-derived.
        // Skip title-derived slugs for "index" pages (well-known convention).
        let current_basename = slug.rsplit('/').next().unwrap_or(slug);
        let new_slug = if let Some(ref s) = fm.slug {
            Some(slug::slugify(s))
        } else if let Some(ref title) = fm.title
            && current_basename != "index"
        {
            Some(slug::slugify(title))
        } else {
            None
        };

        if let Some(new_slug) = new_slug {
            // Only update if the slug actually changes (compare against filename portion)
            if new_slug != current_basename {
                slug_updates.push((slug.clone(), new_slug));
            }
        }

        sources.insert(slug.clone(), source);
        front_matters.insert(slug.clone(), fm);
    }

    // Apply slug updates after the loop to avoid mutating while iterating.
    for (old_slug, new_slug) in slug_updates {
        // Re-key source and front matter entries
        if let Some(source) = sources.remove(&old_slug) {
            let fm = front_matters.remove(&old_slug).unwrap_or_default();
            inventory.update_slug(&old_slug, new_slug);
            // Find the new full slug (with directory prefix preserved)
            let full_new_slug = inventory
                .slug_aliases
                .get(&old_slug)
                .cloned()
                .unwrap_or(old_slug);
            sources.insert(full_new_slug.clone(), source);
            front_matters.insert(full_new_slug, fm);
        }
    }

    let registry = ComponentRegistry::with_builtins();

    // Create syntax highlighter if enabled
    let highlighter = if config.syntax.enabled {
        Some(SyntaxHighlighter::new(&config.syntax.theme))
    } else {
        None
    };

    // Dev server always uses "/" — base_url only applies to static builds
    let root_base_url = if live_reload {
        "/".to_string()
    } else {
        config.base_url()
    };
    // Compute logo and favicon paths with root base_url prefix
    let logo_path = config
        .project
        .logo
        .as_ref()
        .map(|p| format!("{}{}", root_base_url, p));
    let favicon_path = config
        .project
        .favicon
        .as_ref()
        .map(|p| format!("{}{}", root_base_url, p));

    // Write JS file to output directory (shared across locales)
    let js_content = if live_reload {
        theme.default_js.clone()
    } else {
        minify_js_source(&theme.default_js)
    };
    std::fs::create_dir_all(output_dir.join("js"))?;
    let js_path = output_dir.join("js/docanvil.js");
    std::fs::write(&js_path, &js_content).map_err(io_context(&js_path))?;

    // Generate cachebust query string from JS content hash
    let js_cachebust = {
        let mut hasher = DefaultHasher::new();
        js_content.hash(&mut hasher);
        format!("?v={:x}", hasher.finish())
    };

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir)?;

    let mut count = 0;

    if config.is_versioning_enabled() {
        // ── Versioned build: outer version loop ──
        //
        // Each version lives in its own subdirectory of content_dir (e.g. docs/v2/).
        // The version dimension is orthogonal to i18n — both can be enabled together.

        // Pre-scan all version directories to know which base slugs exist per version.
        // This powers the version switcher's has_page flag without full re-scans later.
        let version_slug_sets = prescan_version_slugs(&content_dir, config, enabled_locales)?;
        let latest_version = config.current_version().map(String::from);
        let current_ver_str = config.current_version().unwrap_or("").to_string();

        // Collect all version inventories for the post-build sitemap.
        let mut all_version_inventories: Vec<PageInventory> = Vec::new();
        // Save the latest version's nav tree and base URL for the 404 page.
        let mut latest_nav_tree: Vec<project::NavNode> = Vec::new();
        let mut latest_version_base_url = root_base_url.clone();

        for version in &config.version.enabled {
            let version_content_dir = content_dir.join(version);
            if !version_content_dir.exists() {
                return Err(Error::ContentDirNotFound(version_content_dir));
            }

            let version_base_url = format!("{}{}/", root_base_url, version);

            let mut ver_inventory = PageInventory::scan(
                &version_content_dir,
                enabled_locales,
                config.default_locale(),
                Some(version),
            )?;

            // Pre-pass: read all sources and extract front matter for this version.
            let mut ver_sources: HashMap<String, String> = HashMap::new();
            let mut ver_front_matters: HashMap<String, FrontMatter> = HashMap::new();
            let mut slug_updates: Vec<(String, String)> = Vec::new();

            for slug in &ver_inventory.ordered {
                let page = &ver_inventory.pages[slug];
                let source = std::fs::read_to_string(&page.source_path)
                    .map_err(io_context(&page.source_path))?;
                let fm = frontmatter::extract(&source);
                if let Some(ref title) = fm.title
                    && let Some(page) = ver_inventory.pages.get_mut(slug)
                {
                    page.title = title.clone();
                }

                let current_basename = slug.rsplit('/').next().unwrap_or(slug);
                let new_slug = if let Some(ref s) = fm.slug {
                    Some(slug::slugify(s))
                } else if let Some(ref title) = fm.title
                    && current_basename != "index"
                {
                    Some(slug::slugify(title))
                } else {
                    None
                };

                if let Some(new_slug) = new_slug
                    && new_slug != current_basename
                {
                    slug_updates.push((slug.clone(), new_slug));
                }

                ver_sources.insert(slug.clone(), source);
                ver_front_matters.insert(slug.clone(), fm);
            }

            for (old_slug, new_slug) in slug_updates {
                if let Some(source) = ver_sources.remove(&old_slug) {
                    let fm = ver_front_matters.remove(&old_slug).unwrap_or_default();
                    ver_inventory.update_slug(&old_slug, new_slug);
                    let full_new_slug = ver_inventory
                        .slug_aliases
                        .get(&old_slug)
                        .cloned()
                        .unwrap_or(old_slug);
                    ver_sources.insert(full_new_slug.clone(), source);
                    ver_front_matters.insert(full_new_slug, fm);
                }
            }

            if config.is_i18n_enabled() {
                // ── versioned + i18n: per-locale loop ──
                let slug_coverage = ver_inventory.slug_locale_coverage();

                for locale in &config.locale.enabled {
                    let locale_base_url = format!("{}{}/", version_base_url, locale);

                    let nav_config =
                        nav::load_nav_for_version_and_locale(project_root, version, locale)?;
                    let nav_tree = match nav_config {
                        Some(entries) => {
                            nav::validate_for_locale(&entries, &ver_inventory, locale);
                            nav::nav_tree_from_config_for_locale(&entries, &ver_inventory, locale)
                        }
                        None => ver_inventory.nav_tree_for_locale(locale),
                    };
                    let breadcrumb_map = project::build_breadcrumb_map(&nav_tree);

                    let flat_pages = project::flatten_nav_pages(&nav_tree);
                    let mut prev_next_map: HashMap<String, (Option<PageLink>, Option<PageLink>)> =
                        HashMap::new();
                    for (i, (slug, _label)) in flat_pages.iter().enumerate() {
                        let prev = if i > 0 {
                            let (ref ps, ref pl) = flat_pages[i - 1];
                            Some(PageLink {
                                title: pl.clone(),
                                url: format!("{}{}.html", locale_base_url, ps),
                            })
                        } else {
                            None
                        };
                        let next = if i + 1 < flat_pages.len() {
                            let (ref ns, ref nl) = flat_pages[i + 1];
                            Some(PageLink {
                                title: nl.clone(),
                                url: format!("{}{}.html", locale_base_url, ns),
                            })
                        } else {
                            None
                        };
                        prev_next_map.insert(slug.clone(), (prev, next));
                    }

                    let mut search_entries = if config.search.enabled {
                        Some(Vec::new())
                    } else {
                        None
                    };

                    let locale_keys = ver_inventory.ordered_for_locale(locale);
                    for key in &locale_keys {
                        let page = &ver_inventory.pages[key];
                        let source = &ver_sources[key];
                        let fm = &ver_front_matters[key];
                        let base_slug = &page.slug;

                        let html_body = pipeline::process(
                            source,
                            &ver_inventory,
                            &page.source_path,
                            &registry,
                            &root_base_url,
                            highlighter.as_ref(),
                            project_root,
                            Some(locale),
                        )?;

                        if let Some(ref mut entries) = search_entries {
                            let crumbs = breadcrumb_map
                                .get(base_slug)
                                .cloned()
                                .unwrap_or_else(|| vec![page.title.clone()]);
                            let mut sections = search::extract_sections(
                                &html_body,
                                base_slug,
                                &page.title,
                                &locale_base_url,
                                crumbs,
                            );
                            entries.append(&mut sections);
                        }

                        let nav_html = project::render_nav(&nav_tree, base_slug, &locale_base_url);

                        let out_path = output_dir.join(&page.output_path);
                        if let Some(parent) = out_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }

                        let (prev_page, next_page) = prev_next_map
                            .get(base_slug)
                            .cloned()
                            .unwrap_or((None, None));

                        let site_url = config.site_url();
                        let site_url_ref = site_url.as_deref();
                        let available_locales = build_locale_info(
                            config,
                            base_slug,
                            locale,
                            &slug_coverage,
                            &root_base_url,
                            site_url_ref,
                        );

                        let canonical_url = site_url_ref.map(|site| {
                            let site = site.trim_end_matches('/');
                            let path = page.output_path.to_string_lossy().replace('\\', "/");
                            format!("{site}/{path}")
                        });

                        let default_locale = config.default_locale().unwrap_or("en");
                        let x_default_url = available_locales
                            .iter()
                            .find(|l| l.code == default_locale)
                            .and_then(|l| l.absolute_url.clone().or(Some(l.url.clone())));

                        let available_versions = build_version_info(
                            config,
                            base_slug,
                            version,
                            Some(locale),
                            &version_slug_sets,
                            &root_base_url,
                        );
                        let latest_ver_url = latest_version.as_deref().and_then(|lv| {
                            available_versions
                                .iter()
                                .find(|v| v.code == lv)
                                .map(|v| v.url.clone())
                        });

                        let ctx = PageContext {
                            page_title: page.title.clone(),
                            project_name: config.project.name.clone(),
                            content: html_body,
                            nav_html,
                            default_css: theme.default_css.clone(),
                            css_overrides: theme.css_overrides.clone(),
                            custom_css_path: theme.custom_css_path.clone(),
                            custom_css: theme.custom_css.clone(),
                            base_url: root_base_url.clone(),
                            logo_path: logo_path.clone(),
                            favicon_path: favicon_path.clone(),
                            live_reload,
                            mermaid_enabled: config.charts.enabled,
                            mermaid_version: config.charts.mermaid_version.clone(),
                            search_enabled: config.search.enabled,
                            meta_description: fm.description.clone(),
                            meta_author: fm.author.clone(),
                            meta_date: fm.date.clone(),
                            prev_page,
                            next_page,
                            color_mode: config.theme.color_mode.clone(),
                            js_cachebust: js_cachebust.clone(),
                            current_locale: Some(locale.clone()),
                            current_flag: Some(config.locale_flag(locale)),
                            available_locales,
                            locale_auto_detect: config.locale.auto_detect,
                            canonical_url,
                            x_default_url,
                            search_index_url: format!("{}search-index.json", locale_base_url),
                            current_version: Some(version.clone()),
                            available_versions,
                            latest_version: latest_version.clone(),
                            latest_version_url: latest_ver_url,
                        };

                        let html = renderer.render_page(&ctx)?;
                        std::fs::write(&out_path, &html).map_err(io_context(&out_path))?;
                        count += 1;
                    }

                    // Write per-locale search index for this version
                    if let Some(entries) = search_entries {
                        let json = search::build_index(&entries);
                        let path =
                            output_dir.join(format!("{}/{}/search-index.json", version, locale));
                        if let Some(parent) = path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::write(&path, json).map_err(io_context(&path))?;
                    }

                    // Track the latest version's default locale nav for the 404 page
                    if version == &current_ver_str
                        && locale == config.default_locale().unwrap_or("en")
                    {
                        latest_nav_tree = nav_tree;
                        latest_version_base_url = locale_base_url;
                    }
                }

                // Emit missing translation warnings
                for (slug, locales_with_page) in &slug_coverage {
                    for locale in &config.locale.enabled {
                        if !locales_with_page.contains(locale) {
                            crate::diagnostics::warn_missing_translation(slug, locale);
                        }
                    }
                }
            } else {
                // ── versioned + single-language build ──
                let nav_config = nav::load_nav_for_version(project_root, version)?;
                let nav_tree = match nav_config {
                    Some(entries) => {
                        nav::validate(&entries, &ver_inventory);
                        nav::nav_tree_from_config(&entries, &ver_inventory)
                    }
                    None => ver_inventory.nav_tree(),
                };
                let breadcrumb_map = project::build_breadcrumb_map(&nav_tree);

                let flat_pages = project::flatten_nav_pages(&nav_tree);
                let mut prev_next_map: HashMap<String, (Option<PageLink>, Option<PageLink>)> =
                    HashMap::new();
                for (i, (slug, _label)) in flat_pages.iter().enumerate() {
                    let prev = if i > 0 {
                        let (ref ps, ref pl) = flat_pages[i - 1];
                        Some(PageLink {
                            title: pl.clone(),
                            url: format!("{}{}.html", version_base_url, ps),
                        })
                    } else {
                        None
                    };
                    let next = if i + 1 < flat_pages.len() {
                        let (ref ns, ref nl) = flat_pages[i + 1];
                        Some(PageLink {
                            title: nl.clone(),
                            url: format!("{}{}.html", version_base_url, ns),
                        })
                    } else {
                        None
                    };
                    prev_next_map.insert(slug.clone(), (prev, next));
                }

                let mut search_entries = if config.search.enabled {
                    Some(Vec::new())
                } else {
                    None
                };

                for slug in &ver_inventory.ordered {
                    let page = &ver_inventory.pages[slug];
                    let source = &ver_sources[slug];
                    let fm = &ver_front_matters[slug];
                    let base_slug = &page.slug;

                    let html_body = pipeline::process(
                        source,
                        &ver_inventory,
                        &page.source_path,
                        &registry,
                        &root_base_url,
                        highlighter.as_ref(),
                        project_root,
                        None,
                    )?;

                    if let Some(ref mut entries) = search_entries {
                        let crumbs = breadcrumb_map
                            .get(slug)
                            .cloned()
                            .unwrap_or_else(|| vec![page.title.clone()]);
                        let mut sections = search::extract_sections(
                            &html_body,
                            base_slug,
                            &page.title,
                            &version_base_url,
                            crumbs,
                        );
                        entries.append(&mut sections);
                    }

                    let nav_html = project::render_nav(&nav_tree, base_slug, &version_base_url);

                    let out_path = output_dir.join(&page.output_path);
                    if let Some(parent) = out_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }

                    let (prev_page, next_page) =
                        prev_next_map.get(slug).cloned().unwrap_or((None, None));

                    let canonical_url = config.site_url().map(|site| {
                        let site = site.trim_end_matches('/');
                        let path = page.output_path.to_string_lossy().replace('\\', "/");
                        format!("{site}/{path}")
                    });

                    let available_versions = build_version_info(
                        config,
                        base_slug,
                        version,
                        None,
                        &version_slug_sets,
                        &root_base_url,
                    );
                    let latest_ver_url = latest_version.as_deref().and_then(|lv| {
                        available_versions
                            .iter()
                            .find(|v| v.code == lv)
                            .map(|v| v.url.clone())
                    });

                    let ctx = PageContext {
                        page_title: page.title.clone(),
                        project_name: config.project.name.clone(),
                        content: html_body,
                        nav_html,
                        default_css: theme.default_css.clone(),
                        css_overrides: theme.css_overrides.clone(),
                        custom_css_path: theme.custom_css_path.clone(),
                        custom_css: theme.custom_css.clone(),
                        base_url: root_base_url.clone(),
                        logo_path: logo_path.clone(),
                        favicon_path: favicon_path.clone(),
                        live_reload,
                        mermaid_enabled: config.charts.enabled,
                        mermaid_version: config.charts.mermaid_version.clone(),
                        search_enabled: config.search.enabled,
                        meta_description: fm.description.clone(),
                        meta_author: fm.author.clone(),
                        meta_date: fm.date.clone(),
                        prev_page,
                        next_page,
                        color_mode: config.theme.color_mode.clone(),
                        js_cachebust: js_cachebust.clone(),
                        current_locale: None,
                        current_flag: None,
                        available_locales: Vec::new(),
                        locale_auto_detect: false,
                        canonical_url,
                        x_default_url: None,
                        search_index_url: format!("{}search-index.json", version_base_url),
                        current_version: Some(version.clone()),
                        available_versions,
                        latest_version: latest_version.clone(),
                        latest_version_url: latest_ver_url,
                    };

                    let html = renderer.render_page(&ctx)?;
                    std::fs::write(&out_path, &html).map_err(io_context(&out_path))?;
                    count += 1;
                }

                // Write search index for this version
                if let Some(entries) = search_entries {
                    let json = search::build_index(&entries);
                    let path = output_dir.join(format!("{}/search-index.json", version));
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&path, json).map_err(io_context(&path))?;
                }

                // Track the latest version's nav for the 404 page
                if version == &current_ver_str {
                    latest_nav_tree = nav_tree;
                    latest_version_base_url = version_base_url;
                }
            }

            all_version_inventories.push(ver_inventory);
        }

        // Write root redirect to current/latest version
        let redirect_ver = config.current_version().unwrap_or_else(|| {
            config
                .version
                .enabled
                .last()
                .map(|s| s.as_str())
                .unwrap_or("")
        });
        let redirect_target = if config.is_i18n_enabled() {
            let default_locale = config.default_locale().unwrap_or("en");
            format!(
                "{}{}/{}/index.html",
                root_base_url, redirect_ver, default_locale
            )
        } else {
            format!("{}{}/index.html", root_base_url, redirect_ver)
        };
        let redirect_html = format!(
            "<!DOCTYPE html>\n\
             <html>\n\
             <head>\n\
             <meta http-equiv=\"refresh\" content=\"0; url={url}\">\n\
             <link rel=\"canonical\" href=\"{url}\">\n\
             </head>\n\
             <body>\n\
             <p><a href=\"{url}\">Redirecting to latest documentation...</a></p>\n\
             </body>\n\
             </html>\n",
            url = redirect_target
        );
        let redirect_path = output_dir.join("index.html");
        std::fs::write(&redirect_path, redirect_html).map_err(io_context(&redirect_path))?;

        // Generate robots.txt and sitemap (merged across all versions)
        if !live_reload {
            let site_url = config.site_url();
            if site_url.is_none() {
                crate::diagnostics::warn_no_site_url();
            }

            let sitemap_url = site_url.as_deref().map(|u| format!("{u}sitemap.xml"));
            let robots = seo::generate_robots_txt(sitemap_url.as_deref());
            let robots_path = output_dir.join("robots.txt");
            std::fs::write(&robots_path, robots).map_err(io_context(&robots_path))?;

            // Merge all version inventories into one for sitemap generation.
            // Output paths already include version prefixes, so URLs are correct.
            // No hreflang annotations for versions — versions aren't translations.
            let mut merged_pages: HashMap<String, project::PageInfo> = HashMap::new();
            let mut merged_ordered: Vec<String> = Vec::new();
            for inv in &all_version_inventories {
                for key in &inv.ordered {
                    let page = &inv.pages[key];
                    let unique_key = page.output_path.to_string_lossy().into_owned();
                    merged_pages.insert(unique_key.clone(), page.clone());
                    merged_ordered.push(unique_key);
                }
            }
            let merged_inv = PageInventory {
                pages: merged_pages,
                ordered: merged_ordered,
                slug_aliases: HashMap::new(),
                discovered_locales: HashSet::new(),
            };
            let sitemap =
                seo::generate_sitemap_xml(&merged_inv, &root_base_url, site_url.as_deref(), None);
            let sitemap_path = output_dir.join("sitemap.xml");
            std::fs::write(&sitemap_path, sitemap).map_err(io_context(&sitemap_path))?;
        }

        // Generate 404 page using the latest version's nav
        {
            let nav_html = project::render_nav(&latest_nav_tree, "", &latest_version_base_url);

            let mut not_found_links = String::from(
                "<div class=\"not-found\">\
                 <h1>404</h1>\
                 <p>The page you're looking for doesn't exist.</p>\
                 <p>",
            );
            for ver in &config.version.enabled {
                let display = config.version_display_name(ver);
                let ver_home = if config.is_i18n_enabled() {
                    let default_locale = config.default_locale().unwrap_or("en");
                    format!("{}{}/{}/index.html", root_base_url, ver, default_locale)
                } else {
                    format!("{}{}/index.html", root_base_url, ver)
                };
                not_found_links.push_str(&format!("<a href=\"{}\">{}</a> ", ver_home, display));
            }
            not_found_links.push_str("</p></div>");

            let search_index_url_404 = format!("{}search-index.json", latest_version_base_url);
            let ctx = PageContext {
                page_title: "Page Not Found".to_string(),
                project_name: config.project.name.clone(),
                content: not_found_links,
                nav_html,
                default_css: theme.default_css.clone(),
                css_overrides: theme.css_overrides.clone(),
                custom_css_path: theme.custom_css_path.clone(),
                custom_css: theme.custom_css.clone(),
                base_url: root_base_url.clone(),
                logo_path: logo_path.clone(),
                favicon_path: favicon_path.clone(),
                live_reload,
                mermaid_enabled: false,
                mermaid_version: String::new(),
                search_enabled: config.search.enabled,
                meta_description: None,
                meta_author: None,
                meta_date: None,
                prev_page: None,
                next_page: None,
                color_mode: config.theme.color_mode.clone(),
                js_cachebust: js_cachebust.clone(),
                current_locale: None,
                current_flag: None,
                available_locales: Vec::new(),
                locale_auto_detect: false,
                canonical_url: None,
                x_default_url: None,
                search_index_url: search_index_url_404,
                current_version: None,
                available_versions: Vec::new(),
                latest_version: None,
                latest_version_url: None,
            };
            let html = renderer.render_page(&ctx)?;
            let not_found_path = output_dir.join("404.html");
            std::fs::write(&not_found_path, html).map_err(io_context(&not_found_path))?;
        }

        assets::copy_assets(project_root, output_dir, config.theme.custom_css.as_deref())?;

        return Ok(count);
    }

    if config.is_i18n_enabled() {
        // ── i18n build: per-locale loop ──
        let slug_coverage = inventory.slug_locale_coverage();

        for locale in &config.locale.enabled {
            let locale_base_url = format!("{}{}/", root_base_url, locale);

            // Load locale-specific nav
            let nav_config = nav::load_nav_for_locale(project_root, locale)?;
            let nav_tree = match nav_config {
                Some(entries) => {
                    nav::validate_for_locale(&entries, &inventory, locale);
                    nav::nav_tree_from_config_for_locale(&entries, &inventory, locale)
                }
                None => inventory.nav_tree_for_locale(locale),
            };
            let breadcrumb_map = project::build_breadcrumb_map(&nav_tree);

            // Build prev/next page map for this locale
            let flat_pages = project::flatten_nav_pages(&nav_tree);
            let mut prev_next_map: HashMap<String, (Option<PageLink>, Option<PageLink>)> =
                HashMap::new();
            for (i, (slug, _label)) in flat_pages.iter().enumerate() {
                let prev = if i > 0 {
                    let (ref ps, ref pl) = flat_pages[i - 1];
                    Some(PageLink {
                        title: pl.clone(),
                        url: format!("{}{}.html", locale_base_url, ps),
                    })
                } else {
                    None
                };
                let next = if i + 1 < flat_pages.len() {
                    let (ref ns, ref nl) = flat_pages[i + 1];
                    Some(PageLink {
                        title: nl.clone(),
                        url: format!("{}{}.html", locale_base_url, ns),
                    })
                } else {
                    None
                };
                prev_next_map.insert(slug.clone(), (prev, next));
            }

            let mut search_entries = if config.search.enabled {
                Some(Vec::new())
            } else {
                None
            };

            let locale_keys = inventory.ordered_for_locale(locale);
            for key in &locale_keys {
                let page = &inventory.pages[key];
                let source = &sources[key];
                let fm = &front_matters[key];
                let base_slug = &page.slug;

                let html_body = pipeline::process(
                    source,
                    &inventory,
                    &page.source_path,
                    &registry,
                    &root_base_url,
                    highlighter.as_ref(),
                    project_root,
                    Some(locale),
                )?;

                if let Some(ref mut entries) = search_entries {
                    let crumbs = breadcrumb_map
                        .get(base_slug)
                        .cloned()
                        .unwrap_or_else(|| vec![page.title.clone()]);
                    let mut sections = search::extract_sections(
                        &html_body,
                        base_slug,
                        &page.title,
                        &locale_base_url,
                        crumbs,
                    );
                    entries.append(&mut sections);
                }

                let nav_html = project::render_nav(&nav_tree, base_slug, &locale_base_url);

                let out_path = output_dir.join(&page.output_path);
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let (prev_page, next_page) = prev_next_map
                    .get(base_slug)
                    .cloned()
                    .unwrap_or((None, None));

                // Build available locales for the language switcher
                let site_url = config.site_url();
                let site_url_ref = site_url.as_deref();
                let available_locales = build_locale_info(
                    config,
                    base_slug,
                    locale,
                    &slug_coverage,
                    &root_base_url,
                    site_url_ref,
                );

                let canonical_url = site_url_ref.map(|site| {
                    let site = site.trim_end_matches('/');
                    let path = page.output_path.to_string_lossy().replace('\\', "/");
                    format!("{site}/{path}")
                });

                let default_locale = config.default_locale().unwrap_or("en");
                let x_default_url = available_locales
                    .iter()
                    .find(|l| l.code == default_locale)
                    .and_then(|l| l.absolute_url.clone().or(Some(l.url.clone())));

                let ctx = PageContext {
                    page_title: page.title.clone(),
                    project_name: config.project.name.clone(),
                    content: html_body,
                    nav_html,
                    default_css: theme.default_css.clone(),
                    css_overrides: theme.css_overrides.clone(),
                    custom_css_path: theme.custom_css_path.clone(),
                    custom_css: theme.custom_css.clone(),
                    base_url: root_base_url.clone(),
                    logo_path: logo_path.clone(),
                    favicon_path: favicon_path.clone(),
                    live_reload,
                    mermaid_enabled: config.charts.enabled,
                    mermaid_version: config.charts.mermaid_version.clone(),
                    search_enabled: config.search.enabled,
                    meta_description: fm.description.clone(),
                    meta_author: fm.author.clone(),
                    meta_date: fm.date.clone(),
                    prev_page,
                    next_page,
                    color_mode: config.theme.color_mode.clone(),
                    js_cachebust: js_cachebust.clone(),
                    current_locale: Some(locale.clone()),
                    current_flag: Some(config.locale_flag(locale)),
                    available_locales,
                    locale_auto_detect: config.locale.auto_detect,
                    canonical_url,
                    x_default_url,
                    search_index_url: format!("{}search-index.json", locale_base_url),
                    current_version: None,
                    available_versions: Vec::new(),
                    latest_version: None,
                    latest_version_url: None,
                };

                let html = renderer.render_page(&ctx)?;
                std::fs::write(&out_path, &html).map_err(io_context(&out_path))?;
                count += 1;
            }

            // Write per-locale search index
            if let Some(entries) = search_entries {
                let json = search::build_index(&entries);
                let path = output_dir.join(format!("{}/search-index.json", locale));
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, json).map_err(io_context(&path))?;
            }
        }

        // Emit missing translation warnings
        for (slug, locales_with_page) in &slug_coverage {
            for locale in &config.locale.enabled {
                if !locales_with_page.contains(locale) {
                    crate::diagnostics::warn_missing_translation(slug, locale);
                }
            }
        }
    } else {
        // ── Single-language build (backward compatible) ──
        let base_url = root_base_url.clone();

        let nav_config = nav::load_nav(project_root)?;
        let nav_tree = match nav_config {
            Some(entries) => {
                nav::validate(&entries, &inventory);
                nav::nav_tree_from_config(&entries, &inventory)
            }
            None => inventory.nav_tree(),
        };
        let breadcrumb_map = project::build_breadcrumb_map(&nav_tree);

        // Build prev/next page map from nav tree ordering
        let flat_pages = project::flatten_nav_pages(&nav_tree);
        let mut prev_next_map: HashMap<String, (Option<PageLink>, Option<PageLink>)> =
            HashMap::new();
        for (i, (slug, _label)) in flat_pages.iter().enumerate() {
            let prev = if i > 0 {
                let (ref ps, ref pl) = flat_pages[i - 1];
                Some(PageLink {
                    title: pl.clone(),
                    url: format!("{}{}.html", base_url, ps),
                })
            } else {
                None
            };
            let next = if i + 1 < flat_pages.len() {
                let (ref ns, ref nl) = flat_pages[i + 1];
                Some(PageLink {
                    title: nl.clone(),
                    url: format!("{}{}.html", base_url, ns),
                })
            } else {
                None
            };
            prev_next_map.insert(slug.clone(), (prev, next));
        }

        let mut search_entries = if config.search.enabled {
            Some(Vec::new())
        } else {
            None
        };

        for slug in &inventory.ordered {
            let page = &inventory.pages[slug];
            let source = &sources[slug];
            let fm = &front_matters[slug];

            let html_body = pipeline::process(
                source,
                &inventory,
                &page.source_path,
                &registry,
                &base_url,
                highlighter.as_ref(),
                project_root,
                None,
            )?;

            if let Some(ref mut entries) = search_entries {
                let crumbs = breadcrumb_map
                    .get(slug)
                    .cloned()
                    .unwrap_or_else(|| vec![page.title.clone()]);
                let mut sections =
                    search::extract_sections(&html_body, slug, &page.title, &base_url, crumbs);
                entries.append(&mut sections);
            }

            let nav_html = project::render_nav(&nav_tree, slug, &base_url);

            let out_path = output_dir.join(&page.output_path);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let (prev_page, next_page) = prev_next_map.get(slug).cloned().unwrap_or((None, None));

            let canonical_url = config.site_url().map(|site| {
                let site = site.trim_end_matches('/');
                let path = page.output_path.to_string_lossy().replace('\\', "/");
                format!("{site}/{path}")
            });

            let ctx = PageContext {
                page_title: page.title.clone(),
                project_name: config.project.name.clone(),
                content: html_body,
                nav_html,
                default_css: theme.default_css.clone(),
                css_overrides: theme.css_overrides.clone(),
                custom_css_path: theme.custom_css_path.clone(),
                custom_css: theme.custom_css.clone(),
                base_url: base_url.clone(),
                logo_path: logo_path.clone(),
                favicon_path: favicon_path.clone(),
                live_reload,
                mermaid_enabled: config.charts.enabled,
                mermaid_version: config.charts.mermaid_version.clone(),
                search_enabled: config.search.enabled,
                meta_description: fm.description.clone(),
                meta_author: fm.author.clone(),
                meta_date: fm.date.clone(),
                prev_page,
                next_page,
                color_mode: config.theme.color_mode.clone(),
                js_cachebust: js_cachebust.clone(),
                current_locale: None,
                current_flag: None,
                available_locales: Vec::new(),
                locale_auto_detect: false,
                canonical_url,
                x_default_url: None,
                search_index_url: format!("{}search-index.json", base_url),
                current_version: None,
                available_versions: Vec::new(),
                latest_version: None,
                latest_version_url: None,
            };

            let html = renderer.render_page(&ctx)?;
            std::fs::write(&out_path, &html).map_err(io_context(&out_path))?;
            count += 1;
        }

        // Write search index
        if let Some(entries) = search_entries {
            let json = search::build_index(&entries);
            let path = output_dir.join("search-index.json");
            std::fs::write(&path, json).map_err(io_context(&path))?;
        }
    }

    // Generate robots.txt and sitemap.xml for production builds
    if !live_reload {
        let site_url = config.site_url();
        if site_url.is_none() {
            crate::diagnostics::warn_no_site_url();
        }

        let sitemap_url = site_url.as_deref().map(|u| format!("{u}sitemap.xml"));
        let robots = seo::generate_robots_txt(sitemap_url.as_deref());
        let robots_path = output_dir.join("robots.txt");
        std::fs::write(&robots_path, robots).map_err(io_context(&robots_path))?;

        let locale_config = if config.is_i18n_enabled() {
            let slug_coverage = inventory.slug_locale_coverage();
            Some(seo::SitemapLocaleConfig {
                enabled: config.locale.enabled.clone(),
                default_locale: config.default_locale().unwrap_or("en").to_string(),
                slug_coverage,
            })
        } else {
            None
        };
        let sitemap = seo::generate_sitemap_xml(
            &inventory,
            &root_base_url,
            site_url.as_deref(),
            locale_config.as_ref(),
        );
        let sitemap_path = output_dir.join("sitemap.xml");
        std::fs::write(&sitemap_path, sitemap).map_err(io_context(&sitemap_path))?;
    }

    // When i18n is enabled, generate a root index.html that redirects to the default locale.
    // The versioning path handles its own redirect above (with an early return), so this only
    // runs for i18n-only builds.
    if config.is_i18n_enabled() {
        let default_locale = config.default_locale().unwrap_or("en");
        let redirect_target = format!("{}{}/index.html", root_base_url, default_locale);
        let redirect_html = format!(
            "<!DOCTYPE html>\n\
             <html>\n\
             <head>\n\
             <meta http-equiv=\"refresh\" content=\"0; url={url}\">\n\
             <link rel=\"canonical\" href=\"{url}\">\n\
             </head>\n\
             <body>\n\
             <p><a href=\"{url}\">Redirecting to documentation...</a></p>\n\
             </body>\n\
             </html>\n",
            url = redirect_target
        );
        let redirect_path = output_dir.join("index.html");
        std::fs::write(&redirect_path, redirect_html).map_err(io_context(&redirect_path))?;
    }

    // Generate 404 page
    {
        let default_locale = config.default_locale().map(String::from);
        let base_url_404 = if let Some(ref locale) = default_locale {
            format!("{}{}/", root_base_url, locale)
        } else {
            root_base_url.clone()
        };
        let nav_tree_404 = if let Some(ref locale) = default_locale {
            let nav_config = nav::load_nav_for_locale(project_root, locale)?;
            match nav_config {
                Some(entries) => nav::nav_tree_from_config_for_locale(&entries, &inventory, locale),
                None => inventory.nav_tree_for_locale(locale),
            }
        } else {
            let nav_config = nav::load_nav(project_root)?;
            match nav_config {
                Some(entries) => nav::nav_tree_from_config(&entries, &inventory),
                None => inventory.nav_tree(),
            }
        };
        let nav_html = project::render_nav(&nav_tree_404, "", &base_url_404);

        let not_found_content = if config.is_i18n_enabled() {
            let mut links = String::from(
                "<div class=\"not-found\">\
                 <h1>404</h1>\
                 <p>The page you're looking for doesn't exist.</p>\
                 <p>",
            );
            for locale in &config.locale.enabled {
                let display = config.locale_display_name(locale);
                let locale_home = format!("{}{}/index.html", root_base_url, locale);
                links.push_str(&format!("<a href=\"{}\">{}</a> ", locale_home, display));
            }
            links.push_str("</p></div>");
            links
        } else {
            format!(
                "<div class=\"not-found\">\
                 <h1>404</h1>\
                 <p>The page you're looking for doesn't exist.</p>\
                 <a href=\"{}\">Back to home</a>\
                 </div>",
                root_base_url
            )
        };

        let search_index_url_404 = if let Some(ref locale) = default_locale {
            format!("{}{}/search-index.json", root_base_url, locale)
        } else {
            format!("{}search-index.json", root_base_url)
        };
        let ctx = PageContext {
            page_title: "Page Not Found".to_string(),
            project_name: config.project.name.clone(),
            content: not_found_content,
            nav_html,
            default_css: theme.default_css.clone(),
            css_overrides: theme.css_overrides.clone(),
            custom_css_path: theme.custom_css_path.clone(),
            custom_css: theme.custom_css.clone(),
            base_url: root_base_url.clone(),
            logo_path: logo_path.clone(),
            favicon_path: favicon_path.clone(),
            live_reload,
            mermaid_enabled: false,
            mermaid_version: String::new(),
            search_enabled: config.search.enabled,
            meta_description: None,
            meta_author: None,
            meta_date: None,
            prev_page: None,
            next_page: None,
            color_mode: config.theme.color_mode.clone(),
            js_cachebust: js_cachebust.clone(),
            current_locale: default_locale,
            current_flag: None,
            available_locales: Vec::new(),
            locale_auto_detect: false,
            canonical_url: None,
            x_default_url: None,
            search_index_url: search_index_url_404,
            current_version: None,
            available_versions: Vec::new(),
            latest_version: None,
            latest_version_url: None,
        };
        let html = renderer.render_page(&ctx)?;
        let not_found_path = output_dir.join("404.html");
        std::fs::write(&not_found_path, html).map_err(io_context(&not_found_path))?;
    }

    // Copy static assets
    assets::copy_assets(project_root, output_dir, config.theme.custom_css.as_deref())?;

    Ok(count)
}

/// Scan all enabled version directories and collect the set of base slugs per version.
/// Used to build the version switcher (so we can show has_page correctly).
fn prescan_version_slugs(
    content_dir: &Path,
    config: &Config,
    enabled_locales: Option<&[String]>,
) -> Result<std::collections::HashMap<String, HashSet<String>>> {
    let mut sets = std::collections::HashMap::new();
    for version in &config.version.enabled {
        let version_dir = content_dir.join(version);
        if !version_dir.exists() {
            sets.insert(version.clone(), HashSet::new());
            continue;
        }
        // Scan without version prefix — we only need slugs, not output paths
        let inv =
            PageInventory::scan(&version_dir, enabled_locales, config.default_locale(), None)?;
        let slugs: HashSet<String> = inv.pages.values().map(|p| p.slug.clone()).collect();
        sets.insert(version.clone(), slugs);
    }
    Ok(sets)
}

/// Build version info for the version switcher on a specific page.
fn build_version_info(
    config: &Config,
    base_slug: &str,
    current_version: &str,
    locale: Option<&str>,
    version_slug_sets: &std::collections::HashMap<String, HashSet<String>>,
    root_base_url: &str,
) -> Vec<VersionInfo> {
    config
        .version
        .enabled
        .iter()
        .map(|ver| {
            let has_page = version_slug_sets
                .get(ver)
                .is_some_and(|slugs| slugs.contains(base_slug));
            let url = if has_page {
                if let Some(loc) = locale {
                    format!("{}{}/{}/{}.html", root_base_url, ver, loc, base_slug)
                } else {
                    format!("{}{}/{}.html", root_base_url, ver, base_slug)
                }
            } else if let Some(loc) = locale {
                format!("{}{}/{}/index.html", root_base_url, ver, loc)
            } else {
                format!("{}{}/index.html", root_base_url, ver)
            };
            VersionInfo {
                code: ver.clone(),
                display_name: config.version_display_name(ver),
                url,
                is_current: ver == current_version,
                has_page,
            }
        })
        .collect()
}

/// Build locale info for the language switcher on a specific page.
fn build_locale_info(
    config: &Config,
    base_slug: &str,
    current_locale: &str,
    slug_coverage: &HashMap<String, HashSet<String>>,
    root_base_url: &str,
    site_url: Option<&str>,
) -> Vec<LocaleInfo> {
    config
        .locale
        .enabled
        .iter()
        .map(|code| {
            let has_page = slug_coverage
                .get(base_slug)
                .is_some_and(|locales| locales.contains(code));
            let url = if has_page {
                format!("{}{}/{}.html", root_base_url, code, base_slug)
            } else {
                // Link to this locale's home page when the specific page doesn't exist
                format!("{}{}/index.html", root_base_url, code)
            };
            let absolute_url = site_url.map(|site| {
                let locale_path = if has_page {
                    format!("{code}/{base_slug}.html")
                } else {
                    format!("{code}/index.html")
                };
                format!("{site}{locale_path}")
            });
            LocaleInfo {
                code: code.clone(),
                display_name: config.locale_display_name(code),
                flag: config.locale_flag(code),
                url,
                absolute_url,
                is_current: code == current_locale,
                has_page,
            }
        })
        .collect()
}

/// Minify JavaScript source for production builds using oxc.
fn minify_js_source(source: &str) -> String {
    let allocator = oxc::allocator::Allocator::default();
    let source_type = oxc::span::SourceType::mjs();
    let ret = oxc::parser::Parser::new(&allocator, source, source_type).parse();
    if !ret.errors.is_empty() {
        return source.to_string();
    }
    let mut program = ret.program;
    let options = oxc::minifier::MinifierOptions {
        mangle: Some(oxc::minifier::MangleOptions::default()),
        compress: Some(oxc::minifier::CompressOptions::smallest()),
    };
    let ret = oxc::minifier::Minifier::new(options).minify(&allocator, &mut program);
    oxc::codegen::Codegen::new()
        .with_options(oxc::codegen::CodegenOptions::minify())
        .with_scoping(ret.scoping)
        .build(&program)
        .code
}
