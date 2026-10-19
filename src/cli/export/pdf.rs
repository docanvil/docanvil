use std::collections::{HashMap, HashSet};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use base64::Engine;
use regex::Regex;

use serde::Serialize;
use tera::{Context, Tera};

use crate::cli::build::{assemble_sources, exclude_drafts};
use crate::components::ComponentRegistry;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::nav;
use crate::pipeline;
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
    site_css: String,
    theme_custom_css: Option<String>,
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
                ..
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
fn rewrite_links_for_pdf(
    html: &str,
    inventory: &PageInventory,
    locale: Option<&str>,
    source: &PageInfo,
) -> String {
    let re = Regex::new(r#"(<a\b[^>]*\bhref=")([^"]*)(")"#).unwrap();
    re.replace_all(html, |caps: &regex::Captures| {
        let href = pipeline::headings::decode_entities(&caps[2]);
        let (path, fragment) = pipeline::links::split_target(&href);
        // Same-page fragments have already been scoped to this chapter.
        if path.is_empty() {
            return caps[0].to_owned();
        }
        let Some(output) = pipeline::links::local_output_path(path, source, "/") else {
            return caps[0].to_owned();
        };
        let Some(page) = inventory.pages.values().find(|page| {
            (locale.is_none() || page.locale.as_deref() == locale)
                && page.output_path == Path::new(&output)
        }) else {
            return caps[0].to_owned();
        };
        let suffix = fragment
            .filter(|fragment| !fragment.is_empty())
            .map(|fragment| format!("--{fragment}"))
            .unwrap_or_default();
        format!(
            "{}#{}{}{}",
            &caps[1],
            crate::util::html_escape(&page.slug),
            crate::util::html_escape(&suffix),
            &caps[3]
        )
    })
    .into_owned()
}

/// Scope page IDs (headings and native controls) to their PDF chapter.
fn scope_page_anchors(html: &str, slug: &str) -> String {
    let re = Regex::new(r#"(<[a-zA-Z][^>]*\s+id=")([^"]+)(")"#).expect("valid regex");
    let ids = pipeline::headings::anchor_ids(html);
    let href_re = Regex::new(r##"(href="#)([^"]+)(")"##).unwrap();
    let result = href_re.replace_all(html, |caps: &regex::Captures| {
        if pipeline::links::decode_fragment(&caps[2]).is_some_and(|id| ids.contains(&id)) {
            format!("{}{slug}--{}{}", &caps[1], &caps[2], &caps[3])
        } else {
            caps[0].to_owned()
        }
    });
    let result = re
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{}{slug}--{}{}", &caps[1], &caps[2], &caps[3])
        })
        .into_owned();
    let aria = Regex::new(r#"(aria-(?:controls|labelledby)=")([^"]+)(")"#).unwrap();
    aria.replace_all(&result, |caps: &regex::Captures| {
        let ids = caps[2]
            .split_whitespace()
            .map(|id| format!("{slug}--{id}"))
            .collect::<Vec<_>>()
            .join(" ");
        format!("{}{ids}{}", &caps[1], &caps[3])
    })
    .into_owned()
}

fn local_file_url(path: &Path) -> Option<String> {
    let path = path
        .canonicalize()
        .ok()?
        .to_string_lossy()
        .replace('\\', "/");
    let encoded: String = path
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"/-._~:".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect();
    Some(format!(
        "file://{}{encoded}",
        if encoded.starts_with('/') { "" } else { "/" }
    ))
}

/// Inline CSS lives in a temporary HTML file; resolve its local resources at source.
fn resolve_css_resources(css: &str, css_path: &Path, project_root: &Path) -> String {
    let re = Regex::new(r#"url\(\s*([^)]*?)\s*\)"#).expect("valid regex");
    re.replace_all(css, |caps: &regex::Captures| {
        let raw = caps[1].trim().trim_matches(['\'', '"']);
        if raw.starts_with('#') || raw.starts_with("//") || raw.contains(':') {
            return caps[0].to_string();
        }
        let path = if raw.starts_with('/') {
            project_root.join(raw.trim_start_matches('/'))
        } else {
            css_path.parent().unwrap_or(project_root).join(raw)
        };
        local_file_url(&path).map_or_else(|| caps[0].to_string(), |url| format!("url(\"{url}\")"))
    })
    .into_owned()
}

fn resolve_pdf_images(html: &str, project_root: &Path) -> String {
    let re = Regex::new(r#"(<img\b[^>]*\bsrc=")([^"]+)(")"#).expect("valid regex");
    re.replace_all(html, |caps: &regex::Captures| {
        let src = &caps[2];
        if src.contains(':') || src.starts_with("//") {
            return caps[0].to_string();
        }
        let path = project_root.join(src.trim_start_matches('/'));
        let url = local_file_url(&path).or_else(|| {
            local_file_url(
                &project_root
                    .join("static")
                    .join(src.trim_start_matches('/')),
            )
        });
        url.map_or_else(
            || caps[0].to_string(),
            |url| format!("{}{url}{}", &caps[1], &caps[3]),
        )
    })
    .into_owned()
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

/// Prepare one export's scoped content and styles without launching Chrome.
fn prepare_pdf(
    project_root: &Path,
    locale_arg: Option<&str>,
    version: Option<&str>,
    config: &Config,
    quiet: bool,
) -> Result<(
    PdfContext,
    Option<String>,
    Option<crate::git_versions::VersionSource>,
)> {
    let version_source = version
        .map(|v| crate::git_versions::VersionSource::resolve(project_root, config, v))
        .transpose()?;
    let source_root = version_source
        .as_ref()
        .map(|s| s.project_root.as_path())
        .unwrap_or(project_root);
    let content_dir = version_source
        .as_ref()
        .map(|s| s.content_dir.clone())
        .unwrap_or_else(|| project_root.join(&config.project.content_dir));
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
    let mut inventory = PageInventory::scan(
        &content_dir,
        enabled_locales,
        config.default_locale(),
        version,
    )?;

    // Determine the locale to export (None when i18n is disabled).
    let export_locale: Option<&str> = if config.is_i18n_enabled() {
        Some(locale_arg.unwrap_or_else(|| config.default_locale().unwrap_or("en")))
    } else {
        None
    };

    let (sources, front_matters) = assemble_sources(
        &mut inventory,
        config,
        &content_dir,
        version,
        version_source
            .as_ref()
            .and_then(|s| s.snapshot.as_ref())
            .map(crate::git_versions::GitSnapshot::repository_root),
    )?;
    for warning in &inventory.generation_warnings {
        crate::diagnostics::warn_openapi(warning);
    }

    // Drafts never go into the PDF.
    exclude_drafts(&mut inventory, &front_matters, config, false);

    // ── Build nav tree (after slug updates) ───────────────────────────────────
    let nav_tree = if let Some(locale) = export_locale {
        let nav_config = match version {
            Some(v) => nav::load_nav_for_version_and_locale(source_root, v, locale)?,
            None => nav::load_nav_for_locale(source_root, locale)?,
        };
        match nav_config {
            Some(entries) => {
                nav::validate_for_locale(&entries, &inventory, locale);
                nav::nav_tree_from_config_for_locale(&entries, &inventory, locale)
            }
            None => inventory.nav_tree_for_locale(locale),
        }
    } else {
        let nav_config = match version {
            Some(v) => nav::load_nav_for_version(source_root, v)?,
            None => nav::load_nav(source_root)?,
        };
        match nav_config {
            Some(entries) => {
                nav::validate(&entries, &inventory);
                nav::nav_tree_from_config(&entries, &inventory)
            }
            None => inventory.nav_tree(),
        }
    };

    let mut nav_tree = nav_tree;
    deduplicate_nav(&mut nav_tree, &mut HashSet::new());
    let flat_pages = flatten_nav_pages(&nav_tree);

    // ── Render each page through the pipeline ────────────────────────────────
    if !quiet {
        let n = flat_pages.len();
        eprintln!("Rendering {} page{}…", n, if n == 1 { "" } else { "s" });
    }
    let registry = ComponentRegistry::load(source_root)?;
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

        let html_body = pipeline::process_page(
            source,
            &inventory,
            page,
            &registry,
            "/",
            highlighter.as_ref(),
            config.syntax.line_numbers,
            source_root,
            export_locale,
        )?
        .html;
        let html_body = scope_page_anchors(&html_body, slug);
        let html_body = rewrite_links_for_pdf(&html_body, &inventory, export_locale, page);
        let html_body = resolve_pdf_images(&html_body, source_root);

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
            Ok(content) => Some(resolve_css_resources(&content, &css_file, project_root)),
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
    let mut merged_theme_vars: HashMap<String, String> = config
        .theme
        .variables
        .iter()
        .map(|(key, value)| (format!("--{}", key.trim_start_matches('-')), value.clone()))
        .collect();
    let mut theme_custom_css = None;
    if let Some(ref css_path) = config.theme.custom_css {
        let css_file = project_root.join(css_path);
        match std::fs::read_to_string(&css_file) {
            Ok(css) => {
                merged_theme_vars.extend(extract_css_vars_from_file(&css));
                theme_custom_css = Some(resolve_css_resources(&css, &css_file, project_root));
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
    let accent_color = merged_theme_vars.get("--color-primary").cloned();

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
        site_css: include_str!("../../theme/default/style.css").to_string(),
        theme_custom_css,
        mermaid_enabled: config.charts.enabled,
        mermaid_version: config.charts.mermaid_version.clone(),
        lang,
        is_rtl,
        paper_size_css,
        theme_css_vars,
    };
    Ok((ctx, accent_color, version_source))
}

fn run_single_locale(
    project_root: &Path,
    out: &Path,
    locale: Option<&str>,
    version: Option<&str>,
    config: &Config,
    quiet: bool,
) -> Result<()> {
    let (ctx, accent_color, _version_source) =
        prepare_pdf(project_root, locale, version, config, quiet)?;
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
            accent_color: accent_color.as_deref(),
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
    run_with_version(project_root, out, locale_arg, None, quiet)
}

pub fn run_with_version(
    project_root: &Path,
    out: &Path,
    locale_arg: Option<&str>,
    version_arg: Option<&str>,
    quiet: bool,
) -> Result<()> {
    let _remote_scope = crate::pipeline::includes::remote::Scope::new();
    if !project_root.join("docanvil.toml").is_file() {
        return Err(Error::General(format!(
            "No docanvil.toml found in {}. Run PDF export from the documentation project root or specify --path <project-root>.",
            project_root.display()
        )));
    }
    let config = Config::load(project_root)?;
    let interactive = !quiet && std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
    let (locale, version) = select_export_scope(&config, locale_arg, version_arg, interactive)?;
    if locale.as_deref() == Some("all") {
        for locale in &config.locale.enabled {
            let locale_out = locale_output_path(out, locale);
            run_single_locale(
                project_root,
                &locale_out,
                Some(locale),
                version.as_deref(),
                &config,
                quiet,
            )?;
        }
        return Ok(());
    }
    run_single_locale(
        project_root,
        out,
        locale.as_deref(),
        version.as_deref(),
        &config,
        quiet,
    )
}

fn select_export_scope(
    config: &Config,
    locale: Option<&str>,
    version: Option<&str>,
    interactive: bool,
) -> Result<(Option<String>, Option<String>)> {
    fn select(
        kind: &str,
        supplied: Option<&str>,
        enabled: &[String],
        default: Option<&str>,
        interactive: bool,
        allow_all: bool,
    ) -> Result<Option<String>> {
        let label = if kind == "doc-version" {
            "version"
        } else {
            "language"
        };
        if let Some(value) = supplied {
            if enabled.is_empty()
                || (!enabled.iter().any(|v| v == value) && !(allow_all && value == "all"))
            {
                return Err(Error::General(format!(
                    "Invalid --{kind} '{value}'; enabled {label}s: {}",
                    enabled.join(", ")
                )));
            }
            return Ok(Some(value.to_string()));
        }
        if enabled.is_empty() {
            return Ok(None);
        }
        if interactive && enabled.len() > 1 {
            let index = dialoguer::Select::new()
                .with_prompt(format!("Documentation {label} to export"))
                .items(enabled)
                .default(
                    enabled
                        .iter()
                        .position(|v| Some(v.as_str()) == default)
                        .unwrap_or(0),
                )
                .interact()
                .map_err(|e| Error::General(format!("prompt failed: {e}")))?;
            return Ok(Some(enabled[index].clone()));
        }
        Ok(default
            .map(String::from)
            .or_else(|| enabled.first().cloned()))
    }
    // Validate explicit selections before prompting for anything else.
    if locale.is_some() {
        select(
            "locale",
            locale,
            &config.locale.enabled,
            config.default_locale(),
            false,
            true,
        )?;
    }
    if version.is_some() {
        select(
            "doc-version",
            version,
            &config.version.enabled,
            config.current_version(),
            false,
            false,
        )?;
    }
    let version = select(
        "doc-version",
        version,
        &config.version.enabled,
        config.current_version(),
        interactive,
        false,
    )?;
    let locale = select(
        "locale",
        locale,
        &config.locale.enabled,
        config.default_locale(),
        interactive,
        true,
    )?;
    Ok((locale, version))
}

/// Keep the first navigation occurrence of each page, preserving chapter order.
fn deduplicate_nav(nodes: &mut Vec<NavNode>, seen: &mut HashSet<String>) {
    nodes.retain_mut(|node| match node {
        NavNode::Page { slug, .. } => seen.insert(slug.clone()),
        NavNode::Separator { .. } => true,
        NavNode::Group { slug, children, .. } => {
            if let Some(s) = slug.as_ref()
                && !seen.insert(s.clone())
            {
                *slug = None;
            }
            deduplicate_nav(children, seen);
            slug.is_some() || !children.is_empty()
        }
    });
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

    fn export_fixture() -> (tempfile::TempDir, Config) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("docanvil.toml"),
            r#"
[project]
name = "Scoped Docs"
[version]
current = "v2"
enabled = ["v1", "v2"]
[locale]
default = "en"
enabled = ["en", "fr"]
[theme]
custom_css = "theme/brand.css"
[theme.variables]
font-body = "Georgia, serif"
[pdf]
custom_css = "theme/print.css"
"#,
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join("theme")).unwrap();
        std::fs::write(dir.path().join("theme/brand.css"), ":root { --color-primary: #123456; } .content h2 { color: #abcdef; } .note { border-width: 3px; }").unwrap();
        std::fs::write(
            dir.path().join("theme/print.css"),
            ".pdf-chapter { font-size: 10pt; }",
        )
        .unwrap();
        for version in ["v1", "v2"] {
            std::fs::create_dir_all(dir.path().join(format!("docs/{version}"))).unwrap();
            for locale in ["en", "fr"] {
                let suffix = if locale == "en" {
                    String::new()
                } else {
                    format!(".{locale}")
                };
                for (slug, content) in [
                    (
                        "index",
                        format!(
                            "---\n{{\"title\": \"{version} {locale} Home\"}}\n---\n# Home\n\nTOKEN-{version}-{locale}\n\n[[guide]]\n\n## Shared\n\n[Local](#shared)\n\n[Other](/{}/{}/guide.html#shared)",
                            version, locale
                        ),
                    ),
                    (
                        "guide",
                        "# Guide\n\n## Shared\n\n:::note\nStyled note.\n:::\n".to_string(),
                    ),
                    (
                        "draft",
                        "---\n{\"draft\": true}\n---\n# Draft\nDRAFT-CONTENT".to_string(),
                    ),
                ] {
                    std::fs::write(
                        dir.path().join(format!("docs/{version}/{slug}{suffix}.md")),
                        content,
                    )
                    .unwrap();
                }
                std::fs::write(dir.path().join(format!("nav.{version}.{locale}.toml")), format!("[[nav]]\npage = \"guide\"\nlabel = \"NAV-{version}-{locale}\"\n\n[[nav]]\npage = \"index\"\n\n[[nav]]\npage = \"guide\"\n")).unwrap();
            }
        }
        let config = Config::load(dir.path()).unwrap();
        (dir, config)
    }

    #[test]
    fn pdf_includes_generated_api_chapters_and_scoped_links() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("docs")).unwrap();
        std::fs::write(
            dir.path().join("docanvil.toml"),
            "[project]\nname = 'API'\n[[openapi]]\nspec = 'openapi.json'\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("docs/index.md"), "# Guide\n[[api/index]]\n[[api/operations/listpets#parameters|API parameters]]\n[API responses](api/operations/listpets.html?view=full#responses)").unwrap();
        std::fs::write(
            dir.path().join("docs/openapi.json"),
            include_str!("../../../tests/fixtures/openapi/petstore.json"),
        )
        .unwrap();
        let mut spec: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/openapi/petstore.json"
        ))
        .unwrap();
        spec["tags"] = serde_json::json!([{"name": "First"}, {"name": "Second"}]);
        spec["paths"]["/pets"]["get"]["tags"] = serde_json::json!(["Second", "First"]);
        std::fs::write(dir.path().join("docs/openapi.json"), spec.to_string()).unwrap();
        let config = Config::load(dir.path()).unwrap();
        let (ctx, _, _) = prepare_pdf(dir.path(), None, None, &config, true).unwrap();
        assert!(ctx.chapters.iter().any(|c| c.slug == "api/index"));
        let guide = ctx.chapters.iter().find(|c| c.slug == "index").unwrap();
        assert!(
            guide
                .content_html
                .contains("href=\"#api/operations/listpets--parameters\"")
        );
        assert!(
            guide
                .content_html
                .contains("href=\"#api/operations/listpets--responses\"")
        );
        assert_eq!(
            ctx.chapters
                .iter()
                .filter(|c| c.slug == "api/operations/listpets")
                .count(),
            1
        );
        let chapter = ctx
            .chapters
            .iter()
            .find(|c| c.slug == "api/operations/listpets")
            .unwrap();
        assert!(chapter.content_html.contains("href=\"#api/schemas/pet\""));
        assert!(
            chapter
                .content_html
                .contains("id=\"api/operations/listpets--parameters\"")
        );
        assert!(ctx.chapters.iter().any(|c| c.slug == "api/schemas/pet"));
    }

    #[test]
    fn tagged_pdf_keeps_snapshot_images_alive_until_rendering() {
        let dir = tempfile::tempdir().unwrap();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(dir.path())
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        git(&["init", "-b", "master"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        std::fs::create_dir_all(dir.path().join("assets")).unwrap();
        std::fs::write(
            dir.path().join("docanvil.toml"),
            "[project]\nname = 'Test'\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("docs/index.md"),
            "# Tagged chapter\n\n![Image](assets/image.svg)\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("assets/image.svg"), "<svg></svg>").unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "Release"]);
        git(&["tag", "v1"]);
        std::fs::write(dir.path().join("docs/index.md"), "# Master chapter\n").unwrap();
        std::fs::write(dir.path().join("docanvil.toml"), "[project]\nname = 'Test'\n[version]\ncurrent = 'v1'\nenabled = ['v1', 'master']\n[version.git]\nrefs = { v1 = 'refs/tags/v1' }\nworking_tree = 'master'\n").unwrap();
        let config = Config::load(dir.path()).unwrap();
        let (ctx, _, source) = prepare_pdf(dir.path(), None, Some("v1"), &config, true).unwrap();
        assert_eq!(ctx.chapters[0].title, "Tagged chapter");
        let source = source.unwrap();
        let image = source.project_root.join("assets/image.svg");
        assert!(image.is_file());
        assert!(
            ctx.chapters[0]
                .content_html
                .contains(&local_file_url(&image).unwrap())
        );
        let root = source.project_root.clone();
        drop(source);
        assert!(!root.exists());
    }

    #[test]
    fn exports_only_selected_version_and_locale_once_with_scoped_links_and_styles() {
        let (dir, config) = export_fixture();
        for version in ["v1", "v2"] {
            for locale in ["en", "fr"] {
                let (ctx, accent, _version_source) =
                    prepare_pdf(dir.path(), Some(locale), Some(version), &config, true).unwrap();
                assert_eq!(
                    ctx.chapters
                        .iter()
                        .map(|c| c.slug.as_str())
                        .collect::<Vec<_>>(),
                    ["guide", "index"]
                );
                let html = assemble_pdf_html(&ctx).unwrap();
                assert!(html.contains(&format!("TOKEN-{version}-{locale}")));
                for other_version in ["v1", "v2"] {
                    for other_locale in ["en", "fr"] {
                        if (version, locale) != (other_version, other_locale) {
                            assert!(
                                !html.contains(&format!("TOKEN-{other_version}-{other_locale}"))
                            );
                        }
                    }
                }
                assert!(!html.contains("DRAFT-CONTENT"));
                assert_eq!(ctx.toc_html.matches("href=\"#guide\"").count(), 1);
                assert!(ctx.toc_html.contains(&format!("NAV-{version}-{locale}")));
                assert!(html.contains("href=\"#guide\""));
                assert!(html.contains("href=\"#guide--shared\""));
                assert!(html.contains("href=\"#index--shared\""));
                assert_eq!(html.matches("id=\"guide--shared\"").count(), 1);
                assert_eq!(html.matches("id=\"index--shared\"").count(), 1);
                assert!(html.contains("--font-body: Georgia, serif"));
                assert!(html.contains(".content h2 { color: #abcdef; }"));
                assert!(html.contains(".note { border-width: 3px; }"));
                assert!(
                    html.find(".content h2 { color: #abcdef; }").unwrap()
                        < html.find(".pdf-chapter { font-size: 10pt; }").unwrap()
                );
                assert_eq!(accent.as_deref(), Some("#123456"));
            }
        }
    }

    #[test]
    fn export_selections_validate_and_default_without_prompts() {
        let (_dir, config) = export_fixture();
        assert_eq!(
            select_export_scope(&config, None, None, false).unwrap(),
            (Some("en".into()), Some("v2".into()))
        );
        assert_eq!(
            select_export_scope(&config, Some("all"), Some("v1"), false).unwrap(),
            (Some("all".into()), Some("v1".into()))
        );
        for (locale, version) in [(Some("de"), None), (None, Some("v3"))] {
            assert!(select_export_scope(&config, locale, version, false).is_err());
        }
        let config = Config::default();
        assert_eq!(
            select_export_scope(&config, None, None, false).unwrap(),
            (None, None)
        );
        assert!(select_export_scope(&config, Some("all"), None, false).is_err());
        assert!(select_export_scope(&config, None, Some("v1"), false).is_err());
    }

    #[test]
    fn pdf_local_images_and_css_resources_resolve_from_project_not_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("theme")).unwrap();
        std::fs::create_dir_all(dir.path().join("assets")).unwrap();
        std::fs::write(dir.path().join("assets/a b.png"), b"image").unwrap();
        let url = local_file_url(&dir.path().join("assets/a b.png")).unwrap();
        assert!(url.contains("a%20b.png"));
        let css = resolve_css_resources(
            ".hero { background: url('../assets/a b.png'); } .external { background: url(https://example.com/bg.png); }",
            &dir.path().join("theme/brand.css"),
            dir.path(),
        );
        assert!(css.contains(&url));
        assert!(css.contains("url(https://example.com/bg.png)"));
        let html = resolve_pdf_images(
            "<img src=\"/assets/a b.png\"><img src=\"https://example.com/a.png\">",
            dir.path(),
        );
        assert!(html.contains(&url));
        assert!(html.contains("src=\"https://example.com/a.png\""));
    }

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
        assert!(!vars.contains_key("--color-primary"));
        assert!(!vars.contains_key("--color-text"));
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
            open: false,
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
            site_css: include_str!("../../theme/default/style.css").to_string(),
            theme_custom_css: None,
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
        let out = rewrite_links_for_pdf(html, &inv, None, &inv.pages["index"]);
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
        let out = rewrite_links_for_pdf(html, &inv, None, &inv.pages["index"]);
        assert_eq!(out, html);
    }

    #[test]
    fn rewrite_links_rewrites_multiple_occurrences() {
        let (_dir, inv) = two_page_inventory();
        let html = r#"<a href="/setup.html">A</a> and <a href="/setup.html">B</a>"#;
        let out = rewrite_links_for_pdf(html, &inv, None, &inv.pages["index"]);
        assert_eq!(out.matches("href=\"#setup\"").count(), 2);
    }

    #[test]
    fn rewrite_links_rewrites_all_known_pages() {
        let (_dir, inv) = two_page_inventory();
        let html = r#"<a href="/index.html">Home</a> <a href="/setup.html">Setup</a>"#;
        let out = rewrite_links_for_pdf(html, &inv, None, &inv.pages["index"]);
        assert!(out.contains("href=\"#index\""));
        assert!(out.contains("href=\"#setup\""));
    }
    #[test]
    fn pdf_scopes_encoded_fragments_native_ids_and_control_relationships() {
        let html = r##"<h2 id="parameters">Parameters</h2><a href="#%70arameters">Read</a><section id="media-panel"></section><button id="media-tab" aria-controls="media-panel" aria-labelledby="parameters media-tab">Tab</button>"##;
        let scoped = scope_page_anchors(html, "api/operations/listpets");
        assert!(scoped.contains("href=\"#api/operations/listpets--%70arameters\""));
        assert!(scoped.contains("id=\"api/operations/listpets--media-panel\""));
        assert!(scoped.contains("aria-controls=\"api/operations/listpets--media-panel\""));
        assert!(scoped.contains("aria-labelledby=\"api/operations/listpets--parameters api/operations/listpets--media-tab\""));
    }
}
