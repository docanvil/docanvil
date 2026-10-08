use std::collections::HashSet;

use serde::Serialize;
use tera::ast::Node;
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

/// Find a chain of `{% include %}`s that leads back to where it started, such as
/// `["a.html", "b.html", "a.html"]`. Tera follows includes with no depth limit, so
/// rendering a template on such a chain overflows the stack instead of erroring.
/// Recursive macros aren't covered: they can legitimately stop on a condition.
pub fn find_include_cycle(tera: &Tera) -> Option<Vec<String>> {
    let mut names: Vec<&str> = tera.get_template_names().collect();
    names.sort_unstable();
    let mut done = HashSet::new();
    names
        .into_iter()
        .find_map(|name| visit_includes(tera, name, &mut Vec::new(), &mut done))
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

fn visit_includes<'a>(
    tera: &'a Tera,
    name: &'a str,
    path: &mut Vec<&'a str>,
    done: &mut HashSet<&'a str>,
) -> Option<Vec<String>> {
    if let Some(start) = path.iter().position(|n| *n == name) {
        let mut cycle: Vec<String> = path[start..].iter().map(|n| n.to_string()).collect();
        cycle.push(name.to_string());
        return Some(cycle);
    }
    if done.contains(name) {
        return None;
    }
    // A missing template can't loop; Tera reports it (or skips it) on its own.
    let template = tera.get_template(name).ok()?;
    let mut includes = Vec::new();
    collect_includes(&template.ast, &mut includes);

    path.push(name);
    for candidates in includes {
        // `{% include ["a.html", "b.html"] %}` renders the first one that exists.
        let Some(target) = candidates.iter().find(|c| tera.get_template(c).is_ok()) else {
            continue;
        };
        if let Some(cycle) = visit_includes(tera, target, path, done) {
            return Some(cycle);
        }
    }
    path.pop();
    done.insert(name);
    None
}

/// Every `{% include %}` in `nodes`, including those nested inside other tags.
fn collect_includes<'a>(nodes: &'a [Node], out: &mut Vec<&'a [String]>) {
    for node in nodes {
        match node {
            Node::Include(_, candidates, _) => out.push(candidates),
            Node::MacroDefinition(_, def, _) => collect_includes(&def.body, out),
            Node::FilterSection(_, section, _) => collect_includes(&section.body, out),
            Node::Block(_, block, _) => collect_includes(&block.body, out),
            Node::Forloop(_, forloop, _) => {
                collect_includes(&forloop.body, out);
                if let Some(body) = &forloop.empty_body {
                    collect_includes(body, out);
                }
            }
            Node::If(if_node, _) => {
                for (_, _, body) in &if_node.conditions {
                    collect_includes(body, out);
                }
                if let Some((_, body)) = &if_node.otherwise {
                    collect_includes(body, out);
                }
            }
            _ => {}
        }
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
        tera.add_raw_template("layout.html", &theme.layout_template)
            .map_err(|e| Error::Render(format!("failed to parse template: {e}")))?;
        if let Some(cycle) = find_include_cycle(&tera) {
            return Err(Error::Render(format!(
                "layout template: {}",
                include_cycle_message(&cycle, "templates")
            )));
        }
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

    fn tera_with(templates: &[(&str, &str)]) -> Tera {
        let mut tera = Tera::default();
        tera.add_raw_templates(templates.to_vec()).unwrap();
        tera
    }

    #[test]
    fn self_include_is_a_cycle() {
        let tera = tera_with(&[("loop.html", "<div>{% include \"loop.html\" %}</div>")]);
        assert_eq!(
            find_include_cycle(&tera),
            Some(vec!["loop.html".to_string(), "loop.html".to_string()])
        );
    }

    #[test]
    fn indirect_include_cycle_is_found() {
        let tera = tera_with(&[
            ("a.html", "{% include \"b.html\" %}"),
            ("b.html", "{% include \"c.html\" %}"),
            ("c.html", "{% include \"a.html\" %}"),
        ]);
        assert_eq!(
            find_include_cycle(&tera),
            Some(
                vec!["a.html", "b.html", "c.html", "a.html"]
                    .into_iter()
                    .map(String::from)
                    .collect()
            )
        );
    }

    #[test]
    fn includes_inside_nested_tags_are_followed() {
        for body in [
            "{% if x %}{% include \"t.html\" %}{% endif %}",
            "{% if x %}{% else %}{% include \"t.html\" %}{% endif %}",
            "{% for i in xs %}{% include \"t.html\" %}{% endfor %}",
            "{% for i in xs %}{% else %}{% include \"t.html\" %}{% endfor %}",
            "{% filter upper %}{% include \"t.html\" %}{% endfilter %}",
            "{% block b %}{% include \"t.html\" %}{% endblock %}",
            "{% macro m() %}{% include \"t.html\" %}{% endmacro %}",
        ] {
            let tera = tera_with(&[("t.html", body)]);
            assert!(
                find_include_cycle(&tera).is_some(),
                "missed cycle in {body}"
            );
        }
    }

    #[test]
    fn shared_includes_without_a_cycle_pass() {
        // a → b → d and a → c → d: d is reached twice, but nothing loops.
        let tera = tera_with(&[
            ("a.html", "{% include \"b.html\" %}{% include \"c.html\" %}"),
            ("b.html", "{% include \"d.html\" %}"),
            ("c.html", "{% include \"d.html\" %}"),
            ("d.html", "leaf"),
        ]);
        assert_eq!(find_include_cycle(&tera), None);
    }

    #[test]
    fn missing_optional_include_is_not_a_cycle() {
        let tera = tera_with(&[("a.html", "{% include \"nope.html\" ignore missing %}")]);
        assert_eq!(find_include_cycle(&tera), None);
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
}
