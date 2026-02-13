use std::path::Path;
use std::time::Instant;

use crate::components::ComponentRegistry;
use crate::config::Config;
use crate::diagnostics::{reset_warnings, warning_count};
use crate::error::{Error, Result};
use crate::nav;
use crate::pipeline;
use crate::pipeline::syntax::SyntaxHighlighter;
use crate::project::{self, PageInventory};
use crate::render::assets;
use crate::render::templates::{PageContext, TemplateRenderer};
use crate::search;
use crate::seo;
use crate::theme::Theme;

/// Run the build command from CLI.
pub fn run(out: &Path, clean: bool, quiet: bool, strict: bool) -> Result<()> {
    let start = Instant::now();

    let project_root = Path::new(".");
    let config = Config::load(project_root)?;

    // Resolve output directory
    let output_dir = if out != Path::new("dist") {
        out.to_path_buf()
    } else {
        config.build.output_dir.clone()
    };

    // Clean output directory if requested
    if clean && output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    reset_warnings();

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
pub fn run_with_options(out: &Path, live_reload: bool) -> Result<()> {
    let project_root = Path::new(".");
    let config = Config::load(project_root)?;
    let output_dir = config.build.output_dir.clone();
    let _ = out; // Use config output dir for serve mode

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
    let inventory = PageInventory::scan(&content_dir)?;
    let nav_config = nav::load_nav(project_root)?;
    let nav_tree = match nav_config {
        Some(entries) => {
            nav::validate(&entries, &inventory);
            nav::nav_tree_from_config(&entries, &inventory)
        }
        None => inventory.nav_tree(),
    };
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

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir)?;

    let mut search_entries = if config.search.enabled {
        Some(Vec::new())
    } else {
        None
    };

    let mut count = 0;
    for slug in &inventory.ordered {
        let page = &inventory.pages[slug];
        let source = std::fs::read_to_string(&page.source_path)?;

        let html_body = pipeline::process(
            &source,
            &inventory,
            &page.source_path,
            &registry,
            &base_url,
            highlighter.as_ref(),
            project_root,
        )?;

        if let Some(ref mut entries) = search_entries {
            let mut sections = search::extract_sections(&html_body, slug, &page.title, &base_url);
            entries.append(&mut sections);
        }

        let nav_html = project::render_nav(&nav_tree, slug, &base_url);

        let out_path = output_dir.join(&page.output_path);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

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
        };

        let html = renderer.render_page(&ctx)?;
        std::fs::write(&out_path, html)?;
        count += 1;
    }

    // Write search index
    if let Some(entries) = search_entries {
        let json = search::build_index(&entries);
        std::fs::write(output_dir.join("search-index.json"), json)?;
    }

    // Generate robots.txt and sitemap.xml for production builds
    if !live_reload {
        let site_url = config.site_url();
        if site_url.is_none() {
            crate::diagnostics::warn_no_site_url();
        }

        let sitemap_url = site_url.as_deref().map(|u| format!("{u}sitemap.xml"));
        let robots = seo::generate_robots_txt(sitemap_url.as_deref());
        std::fs::write(output_dir.join("robots.txt"), robots)?;

        let sitemap = seo::generate_sitemap_xml(&inventory, &base_url, site_url.as_deref());
        std::fs::write(output_dir.join("sitemap.xml"), sitemap)?;
    }

    // Copy static assets
    assets::copy_assets(project_root, output_dir, config.theme.custom_css.as_deref())?;

    Ok(count)
}
