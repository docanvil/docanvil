use crate::components::{Component, ComponentContext};
use crate::error::Result;
use crate::pipeline::directives;

pub struct Tabs;

impl Component for Tabs {
    fn name(&self) -> &str {
        "tabs"
    }

    fn render(&self, ctx: &ComponentContext) -> Result<String> {
        // Parse inner :::tab directives from the raw body
        let mut tabs = Vec::new();
        directives::process_directives(&ctx.body_raw, &mut |block| {
            if block.name == "tab" {
                let title = block
                    .attributes
                    .get("title")
                    .cloned()
                    .unwrap_or_else(|| format!("Tab {}", tabs.len() + 1));
                let body_html = crate::pipeline::markdown::render(&block.body);
                tabs.push((title, body_html));
            }
            String::new()
        });

        let mut html = String::from("<div class=\"tabs\">\n<div class=\"tab-headers\">\n");
        for (i, (title, _)) in tabs.iter().enumerate() {
            let active = if i == 0 { " active" } else { "" };
            html.push_str(&format!(
                "  <button class=\"tab-header{active}\" data-tab=\"{i}\">{title}</button>\n"
            ));
        }
        html.push_str("</div>\n");

        for (i, (_, body)) in tabs.iter().enumerate() {
            let active = if i == 0 { " active" } else { "" };
            html.push_str(&format!(
                "<div class=\"tab-content{active}\" data-tab=\"{i}\">\n{body}</div>\n"
            ));
        }
        html.push_str("</div>");
        Ok(html)
    }
}
