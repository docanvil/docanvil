use serde::Serialize;
use tera::{Context, Tera};

use crate::config::ColorMode;
use crate::error::{Error, Result};
use crate::project::Crumb;
use crate::theme::Theme;
use crate::util::html_escape;

/// A link to a previous or next page.
#[derive(Debug, Clone, Serialize)]
pub struct PageLink {
    pub title: String,
    pub url: String,
}

/// One step in the breadcrumb trail shown above a page's content.
#[derive(Debug, Clone, Serialize)]
pub struct Breadcrumb {
    pub title: String,
    /// `None` for the current page and for groups without an index page.
    pub url: Option<String>,
}

/// Turn a nav breadcrumb trail into template breadcrumbs.
///
/// Top-level pages get no breadcrumbs — a trail of one is just the page title.
pub fn page_breadcrumbs(trail: Option<&Vec<Crumb>>, base_url: &str) -> Vec<Breadcrumb> {
    let Some(trail) = trail.filter(|t| t.len() > 1) else {
        return Vec::new();
    };
    let last = trail.len() - 1;
    trail
        .iter()
        .enumerate()
        .map(|(i, crumb)| Breadcrumb {
            title: crumb.label.clone(),
            url: crumb
                .slug
                .as_ref()
                .filter(|_| i < last)
                .map(|slug| format!("{base_url}{slug}.html")),
        })
        .collect()
}

/// Show the front matter `description` as a subtitle under the page's leading `<h1>`.
///
/// Pages that don't open with an `<h1>` get the subtitle at the top instead.
pub fn with_page_description(html: String, description: Option<&str>) -> String {
    let Some(description) = description.map(str::trim).filter(|d| !d.is_empty()) else {
        return html;
    };
    let subtitle = format!(
        "<p class=\"page-description\">{}</p>\n",
        html_escape(description)
    );
    let body = html.trim_start();
    if body.starts_with("<h1")
        && let Some(end) = body.find("</h1>")
    {
        let split = end + "</h1>".len();
        format!("{}\n{}{}", &body[..split], subtitle, &body[split..])
    } else {
        format!("{subtitle}{html}")
    }
}

/// Information about an available version for the version switcher.
#[derive(Debug, Clone, Serialize)]
pub struct VersionInfo {
    pub code: String,
    pub display_name: String,
    /// URL to this page in the given version, or the version's home page if `has_page` is false.
    pub url: String,
    pub is_current: bool,
    /// `false` when the current page doesn't exist in this version — link falls back to version home.
    pub has_page: bool,
}

/// Information about an available locale for the language switcher.
#[derive(Debug, Clone, Serialize)]
pub struct LocaleInfo {
    pub code: String,
    pub display_name: String,
    pub flag: String,
    pub url: String,
    pub absolute_url: Option<String>,
    pub is_current: bool,
    pub has_page: bool,
}

/// Tera-based template renderer.
pub struct TemplateRenderer {
    tera: Tera,
}

impl TemplateRenderer {
    /// Create a new renderer from the resolved theme.
    pub fn new(theme: &Theme) -> Result<Self> {
        let mut tera = Tera::default();
        tera.add_raw_template("layout.html", &theme.layout_template)
            .map_err(|e| Error::Render(format!("failed to parse template: {e}")))?;
        Ok(Self { tera })
    }

    /// Render a page with the given context values.
    pub fn render_page(&self, ctx: &PageContext) -> Result<String> {
        let mut context = Context::new();
        context.insert("page_title", &ctx.page_title);
        context.insert("project_name", &ctx.project_name);
        context.insert("content", &ctx.content);
        context.insert("nav_html", &ctx.nav_html);
        context.insert("default_css", &ctx.default_css);
        context.insert("css_overrides", &ctx.css_overrides);
        context.insert("custom_css_path", &ctx.custom_css_path);
        context.insert("custom_css", &ctx.custom_css);
        context.insert("base_url", &ctx.base_url);
        context.insert("logo_path", &ctx.logo_path);
        context.insert("favicon_path", &ctx.favicon_path);
        context.insert("live_reload", &ctx.live_reload);
        context.insert("mermaid_enabled", &ctx.mermaid_enabled);
        context.insert("mermaid_version", &ctx.mermaid_version);
        context.insert("search_enabled", &ctx.search_enabled);
        context.insert("meta_description", &ctx.meta_description);
        context.insert("meta_author", &ctx.meta_author);
        context.insert("meta_date", &ctx.meta_date);
        context.insert("prev_page", &ctx.prev_page);
        context.insert("next_page", &ctx.next_page);
        context.insert("color_mode", &ctx.color_mode);
        context.insert("js_cachebust", &ctx.js_cachebust);
        context.insert("current_locale", &ctx.current_locale);
        context.insert("current_flag", &ctx.current_flag);
        context.insert("available_locales", &ctx.available_locales);
        context.insert("locale_auto_detect", &ctx.locale_auto_detect);
        context.insert("canonical_url", &ctx.canonical_url);
        context.insert("x_default_url", &ctx.x_default_url);
        context.insert("search_index_url", &ctx.search_index_url);
        context.insert("current_version", &ctx.current_version);
        context.insert("available_versions", &ctx.available_versions);
        context.insert("latest_version", &ctx.latest_version);
        context.insert("latest_version_url", &ctx.latest_version_url);
        context.insert("edit_url", &ctx.edit_url);
        context.insert("breadcrumbs", &ctx.breadcrumbs);

        self.tera
            .render("layout.html", &context)
            .map_err(|e| Error::Render(format!("template render error: {e}")))
    }
}

