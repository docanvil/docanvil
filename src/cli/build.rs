use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Instant;

use walkdir::WalkDir;

use crate::components::ComponentRegistry;
use crate::config::Config;
use crate::config::LastUpdatedSource;
use crate::diagnostics::{self, reset_warnings, warning_count};
use crate::edit::{EditLinks, Editor, RepoLink};
use crate::error::{Error, Result};
use crate::last_updated::{self, DateSource, GitDates, NoDates, Override};
use crate::llms::{self, LlmsScope};
use crate::nav;
use crate::pipeline;
use crate::pipeline::frontmatter::{self, FrontMatter};
use crate::pipeline::markdown;
use crate::pipeline::syntax::SyntaxHighlighter;
use crate::project::{self, PageInfo, PageInventory};
use crate::redirects::{self, PageSet};
use crate::render::assets;
use crate::render::templates::{
    LocaleInfo, PageContext, PageLink, TemplateRenderer, VersionInfo, page_breadcrumbs,
    with_page_description,
};
use crate::search;
use crate::seo;
use crate::theme::Theme;

/// Wrap an IO error with the file path that caused it.
fn io_context(path: &Path) -> impl FnOnce(std::io::Error) -> Error + '_ {
    move |e| Error::General(format!("{}: {e}", path.display()))
}

/// Run the build command from CLI.
///
/// `out` is the `--out` flag: when given it wins, otherwise `[build] output_dir`
/// from the config is used (relative to the project root).
pub fn run(
    project_root: &Path,
    out: Option<&Path>,
    clean: bool,
    quiet: bool,
    strict: bool,
    drafts: bool,
) -> Result<()> {
    let start = Instant::now();
    let config = Config::load(project_root)?;

    // Resolve output directory
    let output_dir = match out {
        Some(out) => out.to_path_buf(),
        None => project_root.join(&config.build.output_dir),
    };

    // Clean output directory if requested
    if clean && output_dir.exists() {
        ensure_safe_to_remove(project_root, &config, &output_dir)?;
        std::fs::remove_dir_all(&output_dir)?;
    }

    reset_warnings();
    crate::pipeline::popovers::reset_popover_ids();

    let mut dependencies = BTreeSet::new();
    let mode = BuildMode {
        live_reload: false,
        drafts,
        editor: None,
    };
    let built = build_into(project_root, &config, &output_dir, mode, &mut dependencies)?;

    if strict && warning_count() > 0 {
        return Err(Error::StrictWarnings(warning_count()));
    }

    if !quiet {
        let elapsed = start.elapsed();
        eprintln!(
            "Built {} page{} in {:.0?}",
            built.pages,
            plural(built.pages),
            elapsed
        );
        if built.drafts_skipped > 0 {
            eprintln!(
                "Skipped {} draft page{} (include them with --drafts)",
                built.drafts_skipped,
                plural(built.drafts_skipped)
            );
        }
    }

    Ok(())
}

/// How a build treats the dev server's extras and draft pages.
#[derive(Debug, Clone, Copy)]
struct BuildMode<'a> {
    /// Building for `docanvil serve`: inject live reload, use `/` as the base URL.
    live_reload: bool,
    /// Include pages marked `"draft": true` (always on for `docanvil serve`).
    drafts: bool,
    /// Link each page to its source in this local editor instead of the Git host.
    editor: Option<&'a Editor>,
}

