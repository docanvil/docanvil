use std::collections::HashMap;
use std::path::{Path, PathBuf};

use base64::Engine;
use regex::Regex;

use serde::Serialize;
use tera::{Context, Tera};

use crate::components::ComponentRegistry;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::nav;
use crate::pipeline;
use crate::pipeline::frontmatter::{self, FrontMatter};
use crate::pipeline::syntax::SyntaxHighlighter;
use crate::project::{NavNode, PageInfo, PageInventory, flatten_nav_pages};

use super::cdp;

/// The PDF Tera template, embedded at compile time.
const PDF_TEMPLATE: &str = include_str!("../../theme/default/pdf.html");

/// Data for a single chapter in the PDF.
#[derive(Serialize)]
struct ChapterData {
    slug: String,
    title: String,
    content_html: String,
}

/// Context passed to the PDF Tera template.
#[derive(Serialize)]
struct PdfContext {
    project_title: String,
    toc_html: String,
    chapters: Vec<ChapterData>,
    show_cover: bool,
    pdf_author: Option<String>,
    /// Base64 data URI for the project logo, shown on the cover page when set.
    /// Encoded here rather than passed as a file path because the template is
    /// rendered from a temp file, so site-relative URLs and relative paths both
    /// fail to resolve.  All common raster and SVG formats are supported.
    cover_logo_data_uri: Option<String>,
    custom_css: Option<String>,
    mermaid_enabled: bool,
    mermaid_version: String,
    lang: String,
    is_rtl: bool,
    paper_size_css: String,
    /// CSS custom-property declarations injected into the template's `:root`.
    /// Derived from `[theme].variables` in `docanvil.toml`, with defaults that
    /// match the values in `style.css` so the PDF looks coherent out of the box.
    theme_css_vars: String,
}

/// Read the project logo from disk and return it as a base64 data URI.
///
/// Returns `None` if no logo is configured, the file cannot be read, or the
/// file extension is not a recognised image type.
fn logo_to_data_uri(project_root: &Path, logo: &str) -> Option<String> {
    let path = project_root.join(logo);
    let mime = match path.extension().and_then(|e| e.to_str()) {
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => return None,
    };
    let bytes = std::fs::read(&path).ok()?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{mime};base64,{encoded}"))
}

/// Build the CSS variable declarations block from the merged theme variable map.
///
/// The 12 known variables are always emitted (falling back to the defaults from
/// `style.css` if the user has not set them).  Any additional variables in the map
/// are emitted after — sorted alphabetically — so that values defined in a custom
/// CSS file and referenced by `[pdf].custom_css` are available in `:root`.
fn build_theme_css_vars(variables: &HashMap<String, String>) -> String {
    const DEFAULTS: &[(&str, &str)] = &[
        ("--color-primary", "#6366f1"),
        ("--color-primary-light", "#818cf8"),
        ("--color-bg", "#ffffff"),
        ("--color-bg-secondary", "#f8fafc"),
        ("--color-text", "#1e293b"),
        ("--color-text-muted", "#64748b"),
        ("--color-border", "#e2e8f0"),
        ("--color-code-bg", "#f1f5f9"),
        ("--color-note-bg", "#eef2ff"),
        ("--color-note-border", "#818cf8"),
        ("--color-warning-bg", "#fff7ed"),
        ("--color-warning-border", "#f97316"),
    ];

    let mut lines: Vec<String> = DEFAULTS
        .iter()
        .map(|(key, default)| {
            let val = variables.get(*key).map(String::as_str).unwrap_or(default);
            format!("      {key}: {val};")
        })
        .collect();

    // Emit any additional variables not covered by the defaults above.
    let known: std::collections::HashSet<&str> = DEFAULTS.iter().map(|(k, _)| *k).collect();
    let mut extras: Vec<(&String, &String)> = variables
        .iter()
        .filter(|(k, _)| !known.contains(k.as_str()))
        .collect();
    extras.sort_by_key(|(k, _)| k.as_str());
    for (key, val) in extras {
        lines.push(format!("      {key}: {val};"));
    }

    lines.join("\n")
}

