use crate::components::{Component, ComponentContext};
use crate::error::Result;

pub struct Note;

impl Component for Note {
    fn name(&self) -> &str {
        "note"
    }

    fn render(&self, ctx: &ComponentContext) -> Result<String> {
        let title = ctx
            .attributes
            .get("title")
            .map(|s| s.as_str())
            .unwrap_or("Note");
        Ok(format!(
            "<div class=\"admonition note\">\n<p class=\"admonition-title\">{title}</p>\n{}</div>",
            ctx.body_html
        ))
    }
}
