use serde::Serialize;
use tera::{Context, Tera};

use crate::config::ColorMode;
use crate::edit::RepoLink;
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
/// Pages that don't open with an `<h1>` get the subtitle at the top instead, except
/// pages that open with a `:::hero`, which already is the page's header.
pub fn with_page_description(html: String, description: Option<&str>) -> String {
    let Some(description) = description.map(str::trim).filter(|d| !d.is_empty()) else {
        return html;
    };
    if html.trim_start().starts_with("<section class=\"hero\"") {
        return html;
    }
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

/// Human-readable form of an include loop such as `["a.html", "b.html", "a.html"]`.
/// `things` names what's looping in the plural ("templates", "files").
pub fn include_cycle_message(cycle: &[String], things: &str) -> String {
    if let [name, _] = cycle {
        format!("{name} includes itself, so it would never finish rendering")
    } else {
        format!(
            "{things} include each other in a loop ({}), so they would never finish rendering",
            cycle.join(" → ")
        )
    }
}

/// Tera-based template renderer.
pub struct TemplateRenderer {
    tera: Tera,
}

impl TemplateRenderer {
    /// Create a new renderer from the resolved theme.
    pub fn new(theme: &Theme) -> Result<Self> {
        let mut tera = Tera::default();
        tera.register_function("icon", crate::icons::template_icon);
        tera.add_raw_template("layout.html", &theme.layout_template)
            .map_err(|e| {
                Error::Render(format!(
                    "failed to parse template: {}",
                    crate::components::tera_error_message(&e)
                ))
            })?;
        Ok(Self { tera })
    }

    /// Render a page with the given context values.
    pub fn render_page(&self, ctx: &PageContext) -> Result<String> {
        let context = Context::from_serialize(ctx)
            .map_err(|e| Error::Render(format!("failed to serialize page context: {e}")))?;

        self.tera
            .render("layout.html", &context)
            .map_err(|e| Error::Render(format!("template render error: {e}")))
    }
}

/// All the data needed to render a single page.
#[derive(Serialize)]
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
    /// Repository link in the header (`[project] repo`).
    pub repo: Option<RepoLink>,
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
    /// This page is built from the working checkout rather than a release.
    pub current_version_unreleased: bool,
    /// All available versions for the version switcher.
    pub available_versions: Vec<VersionInfo>,
    /// The current/latest version code (from `version.current` config).
    pub latest_version: Option<String>,
    /// URL to the equivalent page (or version home) in the latest version.
    pub latest_version_url: Option<String>,
    /// "Edit this page" URL for the page's source on its Git host.
    pub edit_url: Option<String>,
    /// `edit_url` opens the source in a local editor (`docanvil serve`), not on the Git host.
    pub edit_local: bool,
    /// Trail of ancestor nav groups down to this page; empty for top-level pages.
    pub breadcrumbs: Vec<Breadcrumb>,
    /// When the page last changed, as `YYYY-MM-DD`; `None` hides the date.
    pub last_updated: Option<String>,
    /// The page is marked `"draft": true` (only rendered by `serve` and `build --drafts`).
    pub draft: bool,
    /// Show the on-page table of contents (front matter `"toc": false` hides it).
    pub toc: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template_result(templates: &[(&str, &str)]) -> tera::TeraResult<()> {
        Tera::default().add_raw_templates(templates.to_vec())
    }

    #[test]
    fn indirect_include_cycle_is_rejected() {
        let err = template_result(&[
            ("a.html", "{% include \"b.html\" %}"),
            ("b.html", "{% include \"a.html\" %}"),
        ])
        .unwrap_err();
        assert!(matches!(
            err.kind(),
            tera::ErrorKind::CircularInclude { .. }
        ));
    }

    #[test]
    fn nested_include_cycles_are_rejected() {
        for body in [
            "{% if x %}{% include \"t.html\" %}{% endif %}",
            "{% for i in xs %}{% include \"t.html\" %}{% endfor %}",
            "{% filter upper %}{% include \"t.html\" %}{% endfilter %}",
            "{% block b %}{% include \"t.html\" %}{% endblock %}",
            "{% component m() %}{% include \"t.html\" %}{% endcomponent %}",
        ] {
            let err = template_result(&[("t.html", body)]).unwrap_err();
            assert!(
                matches!(err.kind(), tera::ErrorKind::CircularInclude { .. }),
                "{err}"
            );
        }
    }

    #[test]
    fn shared_includes_without_a_cycle_pass() {
        template_result(&[
            ("a.html", "{% include \"b.html\" %}{% include \"c.html\" %}"),
            ("b.html", "{% include \"d.html\" %}"),
            ("c.html", "{% include \"d.html\" %}"),
            ("d.html", "leaf"),
        ])
        .unwrap();
    }

    #[test]
    fn self_including_layout_is_rejected() {
        let theme = Theme {
            layout_template: "{% include \"layout.html\" %}".into(),
            default_css: String::new(),
            default_js: String::new(),
            css_overrides: None,
            custom_css_path: None,
            custom_css: None,
        };
        let err = TemplateRenderer::new(&theme)
            .err()
            .expect("cycle should be an error");
        assert!(
            err.to_string().contains("layout.html includes itself"),
            "{err}"
        );
    }

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

    #[test]
    fn leading_hero_keeps_description_out_of_the_page() {
        let html =
            "<section class=\"hero\">\n<h1 class=\"hero-title\">T</h1>\n</section>".to_string();
        assert_eq!(with_page_description(html.clone(), Some("Desc")), html);
    }
}