/// Extract CSS custom-property declarations from a CSS string.
///
/// Only declarations inside `:root { … }` blocks are extracted, as those are
/// the ones that set global custom properties (the same scope the web renderer
/// uses when it applies the user's theme overrides).
fn extract_css_vars_from_file(css: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    // Match :root { … } blocks.  [^}]* handles newlines in Rust's regex crate
    // because character-class negations match any character except the listed one,
    // including newlines.  This won't handle pathological cases where a custom
    // property value itself contains `}`, but that is not a valid CSS value for
    // the colour variables we care about.
    let root_re = Regex::new(r":root\s*\{([^}]*)\}").expect("valid regex");
    let var_re = Regex::new(r"(--[\w-]+)\s*:\s*([^;]+);").expect("valid regex");

    for root_cap in root_re.captures_iter(css) {
        for var_cap in var_re.captures_iter(&root_cap[1]) {
            let name = var_cap[1].trim().to_string();
            let value = var_cap[2].trim().to_string();
            vars.insert(name, value);
        }
    }

    vars
}

/// Detect Chrome or Chromium on this platform, returning the path if found.
fn find_chrome() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        // Standard Google Chrome app bundle.
        let p = PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome");
        if p.is_file() {
            return Some(p);
        }
        // Chromium app bundle — `brew install --cask chromium` or a direct download.
        let p = PathBuf::from("/Applications/Chromium.app/Contents/MacOS/Chromium");
        if p.is_file() {
            return Some(p);
        }
        // Fall through to the generic PATH loop below.
    }

    #[cfg(target_os = "windows")]
    {
        let pf = std::env::var("ProgramFiles").unwrap_or_default();
        let pf86 = std::env::var("ProgramFiles(x86)").unwrap_or_default();
        let local = std::env::var("LOCALAPPDATA").unwrap_or_default();

        // Checked in priority order:
        //   1. System-wide Chrome  (ProgramFiles / ProgramFiles(x86))
        //   2. User-level Chrome   (%LOCALAPPDATA% — no admin rights required; common on corporate machines)
        //   3. User-level Chromium (%LOCALAPPDATA% — Chromium mini-installer)
        //   4. Microsoft Edge      (pre-installed on Windows 10 post-2021 and Windows 11)
        let checks: &[(&str, &[&str])] = &[
            (
                pf.as_str(),
                &["Google", "Chrome", "Application", "chrome.exe"],
            ),
            (
                pf86.as_str(),
                &["Google", "Chrome", "Application", "chrome.exe"],
            ),
            (
                local.as_str(),
                &["Google", "Chrome", "Application", "chrome.exe"],
            ),
            (local.as_str(), &["Chromium", "Application", "chrome.exe"]),
            (
                pf.as_str(),
                &["Microsoft", "Edge", "Application", "msedge.exe"],
            ),
            (
                pf86.as_str(),
                &["Microsoft", "Edge", "Application", "msedge.exe"],
            ),
        ];
        for &(base, parts) in checks {
            if base.is_empty() {
                continue;
            }
            let path = parts.iter().fold(PathBuf::from(base), |p, s| p.join(s));
            if path.is_file() {
                return Some(path);
            }
        }
        // Fall through to the generic PATH loop below (handles Scoop, Chocolatey, etc.).
    }

    // Linux and PATH fallback for all platforms.
    for name in &[
        "google-chrome",
        "google-chrome-stable",
        "chromium-browser",
        "chromium",
        "chrome",
    ] {
        if let Some(p) = which_in_path(name) {
            return Some(p);
        }
    }

    None
}

/// Search PATH for an executable by name and return its full path if found.
///
/// On Windows, also tries `name.exe` because raw `is_file()` checks bypass the
/// OS `PATHEXT` mechanism that shells use to resolve extension-less commands.
fn which_in_path(name: &str) -> Option<PathBuf> {
    let path_env = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_env) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(target_os = "windows")]
        {
            let candidate_exe = dir.join(format!("{name}.exe"));
            if candidate_exe.is_file() {
                return Some(candidate_exe);
            }
        }
    }
    None
}

/// Render the nav tree as a nested `<ol>` TOC with anchor links.
fn render_toc(nodes: &[NavNode]) -> String {
    let mut html = String::from("<ol>\n");
    render_toc_nodes(nodes, &mut html, 1);
    html.push_str("</ol>\n");
    html
}

