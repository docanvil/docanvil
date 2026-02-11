use crate::components::{Component, ComponentContext};
use crate::error::Result;

pub struct CodeGroup;

impl Component for CodeGroup {
    fn name(&self) -> &str {
        "code-group"
    }

    fn render(&self, ctx: &ComponentContext) -> Result<String> {
        // Code groups work like tabs but are specifically for code blocks.
        // Parse code blocks from the raw body, using ```lang as tab labels.
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
                blocks.push((current_lang.clone(), current_code.join("\n")));
            } else if in_block {
                current_code.push(line.to_string());
            }
        }

        let mut html = String::from("<div class=\"code-group\">\n<div class=\"tab-headers\">\n");
        for (i, (lang, _)) in blocks.iter().enumerate() {
            let active = if i == 0 { " active" } else { "" };
            html.push_str(&format!(
                "  <button class=\"tab-header{active}\" data-tab=\"{i}\">{lang}</button>\n"
            ));
        }
        html.push_str("</div>\n");

        for (i, (lang, code)) in blocks.iter().enumerate() {
            let active = if i == 0 { " active" } else { "" };
            let escaped = html_escape(code);
            html.push_str(&format!(
                "<div class=\"tab-content{active}\" data-tab=\"{i}\"><pre><code class=\"language-{lang}\">{escaped}</code></pre></div>\n"
            ));
        }
        html.push_str("</div>");
        Ok(html)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