/// All the data needed to render a single page.
pub struct PageContext {
    pub page_title: String,
    pub project_name: String,
    pub content: String,
    pub nav_html: String,
    pub default_css: String,
    pub css_overrides: Option<String>,
    pub custom_css_path: Option<String>,
    pub custom_css: Option<String>,
    pub base_url: String,
    pub logo_path: Option<String>,
    pub favicon_path: Option<String>,
    pub live_reload: bool,
    pub mermaid_enabled: bool,
    pub mermaid_version: String,
    pub search_enabled: bool,
    pub meta_description: Option<String>,
    pub meta_author: Option<String>,
    pub meta_date: Option<String>,
    pub prev_page: Option<PageLink>,
    pub next_page: Option<PageLink>,
    pub color_mode: ColorMode,
    pub js_cachebust: String,
    pub current_locale: Option<String>,
    pub current_flag: Option<String>,
    pub available_locales: Vec<LocaleInfo>,
    pub locale_auto_detect: bool,
    pub canonical_url: Option<String>,
    pub x_default_url: Option<String>,
    /// URL to the search index JSON for this page's locale (e.g. `/en/search-index.json`).
    pub search_index_url: String,
    /// Version code for the current page (e.g. "v2"), if versioning is enabled.
    pub current_version: Option<String>,
    /// All available versions for the version switcher.
    pub available_versions: Vec<VersionInfo>,
    /// The current/latest version code (from `version.current` config).
    pub latest_version: Option<String>,
    /// URL to the equivalent page (or version home) in the latest version.
    pub latest_version_url: Option<String>,
    /// "Edit this page" URL for the page's source on its Git host.
    pub edit_url: Option<String>,
    /// Trail of ancestor nav groups down to this page; empty for top-level pages.
    pub breadcrumbs: Vec<Breadcrumb>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crumb(label: &str, slug: Option<&str>) -> Crumb {
        Crumb {
            label: label.into(),
            slug: slug.map(Into::into),
        }
    }

    #[test]
    fn breadcrumbs_empty_for_top_level_page() {
        let trail = vec![crumb("Home", Some("index"))];
        assert!(page_breadcrumbs(Some(&trail), "/").is_empty());
        assert!(page_breadcrumbs(None, "/").is_empty());
    }

    #[test]
    fn breadcrumbs_link_groups_with_index_pages_only() {
        let trail = vec![
            crumb("Guides", Some("guides/index")),
            crumb("Advanced", None),
            crumb("Setup", Some("guides/advanced/setup")),
        ];
        let crumbs = page_breadcrumbs(Some(&trail), "/docs/");
        let urls: Vec<_> = crumbs.iter().map(|c| c.url.as_deref()).collect();
        assert_eq!(urls, vec![Some("/docs/guides/index.html"), None, None]);
        assert_eq!(crumbs[2].title, "Setup");
    }

    #[test]
    fn description_goes_after_leading_h1() {
        let html = "<h1 id=\"intro\">Intro</h1>\n<p>Body</p>\n".to_string();
        assert_eq!(
            with_page_description(html, Some("A & B")),
            "<h1 id=\"intro\">Intro</h1>\n<p class=\"page-description\">A &amp; B</p>\n\n<p>Body</p>\n"
        );
    }

    #[test]
    fn description_goes_first_without_leading_h1() {
        let html = "<p>Body</p>\n<h1>Later</h1>\n".to_string();
        assert!(
            with_page_description(html, Some("Desc"))
                .starts_with("<p class=\"page-description\">Desc</p>\n<p>Body</p>")
        );
    }

    #[test]
    fn no_description_leaves_html_alone() {
        let html = "<h1>T</h1>".to_string();
        assert_eq!(with_page_description(html.clone(), None), html);
        assert_eq!(with_page_description(html.clone(), Some("  ")), html);
    }
}