fn render_toc_nodes(nodes: &[NavNode], html: &mut String, depth: usize) {
    let indent = "  ".repeat(depth);
    for node in nodes {
        match node {
            NavNode::Page { label, slug } => {
                html.push_str(&format!(
                    "{indent}<li><a href=\"#{slug}\">{label}</a></li>\n",
                    label = crate::util::html_escape(label),
                ));
            }
            NavNode::Group {
                label,
                slug,
                children,
            } => {
                if let Some(s) = slug {
                    html.push_str(&format!(
                        "{indent}<li><a href=\"#{s}\">{label}</a>\n",
                        label = crate::util::html_escape(label),
                    ));
                } else {
                    html.push_str(&format!(
                        "{indent}<li>{label}\n",
                        label = crate::util::html_escape(label),
                    ));
                }
                if !children.is_empty() {
                    html.push_str(&format!("{indent}  <ol>\n"));
                    render_toc_nodes(children, html, depth + 2);
                    html.push_str(&format!("{indent}  </ol>\n"));
                }
                html.push_str(&format!("{indent}</li>\n"));
            }
            NavNode::Separator { label } => {
                if let Some(text) = label {
                    html.push_str(&format!(
                        "{indent}<li class=\"toc-separator\">{text}</li>\n",
                        text = crate::util::html_escape(text),
                    ));
                }
            }
        }
    }
}

/// Assemble the final PDF HTML by rendering the Tera template with the given context.
fn assemble_pdf_html(ctx: &PdfContext) -> Result<String> {
    let mut tera = Tera::default();
    tera.add_raw_template("pdf.html", PDF_TEMPLATE)
        .map_err(|e| Error::Render(format!("failed to parse PDF template: {e}")))?;
    let context = Context::from_serialize(ctx)
        .map_err(|e| Error::Render(format!("failed to serialize PDF context: {e}")))?;
    tera.render("pdf.html", &context)
        .map_err(|e| Error::Render(format!("PDF template render error: {e}")))
}

/// Rewrite wiki-link hrefs from site-relative URLs to in-document anchor fragments.
///
/// The normal pipeline resolves `[[page]]` to `href="/page.html"`, which is
/// correct for the multi-file HTML site.  In the PDF, all chapters live in a
/// single HTML document whose sections are identified by `id="<slug>"`, so the
/// links need to become `href="#<slug>"` instead.
///
/// Only links whose href exactly matches a known page's output path (filtered to
/// the current export locale) are rewritten; external links are left untouched.
fn rewrite_links_for_pdf(html: &str, inventory: &PageInventory, locale: Option<&str>) -> String {
    let mut result = html.to_owned();
    for page in inventory.pages.values() {
        if locale.is_some() && page.locale.as_deref() != locale {
            continue;
        }
        let src = format!("href=\"/{}\"", page.output_path.display());
        let dest = format!("href=\"#{}\"", page.slug);
        if result.contains(&src) {
            result = result.replace(&src, &dest);
        }
    }
    result
}

/// Wrap an IO error with the file path that caused it.
fn io_context(path: &Path) -> impl FnOnce(std::io::Error) -> Error + '_ {
    move |e| Error::General(format!("{}: {e}", path.display()))
}

/// Insert `locale` before the file extension of `base`.
///
/// `guide.pdf` + `"en"` → `guide.en.pdf`
/// `out/guide.pdf` + `"fr"` → `out/guide.fr.pdf`
/// `guide` (no extension) + `"de"` → `guide.de`
fn locale_output_path(base: &Path, locale: &str) -> PathBuf {
    let stem = base.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    let new_name = match base.extension().and_then(|e| e.to_str()) {
        Some(ext) => format!("{stem}.{locale}.{ext}"),
        None => format!("{stem}.{locale}"),
    };
    match base.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(new_name),
        _ => PathBuf::from(new_name),
    }
}

