use crate::components::{Component, ComponentContext};
use crate::error::Result;

pub struct Warning;

impl Component for Warning {
    fn name(&self) -> &str {
        "warning"
    }

    fn render(&self, ctx: &ComponentContext) -> Result<String> {
        let title = ctx
            .attributes
            .get("title")
            .map(|s| s.as_str())
            .unwrap_or("Warning");
        Ok(format!(
            "<div class=\"admonition warning\">\n<p class=\"admonition-title\">{title}</p>\n{}</div>",
            ctx.body_html
        ))
    }
}
