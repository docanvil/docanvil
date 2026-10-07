use serde::Serialize;

use crate::components::{Component, ComponentContext};
use crate::error::Result;

/// Code groups: each fenced code block in the body becomes one tab.
pub struct CodeGroup;

#[derive(Serialize)]
struct CodeBlock {
    lang: String,
    code: String,
}

impl Component for CodeGroup {
    fn name(&self) -> &str {
        "code-group"
    }

    fn renders_body(&self) -> bool {
        false
    }

    fn data(&self, ctx: &ComponentContext) -> Result<tera::Context> {
        let mut blocks = Vec::new();
        let mut current_lang = String::new();
        let mut current_code = Vec::new();
        let mut in_block = false;

        for line in ctx.body_raw.lines() {
            if line.starts_with("```") && !in_block {
                in_block = true;
                current_lang = line.trim_start_matches('`').trim().to_string();
                if current_lang.is_empty() {
                    current_lang = "text".to_string();
                }
                current_code.clear();
            } else if line.starts_with("```") && in_block {
                in_block = false;
                blocks.push(CodeBlock {
                    lang: current_lang.clone(),
                    code: current_code.join("\n"),
                });
            } else if in_block {
                current_code.push(line.to_string());
            }
        }

        let mut data = tera::Context::new();
        data.insert("blocks", &blocks);
        Ok(data)
    }
}
