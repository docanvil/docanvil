use serde::Serialize;

use crate::components::{Component, ComponentContext};
use crate::error::Result;
use crate::pipeline::directives;

/// Tabs: each `:::tab{title="…"}` child becomes one entry in `tabs`.
pub struct Tabs;

#[derive(Serialize)]
struct Tab {
    title: String,
    body: String,
}

impl Component for Tabs {
    fn name(&self) -> &str {
        "tabs"
    }

    fn renders_body(&self) -> bool {
        false
    }

    fn data(&self, ctx: &ComponentContext) -> Result<tera::Context> {
        let mut tabs: Vec<Tab> = Vec::new();
        directives::process_directives(ctx.body_raw, &mut |block| {
            if block.name == "tab" {
                let title = block
                    .attributes
                    .get("title")
                    .cloned()
                    .unwrap_or_else(|| format!("Tab {}", tabs.len() + 1));
                let body = (ctx.render_markdown)(&block.body);
                tabs.push(Tab { title, body });
            }
            String::new()
        });

        let mut data = tera::Context::new();
        data.insert("tabs", &tabs);
        Ok(data)
    }
}