/// What a build produced.
struct Built {
    pages: usize,
    drafts_skipped: usize,
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// Unless drafts are included, move pages marked `"draft": true` out of the
/// inventory so nothing in the build sees them. Returns how many were left out.
pub(crate) fn exclude_drafts(
    inventory: &mut PageInventory,
    front_matters: &HashMap<String, FrontMatter>,
    config: &Config,
    include_drafts: bool,
) -> usize {
    if include_drafts {
        return 0;
    }
    let keys: Vec<String> = front_matters
        .iter()
        .filter(|(_, fm)| fm.draft)
        .map(|(key, _)| key.clone())
        .collect();
    inventory.exclude_drafts(&keys, config.build.draft_links);
    keys.len()
}

/// Read every page's source and extract its front matter. Sets each page's title
/// (front matter, then the first `# H1`) and applies front matter slugs to the
/// inventory. Returns sources and front matter keyed like `inventory.pages`.
pub(crate) fn read_sources(
    inventory: &mut PageInventory,
) -> Result<(HashMap<String, String>, HashMap<String, FrontMatter>)> {
    let mut sources: HashMap<String, String> = HashMap::new();
    let mut front_matters: HashMap<String, FrontMatter> = HashMap::new();
    let mut slug_updates: Vec<(String, String)> = Vec::new();

    for slug in &inventory.ordered {
        let page = &inventory.pages[slug];
        let source =
            std::fs::read_to_string(&page.source_path).map_err(io_context(&page.source_path))?;
        let fm = frontmatter::extract(&source);
        // Title: front matter, then the first `# H1`, then the filename (from scan).
        // Only front matter titles change the slug, so editing a heading never moves a URL.
        if let Some(title) = fm
            .title
            .clone()
            .or_else(|| markdown::first_h1_text(&source))
            && let Some(page) = inventory.pages.get_mut(slug)
        {
            page.title = title;
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

        // Only update if the slug actually changes (compare against filename portion)
        if let Some(new_slug) = new_slug
            && new_slug != current_basename
        {
            slug_updates.push((slug.clone(), new_slug));
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

    Ok((sources, front_matters))
}

/// One version's pages (version `None` without versioning): inventory and front matter.
pub(crate) type SiteVersion = (Option<String>, PageInventory, HashMap<String, FrontMatter>);

/// Scan the site the way `docanvil build` sees it: one inventory per version (or
/// one for the whole site), with front matter slugs applied and drafts left out.
/// Used by `docanvil doctor` to check what a build would do.
pub(crate) fn scan_site(project_root: &Path, config: &Config) -> Result<Vec<SiteVersion>> {
    let content_dir = project_root.join(&config.project.content_dir);
    let enabled_locales = config
        .is_i18n_enabled()
        .then_some(config.locale.enabled.as_slice());
    let versions: Vec<Option<String>> = if config.is_versioning_enabled() {
        config.version.enabled.iter().cloned().map(Some).collect()
    } else {
        vec![None]
    };

    let mut sites = Vec::new();
    for version in versions {
        let dir = match &version {
            Some(v) => content_dir.join(v),
            None => content_dir.clone(),
        };
        if !dir.is_dir() {
            continue;
        }
        let mut inventory = PageInventory::scan(
            &dir,
            enabled_locales,
            config.default_locale(),
            version.as_deref(),
        )?;
        let (_, front_matters) = read_sources(&mut inventory)?;
        exclude_drafts(&mut inventory, &front_matters, config, false);
        sites.push((version, inventory, front_matters));
    }
    Ok(sites)
}

/// Refuse to delete an output directory that would take project files with it.
///
/// Guards against a mistyped `--out` or `[build] output_dir` (e.g. `.` or `..`)
/// wiping the project root, its content or theme, or another DocAnvil project.
pub(crate) fn ensure_safe_to_remove(
    project_root: &Path,
    config: &Config,
    output_dir: &Path,
) -> Result<()> {
    let unsafe_dir = |reason: &str| Error::UnsafeOutputDir {
        path: output_dir.to_path_buf(),
        reason: reason.to_string(),
    };

    // Canonicalize so `.`, `..`, symlinks and relative paths compare correctly.
    let output = output_dir.canonicalize()?;
    let root = project_root.canonicalize()?;

    if root.starts_with(&output) {
        return Err(unsafe_dir("it contains the project root"));
    }

    let protected = [
        (config.project.content_dir.as_path(), "content directory"),
        (Path::new("theme"), "theme directory"),
        (Path::new("assets"), "assets directory"),
        (Path::new("static"), "static directory"),
    ];
    for (dir, name) in protected {
        if let Ok(dir) = root.join(dir).canonicalize()
            && dir.starts_with(&output)
        {
            return Err(unsafe_dir(&format!("it contains the project's {name}")));
        }
    }

    if output.join("docanvil.toml").exists() {
        return Err(unsafe_dir(
            "it looks like a DocAnvil project (has docanvil.toml)",
        ));
    }

    Ok(())
}

/// Build into `output_dir`, optionally with live reload (used by the dev server).
/// With an `editor`, each page's edit link opens its source in that editor.
/// Returns every file the build read through `:::include` and `file="…"` code
/// blocks, so the dev server can watch the ones outside the project's folders.
pub fn run_with_options(
    project_root: &Path,
    output_dir: &Path,
    live_reload: bool,
    editor: Option<&Editor>,
) -> Result<BTreeSet<PathBuf>> {
    let config = Config::load(project_root)?;

    reset_warnings();
    crate::pipeline::popovers::reset_popover_ids();

    let mut dependencies = BTreeSet::new();
    // The dev server always shows drafts, so authors can see them as they write.
    let mode = BuildMode {
        live_reload,
        drafts: true,
        editor,
    };
    let built = build_into(project_root, &config, output_dir, mode, &mut dependencies)?;
    eprintln!("Built {} page{}", built.pages, plural(built.pages));
    Ok(dependencies)
}

/// Build the site into a staging directory, then sync it into `output_dir`.
///
/// Files in `output_dir` that the build didn't produce (e.g. pages that were
/// renamed or deleted) are removed, so the output always matches the sources.
/// Hidden top-level entries such as `.git` (a gh-pages worktree) or
/// `.well-known` are left alone. If the build fails, `output_dir` is untouched.
fn build_into(
    project_root: &Path,
    config: &Config,
    output_dir: &Path,
    mode: BuildMode<'_>,
    dependencies: &mut BTreeSet<PathBuf>,
) -> Result<Built> {
    if output_dir.exists() {
        ensure_safe_to_remove(project_root, config, output_dir)?;
    }

    let name = output_dir
        .canonicalize()
        .unwrap_or_else(|_| output_dir.to_path_buf())
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "dist".to_string());
    let staging = output_dir.with_file_name(format!(".{name}.docanvil-staging"));
    if staging.exists() {
        std::fs::remove_dir_all(&staging).map_err(io_context(&staging))?;
    }

    let built = match build_site(project_root, config, &staging, mode, dependencies) {
        Ok(built) => built,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(e);
        }
    };

    sync_output(&staging, output_dir)?;
    std::fs::remove_dir_all(&staging).map_err(io_context(&staging))?;
    Ok(built)
}

/// Move everything from `staging` into `output_dir`, then delete whatever in
/// `output_dir` wasn't in `staging` (skipping hidden top-level entries).
fn sync_output(staging: &Path, output_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(output_dir).map_err(io_context(output_dir))?;

    let mut fresh: HashSet<PathBuf> = HashSet::new();
    for entry in WalkDir::new(staging).min_depth(1) {
        let entry = entry.map_err(std::io::Error::from)?;
        let rel = entry.path().strip_prefix(staging).unwrap_or(entry.path());
        let dest = output_dir.join(rel);
        fresh.insert(rel.to_path_buf());

        if entry.file_type().is_dir() {
            if dest.is_file() || dest.is_symlink() {
                std::fs::remove_file(&dest).map_err(io_context(&dest))?;
            }
            std::fs::create_dir_all(&dest).map_err(io_context(&dest))?;
        } else {
            if dest.is_dir() && !dest.is_symlink() {
                std::fs::remove_dir_all(&dest).map_err(io_context(&dest))?;
            }
            // rename is cheap and atomic; fall back to copying when the output
            // directory is on another filesystem (e.g. a mounted volume).
            if std::fs::rename(entry.path(), &dest).is_err() {
                std::fs::copy(entry.path(), &dest).map_err(io_context(&dest))?;
            }
        }
    }

    for top in std::fs::read_dir(output_dir).map_err(io_context(output_dir))? {
        let top = top?;
        if top.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        // Children come before their parent, so stale directories are empty by
        // the time we reach them.
        for entry in WalkDir::new(top.path()).contents_first(true) {
            let entry = entry.map_err(std::io::Error::from)?;
            let rel = entry
                .path()
                .strip_prefix(output_dir)
                .unwrap_or(entry.path());
            if fresh.contains(rel) {
                continue;
            }
            if entry.file_type().is_dir() {
                std::fs::remove_dir(entry.path()).map_err(io_context(entry.path()))?;
            } else {
                std::fs::remove_file(entry.path()).map_err(io_context(entry.path()))?;
            }
        }
    }

    Ok(())
}

/// The page's edit link, unless its front matter opts out: its source in the
/// local editor under `docanvil serve`, otherwise on the Git host.
fn page_edit_url(
    edit_links: Option<&EditLinks>,
    editor: Option<&Editor>,
    page: &PageInfo,
    fm: &FrontMatter,
) -> Option<String> {
    if fm.edit_link == Some(false) {
        return None;
    }
    match editor {
        Some(editor) => editor.url_for(&page.source_path),
        None => edit_links?.url_for(&page.source_path),
    }
}

/// The source of "last updated" dates for this build, or `None` when the feature is off.
fn last_updated_source(config: &Config, project_root: &Path) -> Option<Box<dyn DateSource>> {
    if !config.last_updated.enabled {
        return None;
    }
    match config.last_updated.source {
        LastUpdatedSource::FrontMatter => Some(Box::new(NoDates)),
        LastUpdatedSource::Git => match GitDates::collect(project_root) {
            Ok(git) => {
                if git.is_shallow() {
                    diagnostics::warn_last_updated_shallow();
                }
                Some(Box::new(git))
            }
            Err(message) => {
                diagnostics::warn_last_updated_no_git(&message);
                Some(Box::new(NoDates))
            }
        },
    }
}

/// A page's "last updated" date: its front matter override, else the newest
/// date across its source file and the files it pulls in.
fn page_last_updated(
    dates: &mut Option<Box<dyn DateSource>>,
    page: &PageInfo,
    fm: &FrontMatter,
    deps: &BTreeSet<PathBuf>,
) -> Option<String> {
    let source = dates.as_mut()?;
    match fm.last_updated.as_ref().map(last_updated::parse_override) {
        Some(Override::Hide) => None,
        Some(Override::Date(date)) => Some(date.to_string()),
        None | Some(Override::Invalid) => {
            last_updated::page_date(source.as_mut(), &page.source_path, deps)
                .map(|date| date.to_string())
        }
    }
}

/// Work out this build's redirects, warn about any problems, and write the stubs.
fn write_redirects(
    project_root: &Path,
    config: &Config,
    sets: &[PageSet],
    output_dir: &Path,
    base_url: &str,
) -> Result<()> {
    let plan = redirects::plan(config, sets);
    for problem in &plan.problems {
        diagnostics::warn_redirect(project_root, problem);
    }
    redirects::write_stubs(
        output_dir,
        &plan.redirects,
        base_url,
        config.site_url().as_deref(),
    )
}

/// Write each scope's `llms.txt` (and `llms-full.txt`) into its folder. The
/// `root_dir` scope is also copied to the site root with an `## Optional`
/// section linking every other scope, since AI tools only look at `/llms.txt`.
fn write_llms(
    output_dir: &Path,
    config: &Config,
    scopes: &[LlmsScope],
    root_dir: &str,
    base: &str,
) -> Result<()> {
    for scope in scopes {
        let sections = llms::sections_from_nav(&scope.nav, &scope.pages);
        write_llms_pair(&output_dir.join(&scope.dir), config, &sections, &[])?;
        if !scope.dir.is_empty() && scope.dir == root_dir {
            let optional: Vec<(String, String)> = scopes
                .iter()
                .filter(|other| other.dir != root_dir)
                .map(|other| (other.label.clone(), format!("{base}{}llms.txt", other.dir)))
                .collect();
            write_llms_pair(output_dir, config, &sections, &optional)?;
        }
    }
    Ok(())
}

/// Write `llms.txt`, plus `llms-full.txt` unless `[llms] full = false`.
fn write_llms_pair(
    dir: &Path,
    config: &Config,
    sections: &[llms::Section],
    optional: &[(String, String)],
) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let name = &config.project.name;
    let description = config.llms.description.as_deref();
    let index_path = dir.join("llms.txt");
    std::fs::write(
        &index_path,
        llms::generate_index(name, description, sections, optional),
    )
    .map_err(io_context(&index_path))?;
    if config.llms.full {
        let full_path = dir.join("llms-full.txt");
        std::fs::write(&full_path, llms::generate_full(name, description, sections))
            .map_err(io_context(&full_path))?;
    }
    Ok(())
}

/// Core build logic shared between CLI and serve.
fn build_site(
    project_root: &Path,
    config: &Config,
    output_dir: &Path,
    mode: BuildMode<'_>,
    dependencies: &mut BTreeSet<PathBuf>,
) -> Result<Built> {
    let BuildMode {
        live_reload,
        drafts: include_drafts,
        editor,
    } = mode;
    let content_dir = project_root.join(&config.project.content_dir);
    if !content_dir.exists() {
        return Err(Error::ContentDirNotFound(content_dir));
    }

    // Resolve theme and create template renderer
    let theme = Theme::resolve(config, project_root);
    let renderer = TemplateRenderer::new(&theme)?;

    let edit_links = match EditLinks::from_config(&config.edit, project_root) {
        Ok(links) => links,
        Err(message) => {
            diagnostics::warn_edit_link_config(&message);
            None
        }
    };

    let repo_link = match RepoLink::from_config(config.project.repo.as_deref()) {
        Ok(link) => link,
        Err(message) => {
            diagnostics::warn_repo_link_config(&message);
            None
        }
    };

    let mut last_updated_dates = last_updated_source(config, project_root);
    // Output path → date, for <lastmod> in the sitemap
    let mut lastmod: HashMap<PathBuf, String> = HashMap::new();

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
    let (sources, front_matters) = read_sources(&mut inventory)?;

    // Versioned builds scan each version below and count their drafts there.
    let unversioned_drafts = exclude_drafts(&mut inventory, &front_matters, config, include_drafts);
    let mut drafts_skipped = if config.is_versioning_enabled() {
        0
    } else {
        unversioned_drafts
    };

    let registry = ComponentRegistry::load(project_root)?;

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
    // llms.txt: one scope per (version, locale), written after all pages render.
    let llms_on = config.llms.enabled && !live_reload;
    let llms_base = config.site_url().unwrap_or_else(|| root_base_url.clone());
    let mut llms_scopes: Vec<LlmsScope> = Vec::new();
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

        // Pre-scan all version directories to know which pages each version publishes.
        // This powers the version switcher's has_page flag without full re-scans later.
        let version_pages =
            prescan_version_pages(&content_dir, config, enabled_locales, include_drafts)?;
        let latest_version = config.current_version().map(String::from);
        let current_ver_str = config.current_version().unwrap_or("").to_string();

        // Each version's pages and front matter, for the sitemap and redirects.
        let mut version_sets: Vec<(String, PageInventory, HashMap<String, FrontMatter>)> =
            Vec::new();
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
            let (ver_sources, ver_front_matters) = read_sources(&mut ver_inventory)?;

            drafts_skipped += exclude_drafts(
                &mut ver_inventory,
                &ver_front_matters,
                config,
                include_drafts,
            );

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

                    let mut llms_pages = llms_on.then(Vec::new);
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

                        let processed = pipeline::process(
                            source,
                            &ver_inventory,
                            &page.source_path,
                            &registry,
                            &root_base_url,
                            highlighter.as_ref(),
                            config.syntax.line_numbers,
                            project_root,
                            Some(locale),
                        )?;
                        if let Some(pages) = llms_pages.as_mut().filter(|_| fm.llms != Some(false))
                        {
                            pages.push(llms::page_entry(
                                page,
                                fm,
                                &processed.markdown,
                                &ver_inventory,
                                Some(locale),
                                &llms_base,
                                project_root,
                            ));
                        }
                        let last_updated = page_last_updated(
                            &mut last_updated_dates,
                            page,
                            fm,
                            &processed.dependencies,
                        );
                        if let Some(date) = &last_updated {
                            lastmod.insert(page.output_path.clone(), date.clone());
                        }
                        dependencies.extend(processed.dependencies);
                        let html_body = processed.html;

                        if let Some(ref mut entries) = search_entries {
                            let crumbs = breadcrumb_map
                                .get(base_slug)
                                .map(|trail| project::crumb_labels(trail))
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

                        let breadcrumbs =
                            page_breadcrumbs(breadcrumb_map.get(base_slug), &locale_base_url);

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
                            &version_pages,
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
                            content: with_page_description(html_body, fm.description.as_deref()),
                            nav_html,
                            default_css: theme.default_css.clone(),
                            css_overrides: theme.css_overrides.clone(),
                            custom_css_path: theme.custom_css_path.clone(),
                            custom_css: theme.custom_css.clone(),
                            base_url: root_base_url.clone(),
                            logo_path: logo_path.clone(),
                            repo: repo_link.clone(),
                            favicon_path: favicon_path.clone(),
                            live_reload,
                            mermaid_enabled: config.charts.enabled,
                            mermaid_version: config.charts.mermaid_version.clone(),
                            search_enabled: config.search.enabled,
                            meta_description: fm.description.clone(),
                            edit_url: page_edit_url(edit_links.as_ref(), editor, page, fm),
                            edit_local: editor.is_some(),
                            breadcrumbs,
                            last_updated,
                            draft: fm.draft,
                            toc: fm.toc != Some(false),
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

                    if let Some(pages) = llms_pages {
                        llms_scopes.push(LlmsScope {
                            dir: format!("{version}/{locale}/"),
                            label: format!(
                                "{} · {}",
                                config.version_display_name(version),
                                config.locale_display_name(locale)
                            ),
                            nav: nav_tree.clone(),
                            pages,
                        });
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
                        // A draft translation isn't missing, just unpublished.
                        if !locales_with_page.contains(locale)
                            && !ver_inventory
                                .drafts
                                .contains_key(&format!("{locale}:{slug}"))
                        {
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

                let mut llms_pages = llms_on.then(Vec::new);
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

                    let processed = pipeline::process(
                        source,
                        &ver_inventory,
                        &page.source_path,
                        &registry,
                        &root_base_url,
                        highlighter.as_ref(),
                        config.syntax.line_numbers,
                        project_root,
                        None,
                    )?;
                    if let Some(pages) = llms_pages.as_mut().filter(|_| fm.llms != Some(false)) {
                        pages.push(llms::page_entry(
                            page,
                            fm,
                            &processed.markdown,
                            &ver_inventory,
                            None,
                            &llms_base,
                            project_root,
                        ));
                    }
                    let last_updated = page_last_updated(
                        &mut last_updated_dates,
                        page,
                        fm,
                        &processed.dependencies,
                    );
                    if let Some(date) = &last_updated {
                        lastmod.insert(page.output_path.clone(), date.clone());
                    }
                    dependencies.extend(processed.dependencies);
                    let html_body = processed.html;

                    if let Some(ref mut entries) = search_entries {
                        let crumbs = breadcrumb_map
                            .get(slug)
                            .map(|trail| project::crumb_labels(trail))
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

                    let breadcrumbs =
                        page_breadcrumbs(breadcrumb_map.get(base_slug), &version_base_url);

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
                        &version_pages,
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
                        content: with_page_description(html_body, fm.description.as_deref()),
                        nav_html,
                        default_css: theme.default_css.clone(),
                        css_overrides: theme.css_overrides.clone(),
                        custom_css_path: theme.custom_css_path.clone(),
                        custom_css: theme.custom_css.clone(),
                        base_url: root_base_url.clone(),
                        logo_path: logo_path.clone(),
                        repo: repo_link.clone(),
                        favicon_path: favicon_path.clone(),
                        live_reload,
                        mermaid_enabled: config.charts.enabled,
                        mermaid_version: config.charts.mermaid_version.clone(),
                        search_enabled: config.search.enabled,
                        meta_description: fm.description.clone(),
                        edit_url: page_edit_url(edit_links.as_ref(), editor, page, fm),
                        edit_local: editor.is_some(),
                        breadcrumbs,
                        last_updated,
                        draft: fm.draft,
                        toc: fm.toc != Some(false),
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

                if let Some(pages) = llms_pages {
                    llms_scopes.push(LlmsScope {
                        dir: format!("{version}/"),
                        label: config.version_display_name(version),
                        nav: nav_tree.clone(),
                        pages,
                    });
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

            version_sets.push((version.clone(), ver_inventory, ver_front_matters));
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
        let redirect_path = output_dir.join("index.html");
        std::fs::write(
            &redirect_path,
            redirects::stub_html(&redirect_target, &redirect_target, None),
        )
        .map_err(io_context(&redirect_path))?;

        let sets: Vec<PageSet> = version_sets
            .iter()
            .map(|(version, inventory, front_matters)| PageSet {
                version: Some(version),
                inventory,
                front_matters,
            })
            .collect();
        write_redirects(project_root, config, &sets, output_dir, &root_base_url)?;
        let llms_root = match config.default_locale().filter(|_| config.is_i18n_enabled()) {
            Some(locale) => format!("{current_ver_str}/{locale}/"),
            None => format!("{current_ver_str}/"),
        };
        write_llms(output_dir, config, &llms_scopes, &llms_root, &llms_base)?;

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
            for (_, inv, _) in &version_sets {
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
                drafts: HashMap::new(),
                draft_links: Default::default(),
            };
            let sitemap = seo::generate_sitemap_xml(
                &merged_inv,
                &root_base_url,
                site_url.as_deref(),
                None,
                &lastmod,
            );
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
                repo: repo_link.clone(),
                favicon_path: favicon_path.clone(),
                live_reload,
                mermaid_enabled: false,
                mermaid_version: String::new(),
                search_enabled: config.search.enabled,
                meta_description: None,
                edit_url: None,
                edit_local: false,
                breadcrumbs: Vec::new(),
                last_updated: None,
                draft: false,
                toc: true,
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

        return Ok(Built {
            pages: count,
            drafts_skipped,
        });
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

            let mut llms_pages = llms_on.then(Vec::new);
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

                let processed = pipeline::process(
                    source,
                    &inventory,
                    &page.source_path,
                    &registry,
                    &root_base_url,
                    highlighter.as_ref(),
                    config.syntax.line_numbers,
                    project_root,
                    Some(locale),
                )?;
                if let Some(pages) = llms_pages.as_mut().filter(|_| fm.llms != Some(false)) {
                    pages.push(llms::page_entry(
                        page,
                        fm,
                        &processed.markdown,
                        &inventory,
                        Some(locale),
                        &llms_base,
                        project_root,
                    ));
                }
                let last_updated =
                    page_last_updated(&mut last_updated_dates, page, fm, &processed.dependencies);
                if let Some(date) = &last_updated {
                    lastmod.insert(page.output_path.clone(), date.clone());
                }
                dependencies.extend(processed.dependencies);
                let html_body = processed.html;

                if let Some(ref mut entries) = search_entries {
                    let crumbs = breadcrumb_map
                        .get(base_slug)
                        .map(|trail| project::crumb_labels(trail))
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

                let breadcrumbs = page_breadcrumbs(breadcrumb_map.get(base_slug), &locale_base_url);

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
                    content: with_page_description(html_body, fm.description.as_deref()),
                    nav_html,
                    default_css: theme.default_css.clone(),
                    css_overrides: theme.css_overrides.clone(),
                    custom_css_path: theme.custom_css_path.clone(),
                    custom_css: theme.custom_css.clone(),
                    base_url: root_base_url.clone(),
                    logo_path: logo_path.clone(),
                    repo: repo_link.clone(),
                    favicon_path: favicon_path.clone(),
                    live_reload,
                    mermaid_enabled: config.charts.enabled,
                    mermaid_version: config.charts.mermaid_version.clone(),
                    search_enabled: config.search.enabled,
                    meta_description: fm.description.clone(),
                    edit_url: page_edit_url(edit_links.as_ref(), editor, page, fm),
                    edit_local: editor.is_some(),
                    breadcrumbs,
                    last_updated,
                    draft: fm.draft,
                    toc: fm.toc != Some(false),
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

            if let Some(pages) = llms_pages {
                llms_scopes.push(LlmsScope {
                    dir: format!("{locale}/"),
                    label: config.locale_display_name(locale),
                    nav: nav_tree.clone(),
                    pages,
                });
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
                // A draft translation isn't missing, just unpublished.
                if !locales_with_page.contains(locale)
                    && !inventory.drafts.contains_key(&format!("{locale}:{slug}"))
                {
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

        let mut llms_pages = llms_on.then(Vec::new);
        let mut search_entries = if config.search.enabled {
            Some(Vec::new())
        } else {
            None
        };

        for slug in &inventory.ordered {
            let page = &inventory.pages[slug];
            let source = &sources[slug];
            let fm = &front_matters[slug];

            let processed = pipeline::process(
                source,
                &inventory,
                &page.source_path,
                &registry,
                &base_url,
                highlighter.as_ref(),
                config.syntax.line_numbers,
                project_root,
                None,
            )?;
            if let Some(pages) = llms_pages.as_mut().filter(|_| fm.llms != Some(false)) {
                pages.push(llms::page_entry(
                    page,
                    fm,
                    &processed.markdown,
                    &inventory,
                    None,
                    &llms_base,
                    project_root,
                ));
            }
            let last_updated =
                page_last_updated(&mut last_updated_dates, page, fm, &processed.dependencies);
            if let Some(date) = &last_updated {
                lastmod.insert(page.output_path.clone(), date.clone());
            }
            dependencies.extend(processed.dependencies);
            let html_body = processed.html;

            if let Some(ref mut entries) = search_entries {
                let crumbs = breadcrumb_map
                    .get(slug)
                    .map(|trail| project::crumb_labels(trail))
                    .unwrap_or_else(|| vec![page.title.clone()]);
                let mut sections =
                    search::extract_sections(&html_body, slug, &page.title, &base_url, crumbs);
                entries.append(&mut sections);
            }

            let nav_html = project::render_nav(&nav_tree, slug, &base_url);

            let breadcrumbs = page_breadcrumbs(breadcrumb_map.get(slug), &base_url);

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
                content: with_page_description(html_body, fm.description.as_deref()),
                nav_html,
                default_css: theme.default_css.clone(),
                css_overrides: theme.css_overrides.clone(),
                custom_css_path: theme.custom_css_path.clone(),
                custom_css: theme.custom_css.clone(),
                base_url: base_url.clone(),
                logo_path: logo_path.clone(),
                repo: repo_link.clone(),
                favicon_path: favicon_path.clone(),
                live_reload,
                mermaid_enabled: config.charts.enabled,
                mermaid_version: config.charts.mermaid_version.clone(),
                search_enabled: config.search.enabled,
                meta_description: fm.description.clone(),
                edit_url: page_edit_url(edit_links.as_ref(), editor, page, fm),
                edit_local: editor.is_some(),
                breadcrumbs,
                last_updated,
                draft: fm.draft,
                toc: fm.toc != Some(false),
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

        if let Some(pages) = llms_pages {
            llms_scopes.push(LlmsScope {
                dir: String::new(),
                label: config.project.name.clone(),
                nav: nav_tree.clone(),
                pages,
            });
        }

        // Write search index
        if let Some(entries) = search_entries {
            let json = search::build_index(&entries);
            let path = output_dir.join("search-index.json");
            std::fs::write(&path, json).map_err(io_context(&path))?;
        }
    }

    let llms_root = match config.default_locale().filter(|_| config.is_i18n_enabled()) {
        Some(locale) => format!("{locale}/"),
        None => String::new(),
    };
    write_llms(output_dir, config, &llms_scopes, &llms_root, &llms_base)?;

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
            &lastmod,
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
        let redirect_path = output_dir.join("index.html");
        std::fs::write(
            &redirect_path,
            redirects::stub_html(&redirect_target, &redirect_target, None),
        )
        .map_err(io_context(&redirect_path))?;
    }

    let sets = [PageSet {
        version: None,
        inventory: &inventory,
        front_matters: &front_matters,
    }];
    write_redirects(project_root, config, &sets, output_dir, &root_base_url)?;

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
            repo: repo_link.clone(),
            favicon_path: favicon_path.clone(),
            live_reload,
            mermaid_enabled: false,
            mermaid_version: String::new(),
            search_enabled: config.search.enabled,
            meta_description: None,
            edit_url: None,
            edit_local: false,
            breadcrumbs: Vec::new(),
            last_updated: None,
            draft: false,
            toc: true,
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

    Ok(Built {
        pages: count,
        drafts_skipped,
    })
}

/// The pages one version publishes, as `(locale, slug)` pairs (`locale` is `None`
/// without i18n). Powers the version switcher's `has_page` flag.
type VersionPages = HashSet<(Option<String>, String)>;

/// Scan all enabled version directories and collect the pages each one publishes.
/// Scans the way the build does, so front matter slugs apply and drafts are left
/// out unless `include_drafts`: the switcher only links to pages that exist.
fn prescan_version_pages(
    content_dir: &Path,
    config: &Config,
    enabled_locales: Option<&[String]>,
    include_drafts: bool,
) -> Result<HashMap<String, VersionPages>> {
    let mut sets = HashMap::new();
    for version in &config.version.enabled {
        let version_dir = content_dir.join(version);
        if !version_dir.exists() {
            sets.insert(version.clone(), HashSet::new());
            continue;
        }
        // Scan without version prefix — we only need slugs, not output paths
        let mut inv =
            PageInventory::scan(&version_dir, enabled_locales, config.default_locale(), None)?;
        let (_, front_matters) = read_sources(&mut inv)?;
        exclude_drafts(&mut inv, &front_matters, config, include_drafts);
        let pages = inv
            .pages
            .into_values()
            .map(|page| (page.locale, page.slug))
            .collect();
        sets.insert(version.clone(), pages);
    }
    Ok(sets)
}

/// Build version info for the version switcher on a specific page.
fn build_version_info(
    config: &Config,
    base_slug: &str,
    current_version: &str,
    locale: Option<&str>,
    version_pages: &HashMap<String, VersionPages>,
    root_base_url: &str,
) -> Vec<VersionInfo> {
    let key = (locale.map(String::from), base_slug.to_string());
    config
        .version
        .enabled
        .iter()
        .map(|ver| {
            // The page must exist in this locale, or the link would 404
            let has_page = version_pages
                .get(ver)
                .is_some_and(|pages| pages.contains(&key));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(root: &Path, files: &[(&str, &str)]) {
        for (path, content) in files {
            let p = root.join(path);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, content).unwrap();
        }
    }

    #[test]
    fn scan_site_matches_the_build_view() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            &[
                (
                    "docanvil.toml",
                    "[project]\nname = \"T\"\n\n[version]\nenabled = [\"v1\", \"v2\"]\n",
                ),
                ("docs/v1/index.md", "# One"),
                ("docs/v2/index.md", "# Two"),
                (
                    "docs/v2/setup.md",
                    "---\n{\"slug\": \"install\"}\n---\n# Setup",
                ),
                ("docs/v2/wip.md", "---\n{\"draft\": true}\n---\n# WIP"),
            ],
        );
        let config = Config::load(dir.path()).unwrap();
        let sites = scan_site(dir.path(), &config).unwrap();

        assert_eq!(sites.len(), 2);
        let (version, inventory, front_matters) = &sites[1];
        assert_eq!(version.as_deref(), Some("v2"));
        assert!(inventory.pages.contains_key("install"));
        assert_eq!(
            inventory.pages["install"].output_path,
            PathBuf::from("v2/install.html")
        );
        assert!(!inventory.pages.contains_key("wip"));
        assert!(inventory.drafts.contains_key("wip"));
        assert!(front_matters.contains_key("install"));
    }

    fn project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("docanvil.toml"),
            "[project]\nname = \"T\"\n",
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        fs::create_dir_all(dir.path().join("theme")).unwrap();
        fs::create_dir_all(dir.path().join("dist")).unwrap();
        dir
    }

    fn is_unsafe(result: Result<()>) -> bool {
        matches!(result, Err(Error::UnsafeOutputDir { .. }))
    }

    #[test]
    fn safe_to_remove_regular_output_dir() {
        let dir = project();
        let config = Config::default();
        assert!(ensure_safe_to_remove(dir.path(), &config, &dir.path().join("dist")).is_ok());
    }

    #[test]
    fn refuses_project_root() {
        let dir = project();
        let config = Config::default();
        assert!(is_unsafe(ensure_safe_to_remove(
            dir.path(),
            &config,
            dir.path()
        )));
        // Relative spellings resolve to the same directory.
        assert!(is_unsafe(ensure_safe_to_remove(
            dir.path(),
            &config,
            &dir.path().join("dist/..")
        )));
    }

    #[test]
    fn refuses_parent_of_project_root() {
        let dir = project();
        let nested = dir.path().join("site");
        fs::create_dir_all(nested.join("docs")).unwrap();
        let config = Config::default();
        assert!(is_unsafe(ensure_safe_to_remove(
            &nested,
            &config,
            dir.path()
        )));
    }

    #[test]
    fn refuses_content_and_theme_dirs() {
        let dir = project();
        let config = Config::default();
        for sub in ["docs", "theme"] {
            assert!(is_unsafe(ensure_safe_to_remove(
                dir.path(),
                &config,
                &dir.path().join(sub)
            )));
        }
    }

    #[test]
    fn refuses_custom_content_dir() {
        let dir = project();
        fs::create_dir_all(dir.path().join("content")).unwrap();
        let mut config = Config::default();
        config.project.content_dir = "content".into();
        assert!(is_unsafe(ensure_safe_to_remove(
            dir.path(),
            &config,
            &dir.path().join("content")
        )));
    }

    #[test]
    fn refuses_another_docanvil_project() {
        let dir = project();
        let other = tempfile::tempdir().unwrap();
        fs::write(other.path().join("docanvil.toml"), "").unwrap();
        let config = Config::default();
        assert!(is_unsafe(ensure_safe_to_remove(
            dir.path(),
            &config,
            other.path()
        )));
    }
}
