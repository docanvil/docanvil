use tera::{Context, Tera};

use crate::error::{Error, Result};
use crate::theme::Theme;

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
        context.insert("live_reload", &ctx.live_reload);

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
    pub live_reload: bool,
}