/// Run the PDF export for a single concrete locale (or no locale when i18n is off).
fn run_single_locale(
    project_root: &Path,
    out: &Path,
    locale_arg: Option<&str>,
    config: &Config,
    quiet: bool,
) -> Result<()> {
    let content_dir = project_root.join(&config.project.content_dir);
    if !content_dir.exists() {
        return Err(Error::ContentDirNotFound(content_dir));
    }

    let enabled_locales = if config.is_i18n_enabled() {
        Some(config.locale.enabled.as_slice())
    } else {
        None
    };
    if !quiet {
        eprintln!("Scanning pages…");
    }
    let mut inventory =
        PageInventory::scan(&content_dir, enabled_locales, config.default_locale(), None)?;

    // Determine the locale to export (None when i18n is disabled).
    let export_locale: Option<&str> = if config.is_i18n_enabled() {
        Some(locale_arg.unwrap_or_else(|| config.default_locale().unwrap_or("en")))
    } else {
        None
    };

    // ── Pre-pass: read sources and extract front matter ──────────────────────
    let page_keys: Vec<String> = if let Some(locale) = export_locale {
        inventory.ordered_for_locale(locale)
    } else {
        inventory.ordered.clone()
    };

    let mut sources: HashMap<String, String> = HashMap::new();
    let mut front_matters: HashMap<String, FrontMatter> = HashMap::new();
    let mut slug_updates: Vec<(String, String)> = Vec::new();

    for key in &page_keys {
        // Extract owned values before any mutable borrow of inventory.
        let (source_path, slug) = {
            let page = &inventory.pages[key];
            (page.source_path.clone(), page.slug.clone())
        };

        let source = std::fs::read_to_string(&source_path).map_err(io_context(&source_path))?;
        let fm = frontmatter::extract(&source);

        if let Some(ref title) = fm.title
            && let Some(p) = inventory.pages.get_mut(key)
        {
            p.title = title.clone();
        }

        let current_basename = slug.rsplit('/').next().unwrap_or(&slug).to_string();
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
            slug_updates.push((key.clone(), new_slug));
        }

        sources.insert(key.clone(), source);
        front_matters.insert(key.clone(), fm);
    }

    // Apply slug updates after the loop to avoid mutating while iterating.
    for (old_key, new_slug) in slug_updates {
        if let Some(source) = sources.remove(&old_key) {
            let fm = front_matters.remove(&old_key).unwrap_or_default();
            inventory.update_slug(&old_key, new_slug);
            let full_new_key = inventory
                .slug_aliases
                .get(&old_key)
                .cloned()
                .unwrap_or(old_key);
            sources.insert(full_new_key.clone(), source);
            front_matters.insert(full_new_key, fm);
        }
    }

    // ── Build nav tree (after slug updates) ───────────────────────────────────
    let nav_tree = if let Some(locale) = export_locale {
        let nav_config = nav::load_nav_for_locale(project_root, locale)?;
        match nav_config {
            Some(entries) => {
                nav::validate_for_locale(&entries, &inventory, locale);
                nav::nav_tree_from_config_for_locale(&entries, &inventory, locale)
            }
            None => inventory.nav_tree_for_locale(locale),
        }
    } else {
        let nav_config = nav::load_nav(project_root)?;
        match nav_config {
            Some(entries) => {
                nav::validate(&entries, &inventory);
                nav::nav_tree_from_config(&entries, &inventory)
            }
            None => inventory.nav_tree(),
        }
    };

    let flat_pages = flatten_nav_pages(&nav_tree);

    // ── Render each page through the pipeline ────────────────────────────────
    if !quiet {
        let n = flat_pages.len();
        eprintln!("Rendering {} page{}…", n, if n == 1 { "" } else { "s" });
    }
    let registry = ComponentRegistry::with_builtins();
    let highlighter = if config.syntax.enabled {
        Some(SyntaxHighlighter::new(&config.syntax.theme))
    } else {
        None
    };

    let mut chapters: Vec<ChapterData> = Vec::new();

    for (slug, _label) in &flat_pages {
        let key = if let Some(locale) = export_locale {
            format!("{}:{}", locale, slug)
        } else {
            slug.clone()
        };

        let Some(page) = inventory.pages.get(&key) else {
            continue; // Page in nav but not in inventory — skip silently
        };
        let Some(source) = sources.get(&key) else {
            continue;
        };

        let html_body = pipeline::process(
            source,
            &inventory,
            &page.source_path,
            &registry,
            "/",
            highlighter.as_ref(),
            project_root,
            export_locale,
        )?;
        let html_body = rewrite_links_for_pdf(&html_body, &inventory, export_locale);

        chapters.push(ChapterData {
            slug: slug.clone(),
            title: page.title.clone(),
            content_html: html_body,
        });
    }

    // ── Load optional custom PDF CSS ──────────────────────────────────────────
    let custom_css: Option<String> = if let Some(ref css_path) = config.pdf.custom_css {
        let css_file = project_root.join(css_path);
        match std::fs::read_to_string(&css_file) {
            Ok(content) => Some(content),
            Err(e) => {
                if !quiet {
                    eprintln!(
                        "Warning: could not read PDF custom CSS at {}: {e}",
                        css_file.display()
                    );
                }
                None
            }
        }
    } else {
        None
    };

    // ── Locale / RTL context ──────────────────────────────────────────────────
    let (lang, is_rtl) = if let Some(locale) = export_locale {
        (locale.to_string(), crate::config::is_rtl_locale(locale))
    } else {
        ("en".to_string(), false)
    };
    let paper_size_css = config
        .pdf
        .paper_size
        .clone()
        .unwrap_or_else(|| "A4".to_string());

    // ── Theme integration ─────────────────────────────────────────────────────
    // Priority (lowest → highest): style.css defaults → [theme].variables →
    // [theme].custom_css :root overrides.  This mirrors the layering order that
    // the web renderer uses so the PDF automatically reflects any colour changes
    // the user has made, with no extra PDF-specific configuration required.
    let mut merged_theme_vars = config.theme.variables.clone();
    if let Some(ref css_path) = config.theme.custom_css {
        let css_file = project_root.join(css_path);
        match std::fs::read_to_string(&css_file) {
            Ok(css) => {
                merged_theme_vars.extend(extract_css_vars_from_file(&css));
            }
            Err(e) => {
                if !quiet {
                    eprintln!(
                        "Warning: could not read theme custom CSS at {}: {e}",
                        css_file.display()
                    );
                }
            }
        }
    }
    let theme_css_vars = build_theme_css_vars(&merged_theme_vars);
    let accent_color = merged_theme_vars.get("--color-primary").map(String::as_str);

    // ── Assemble PDF HTML ─────────────────────────────────────────────────────
    let cover_logo_data_uri = config
        .project
        .logo
        .as_deref()
        .and_then(|logo| logo_to_data_uri(project_root, logo));

    let toc_html = render_toc(&nav_tree);
    let ctx = PdfContext {
        project_title: config.project.name.clone(),
        toc_html,
        chapters,
        show_cover: config.pdf.cover_page,
        pdf_author: config.pdf.author.clone(),
        cover_logo_data_uri,
        custom_css,
        mermaid_enabled: config.charts.enabled,
        mermaid_version: config.charts.mermaid_version.clone(),
        lang,
        is_rtl,
        paper_size_css,
        theme_css_vars,
    };
    let html = assemble_pdf_html(&ctx)?;

    // Write HTML to a temp file
    let tmp_path = std::env::temp_dir().join(format!("docanvil-pdf-{}.html", std::process::id()));
    std::fs::write(&tmp_path, &html)
        .map_err(|e| Error::General(format!("failed to write temporary HTML: {e}")))?;

    // Find Chrome
    if !quiet {
        eprintln!("Looking for Chrome or Chromium…");
    }
    let chrome = find_chrome().ok_or(Error::ChromeNotFound)?;
    if !quiet {
        eprintln!("  Found: {}", chrome.display());
    }

    // Ensure output directory exists
    if let Some(parent) = out.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    // Generate the PDF via CDP
    let pdf_result = cdp::render_to_pdf_cdp(
        &chrome,
        &tmp_path,
        out,
        cdp::PdfRenderOptions {
            project_title: &config.project.name,
            pdf_author: config.pdf.author.as_deref(),
            wait_mermaid: config.charts.enabled,
            paper_size: config.pdf.paper_size.as_deref(),
            accent_color,
            quiet,
        },
    );

    // Always clean up the temp file, even on error
    let _ = std::fs::remove_file(&tmp_path);

    pdf_result?;

    if !quiet {
        eprintln!("PDF written to {}", out.display());
    }

    Ok(())
}

