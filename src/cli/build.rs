use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
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
use crate::render::templates::{PageContext, PageLink, TemplateRenderer};
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
    let mut inventory = PageInventory::scan(&content_dir)?;

    // Pre-pass: read all sources and extract front matter.
    // Override page titles from front matter before nav/search are built.
    let mut sources: HashMap<String, String> = HashMap::new();
    let mut front_matters: HashMap<String, FrontMatter> = HashMap::new();
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
        sources.insert(slug.clone(), source);
        front_matters.insert(slug.clone(), fm);
    }

    let nav_config = nav::load_nav(project_root)?;
    let nav_tree = match nav_config {
        Some(entries) => {
            nav::validate(&entries, &inventory);
            nav::nav_tree_from_config(&entries, &inventory)
        }
        None => inventory.nav_tree(),
    };
    let breadcrumb_map = project::build_breadcrumb_map(&nav_tree);
    let registry = ComponentRegistry::with_builtins();

    // Create syntax highlighter if enabled
    let highlighter = if config.syntax.enabled {
        Some(SyntaxHighlighter::new(&config.syntax.theme))
    } else {
        None
    };

    // Dev server always uses "/" — base_url only applies to static builds
    let base_url = if live_reload {
        "/".to_string()
    } else {
        config.base_url()
    };
    // Compute logo and favicon paths with base_url prefix
    let logo_path = config
        .project
        .logo
        .as_ref()
        .map(|p| format!("{}{}", base_url, p));
    let favicon_path = config
        .project
        .favicon
        .as_ref()
        .map(|p| format!("{}{}", base_url, p));

    // Write JS file to output directory
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

    // Build prev/next page map from nav tree ordering
    let flat_pages = project::flatten_nav_pages(&nav_tree);
    let mut prev_next_map: HashMap<String, (Option<PageLink>, Option<PageLink>)> = HashMap::new();
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

    let mut count = 0;
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

        let sitemap = seo::generate_sitemap_xml(&inventory, &base_url, site_url.as_deref());
        let sitemap_path = output_dir.join("sitemap.xml");
        std::fs::write(&sitemap_path, sitemap).map_err(io_context(&sitemap_path))?;
    }

    // Generate 404 page
    {
        let nav_html = project::render_nav(&nav_tree, "", &base_url);
        let not_found_content = format!(
            "<div class=\"not-found\">\
             <h1>404</h1>\
             <p>The page you're looking for doesn't exist.</p>\
             <a href=\"{}\">Back to home</a>\
             </div>",
            base_url
        );
        let ctx = PageContext {
            page_title: "Page Not Found".to_string(),
            project_name: config.project.name.clone(),
            content: not_found_content,
            nav_html,
            default_css: theme.default_css.clone(),
            css_overrides: theme.css_overrides.clone(),
            custom_css_path: theme.custom_css_path.clone(),
            custom_css: theme.custom_css.clone(),
            base_url: base_url.clone(),
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
        };
        let html = renderer.render_page(&ctx)?;
        let not_found_path = output_dir.join("404.html");
        std::fs::write(&not_found_path, html).map_err(io_context(&not_found_path))?;
    }

    // Copy static assets
    assets::copy_assets(project_root, output_dir, config.theme.custom_css.as_deref())?;

    Ok(count)
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
