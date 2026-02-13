use crate::components::{Component, ComponentContext};
use crate::error::Result;

pub struct Lozenge;

impl Component for Lozenge {
    fn name(&self) -> &str {
        "lozenge"
    }

    fn render(&self, ctx: &ComponentContext) -> Result<String> {
        let variation = ctx
            .attributes
            .get("type")
            .map(|s| s.as_str())
            .unwrap_or("default");

        let text = ctx
            .attributes
            .get("text")
            .map(|s| s.as_str())
            .unwrap_or("");

        Ok(format!(
            "<span class=\"lozenge {variation}\">{text}</span>"))
    }
}