/// Run the `export pdf` command.
pub fn run(project_root: &Path, out: &Path, locale_arg: Option<&str>, quiet: bool) -> Result<()> {
    let config = Config::load(project_root)?;

    if locale_arg == Some("all") {
        if !config.is_i18n_enabled() {
            return Err(Error::General(
                "--locale all requires i18n to be configured \
                 (set [locale] default and enabled in docanvil.toml)"
                    .into(),
            ));
        }
        for locale in &config.locale.enabled {
            let locale_out = locale_output_path(out, locale);
            if !quiet {
                eprintln!("Exporting PDF for locale '{locale}'…");
            }
            run_single_locale(project_root, &locale_out, Some(locale), &config, quiet)?;
        }
        return Ok(());
    }

    run_single_locale(project_root, out, locale_arg, &config, quiet)
}

/// Retrieve a page by its inventory key, used in tests.
#[allow(dead_code)]
fn lookup_page<'a>(
    inventory: &'a PageInventory,
    slug: &str,
    locale: Option<&str>,
) -> Option<&'a PageInfo> {
    let key = if let Some(loc) = locale {
        format!("{}:{}", loc, slug)
    } else {
        slug.to_string()
    };
    inventory.pages.get(&key)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── extract_css_vars_from_file ────────────────────────────────────────────

    #[test]
    fn extract_css_vars_simple_root_block() {
        let css = ":root {\n  --color-primary: #e63946;\n  --color-bg: #f1faee;\n}";
        let vars = extract_css_vars_from_file(css);
        assert_eq!(
            vars.get("--color-primary").map(String::as_str),
            Some("#e63946")
        );
        assert_eq!(vars.get("--color-bg").map(String::as_str), Some("#f1faee"));
    }

    #[test]
    fn extract_css_vars_multiple_root_blocks() {
        let css = ":root { --color-primary: #abc; }\n\n:root { --color-bg: #fff; }";
        let vars = extract_css_vars_from_file(css);
        assert_eq!(
            vars.get("--color-primary").map(String::as_str),
            Some("#abc")
        );
        assert_eq!(vars.get("--color-bg").map(String::as_str), Some("#fff"));
    }

    #[test]
    fn extract_css_vars_ignores_outside_root() {
        let css =
            "--color-primary: #bad;\nbody { --color-text: #333; }\n:root { --color-bg: #fff; }";
        let vars = extract_css_vars_from_file(css);
        // Only the :root block variable is captured
        assert!(vars.get("--color-primary").is_none());
        assert!(vars.get("--color-text").is_none());
        assert_eq!(vars.get("--color-bg").map(String::as_str), Some("#fff"));
    }

    #[test]
    fn extract_css_vars_trims_whitespace() {
        let css = ":root {\n  --color-primary :   #abc123  ;\n}";
        let vars = extract_css_vars_from_file(css);
        assert_eq!(
            vars.get("--color-primary").map(String::as_str),
            Some("#abc123")
        );
    }

    #[test]
    fn extract_css_vars_complex_value() {
        let css = ":root { --color-primary: rgba(99, 102, 241, 0.9); }";
        let vars = extract_css_vars_from_file(css);
        assert_eq!(
            vars.get("--color-primary").map(String::as_str),
            Some("rgba(99, 102, 241, 0.9)")
        );
    }

    #[test]
    fn extract_css_vars_empty_css() {
        let vars = extract_css_vars_from_file("body { color: red; }");
        assert!(vars.is_empty());
    }

    // ── build_theme_css_vars ──────────────────────────────────────────────────

    #[test]
    fn build_theme_css_vars_uses_defaults_when_empty() {
        let vars = HashMap::new();
        let out = build_theme_css_vars(&vars);
        assert!(out.contains("--color-primary: #6366f1;"));
        assert!(out.contains("--color-bg: #ffffff;"));
    }

    #[test]
    fn build_theme_css_vars_overrides_with_user_value() {
        let mut vars = HashMap::new();
        vars.insert("--color-primary".to_string(), "#e63946".to_string());
        let out = build_theme_css_vars(&vars);
        assert!(out.contains("--color-primary: #e63946;"));
        assert!(!out.contains("--color-primary: #6366f1;"));
        // Other defaults unchanged
        assert!(out.contains("--color-bg: #ffffff;"));
    }

    #[test]
    fn build_theme_css_vars_emits_extra_vars() {
        let mut vars = HashMap::new();
        vars.insert("--my-custom-brand".to_string(), "#aabbcc".to_string());
        let out = build_theme_css_vars(&vars);
        assert!(out.contains("--my-custom-brand: #aabbcc;"));
        // Known defaults still present
        assert!(out.contains("--color-primary: #6366f1;"));
    }

    #[test]
    fn build_theme_css_vars_merge_priority() {
        // Custom CSS vars should win over TOML vars when merged before calling.
        let mut merged = HashMap::new();
        merged.insert("--color-primary".to_string(), "#toml-value".to_string());
        // Simulate the CSS file override winning
        merged.insert("--color-primary".to_string(), "#css-file-value".to_string());
        let out = build_theme_css_vars(&merged);
        assert!(out.contains("--color-primary: #css-file-value;"));
        assert!(!out.contains("#toml-value"));
    }

    #[test]
    fn render_toc_flat_pages() {
        let nodes = vec![
            NavNode::Page {
                label: "Home".into(),
                slug: "index".into(),
            },
            NavNode::Page {
                label: "Getting Started".into(),
                slug: "getting-started".into(),
            },
        ];
        let toc = render_toc(&nodes);
        assert!(toc.contains("<ol>"));
        assert!(toc.contains("href=\"#index\""));
        assert!(toc.contains("Home"));
        assert!(toc.contains("href=\"#getting-started\""));
        assert!(toc.contains("Getting Started"));
    }

    #[test]
    fn render_toc_with_group() {
        let nodes = vec![NavNode::Group {
            label: "Guides".into(),
            slug: None,
            children: vec![NavNode::Page {
                label: "Setup".into(),
                slug: "guides/setup".into(),
            }],
        }];
        let toc = render_toc(&nodes);
        assert!(toc.contains("Guides"));
        assert!(toc.contains("href=\"#guides/setup\""));
        assert!(toc.contains("Setup"));
        // Group without slug should not produce a link for itself
        assert!(!toc.contains("href=\"#\""));
    }

    #[test]
    fn render_toc_with_separator() {
        let nodes = vec![
            NavNode::Separator {
                label: Some("Section One".into()),
            },
            NavNode::Page {
                label: "Page".into(),
                slug: "page".into(),
            },
        ];
        let toc = render_toc(&nodes);
        assert!(toc.contains("toc-separator"));
        assert!(toc.contains("Section One"));
        assert!(toc.contains("href=\"#page\""));
    }

    #[test]
    fn render_toc_escapes_html_in_labels() {
        let nodes = vec![NavNode::Page {
            label: "A & B <test>".into(),
            slug: "a-b".into(),
        }];
        let toc = render_toc(&nodes);
        assert!(toc.contains("A &amp; B &lt;test&gt;"));
        assert!(!toc.contains("A & B <test>"));
    }

    fn default_ctx() -> PdfContext {
        PdfContext {
            project_title: "My Docs".into(),
            toc_html: "<ol></ol>".into(),
            chapters: vec![],
            show_cover: false,
            pdf_author: None,
            cover_logo_data_uri: None,
            custom_css: None,
            mermaid_enabled: false,
            mermaid_version: "11".into(),
            lang: "en".into(),
            is_rtl: false,
            paper_size_css: "A4".into(),
            theme_css_vars: String::new(),
        }
    }

    #[test]
    fn assemble_pdf_html_renders_template() {
        let ctx = PdfContext {
            project_title: "My Docs".into(),
            toc_html: "<ol><li>Page One</li></ol>".into(),
            chapters: vec![ChapterData {
                slug: "intro".into(),
                title: "Introduction".into(),
                content_html: "<h1>Introduction</h1><p>Hello world.</p>".into(),
            }],
            ..default_ctx()
        };
        let html = assemble_pdf_html(&ctx).unwrap();
        assert!(html.contains("My Docs"));
        assert!(html.contains("Page One"));
        assert!(html.contains("id=\"intro\""));
        assert!(html.contains("Hello world."));
        assert!(html.contains("@page"));
    }

    #[test]
    fn assemble_pdf_html_cover_page() {
        let ctx = PdfContext {
            project_title: "Cover Test".into(),
            show_cover: true,
            pdf_author: Some("Jane Doe".into()),
            ..default_ctx()
        };
        let html = assemble_pdf_html(&ctx).unwrap();
        assert!(html.contains("pdf-cover"));
        assert!(html.contains("Cover Test"));
        assert!(html.contains("Jane Doe"));
    }

    #[test]
    fn assemble_pdf_html_cover_logo_rendered() {
        let ctx = PdfContext {
            show_cover: true,
            cover_logo_data_uri: Some("data:image/png;base64,abc123".into()),
            ..default_ctx()
        };
        let html = assemble_pdf_html(&ctx).unwrap();
        assert!(html.contains("cover-logo"));
        assert!(html.contains("data:image/png;base64,abc123"));
    }

    #[test]
    fn assemble_pdf_html_no_logo_when_no_cover() {
        // Logo data URI present but cover disabled — img should not appear.
        // Note: we check for the data URI itself, not the class name, because
        // the CSS always defines the .cover-logo rule regardless of whether the
        // cover section is rendered.
        let ctx = PdfContext {
            show_cover: false,
            cover_logo_data_uri: Some("data:image/png;base64,abc123".into()),
            ..default_ctx()
        };
        let html = assemble_pdf_html(&ctx).unwrap();
        assert!(!html.contains("data:image/png;base64,abc123"));
    }

    #[test]
    fn logo_to_data_uri_png() {
        let dir = tempfile::tempdir().unwrap();
        let logo_path = dir.path().join("logo.png");
        // Minimal 1×1 PNG (valid header so MIME resolves correctly)
        std::fs::write(&logo_path, b"\x89PNG\r\n\x1a\n").unwrap();
        let uri = logo_to_data_uri(dir.path(), "logo.png").unwrap();
        assert!(uri.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn logo_to_data_uri_svg() {
        let dir = tempfile::tempdir().unwrap();
        let logo_path = dir.path().join("logo.svg");
        std::fs::write(&logo_path, b"<svg></svg>").unwrap();
        let uri = logo_to_data_uri(dir.path(), "logo.svg").unwrap();
        assert!(uri.starts_with("data:image/svg+xml;base64,"));
    }

    #[test]
    fn logo_to_data_uri_missing_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(logo_to_data_uri(dir.path(), "missing.png").is_none());
    }

    #[test]
    fn logo_to_data_uri_unknown_extension_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("logo.bmp"), b"BM").unwrap();
        assert!(logo_to_data_uri(dir.path(), "logo.bmp").is_none());
    }

    #[test]
    fn assemble_pdf_html_custom_css() {
        let ctx = PdfContext {
            project_title: "CSS Test".into(),
            custom_css: Some("body { background: red; }".into()),
            ..default_ctx()
        };
        let html = assemble_pdf_html(&ctx).unwrap();
        assert!(html.contains("background: red"));
    }

    #[test]
    fn assemble_pdf_html_mermaid_script() {
        let ctx = PdfContext {
            project_title: "Mermaid Test".into(),
            mermaid_enabled: true,
            ..default_ctx()
        };
        let html = assemble_pdf_html(&ctx).unwrap();
        assert!(html.contains("mermaid"));
        assert!(html.contains("startOnLoad"));
    }

    #[test]
    fn assemble_pdf_html_rtl_lang() {
        let ctx = PdfContext {
            lang: "ar".into(),
            is_rtl: true,
            ..default_ctx()
        };
        let html = assemble_pdf_html(&ctx).unwrap();
        assert!(html.contains(r#"lang="ar""#));
        assert!(html.contains(r#"dir="rtl""#));
    }

    #[test]
    fn assemble_pdf_html_ltr_no_dir_attr() {
        let ctx = default_ctx();
        let html = assemble_pdf_html(&ctx).unwrap();
        assert!(html.contains(r#"lang="en""#));
        assert!(!html.contains("dir="));
    }

    #[test]
    fn assemble_pdf_html_paper_size_css() {
        let ctx = PdfContext {
            paper_size_css: "Letter".into(),
            ..default_ctx()
        };
        let html = assemble_pdf_html(&ctx).unwrap();
        assert!(html.contains("size: Letter"));
    }

    #[test]
    fn locale_output_path_with_extension() {
        let p = locale_output_path(Path::new("guide.pdf"), "en");
        assert_eq!(p, PathBuf::from("guide.en.pdf"));

        let p = locale_output_path(Path::new("out/guide.pdf"), "fr");
        assert_eq!(p, PathBuf::from("out/guide.fr.pdf"));
    }

    #[test]
    fn locale_output_path_without_extension() {
        let p = locale_output_path(Path::new("guide"), "de");
        assert_eq!(p, PathBuf::from("guide.de"));
    }

    // ── rewrite_links_for_pdf ─────────────────────────────────────────────────

    fn two_page_inventory() -> (tempfile::TempDir, PageInventory) {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.md"), "# Home").unwrap();
        fs::write(docs.join("setup.md"), "# Setup").unwrap();
        let inv = PageInventory::scan(&docs, None, None, None).unwrap();
        (dir, inv)
    }

    #[test]
    fn rewrite_links_rewrites_known_page_href() {
        let (_dir, inv) = two_page_inventory();
        let html = r#"<p>See <a href="/setup.html">Setup</a> for details.</p>"#;
        let out = rewrite_links_for_pdf(html, &inv, None);
        assert!(
            out.contains("href=\"#setup\""),
            "expected anchor link, got: {out}"
        );
        assert!(!out.contains("/setup.html"), "original href should be gone");
    }

    #[test]
    fn rewrite_links_leaves_external_links_untouched() {
        let (_dir, inv) = two_page_inventory();
        let html = r#"<a href="https://example.com/page.html">External</a>"#;
        let out = rewrite_links_for_pdf(html, &inv, None);
        assert_eq!(out, html);
    }

    #[test]
    fn rewrite_links_rewrites_multiple_occurrences() {
        let (_dir, inv) = two_page_inventory();
        let html = r#"<a href="/setup.html">A</a> and <a href="/setup.html">B</a>"#;
        let out = rewrite_links_for_pdf(html, &inv, None);
        assert_eq!(out.matches("href=\"#setup\"").count(), 2);
    }

    #[test]
    fn rewrite_links_rewrites_all_known_pages() {
        let (_dir, inv) = two_page_inventory();
        let html = r#"<a href="/index.html">Home</a> <a href="/setup.html">Setup</a>"#;
        let out = rewrite_links_for_pdf(html, &inv, None);
        assert!(out.contains("href=\"#index\""));
        assert!(out.contains("href=\"#setup\""));
    }
}
