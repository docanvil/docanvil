use serde::Serialize;

use crate::components::{Component, ComponentContext};
use crate::error::Result;

/// Code groups: each fenced code block in the body becomes one tab.
pub struct CodeGroup;

#[derive(Serialize)]
struct CodeBlock {
    lang: String,
    /// Rest of the fence's info string after the language (DocAnvil's
    /// line-number and caption settings travel here).
    meta: String,
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
        // Backtick count of the open fence: only a bare fence at least that
        // long closes it, so a ```` fence can hold ``` lines.
        let mut open: Option<usize> = None;
        let mut lang = String::new();
        let mut meta = String::new();
        let mut code: Vec<&str> = Vec::new();

        for line in ctx.body_raw.lines() {
            let trimmed = line.trim();
            let ticks = trimmed.chars().take_while(|&c| c == '`').count();
            match open {
                None if ticks >= 3 => {
                    let info = trimmed[ticks..].trim();
                    let (first, rest) = info.split_once(char::is_whitespace).unwrap_or((info, ""));
                    lang = if first.is_empty() {
                        "text".to_string()
                    } else {
                        first.to_string()
                    };
                    meta = rest.trim().to_string();
                    code.clear();
                    open = Some(ticks);
                }
                Some(len) if ticks >= len && ticks == trimmed.len() => {
                    blocks.push(CodeBlock {
                        lang: lang.clone(),
                        meta: meta.clone(),
                        code: code.join("\n"),
                    });
                    open = None;
                }
                Some(_) => code.push(line),
                None => {}
            }
        }

        let mut data = tera::Context::new();
        data.insert("blocks", &blocks);
        Ok(data)
    }
}
