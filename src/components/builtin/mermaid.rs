use crate::components::{Component, ComponentContext};
use crate::error::Result;

pub struct Mermaid;

impl Component for Mermaid {
    fn name(&self) -> &str {
        "mermaid"
    }

    fn render(&self, ctx: &ComponentContext) -> Result<String> {
        let escaped = html_escape(&ctx.body_raw);
        Ok(format!("<pre class=\"mermaid\">{escaped}</pre>"))
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn renders_mermaid_block() {
        let mermaid = Mermaid;
        let ctx = ComponentContext {
            attributes: HashMap::new(),
            body_raw: "graph TD\n    A --> B".to_string(),
            body_html: String::new(),
        };
        let html = mermaid.render(&ctx).unwrap();
        assert!(html.contains("<pre class=\"mermaid\">"));
        assert!(html.contains("graph TD"));
        assert!(html.contains("A --&gt; B"));
    }

    #[test]
    fn escapes_html_in_diagram() {
        let mermaid = Mermaid;
        let ctx = ComponentContext {
            attributes: HashMap::new(),
            body_raw: "graph TD\n    A[\"<script>alert(1)</script>\"]".to_string(),
            body_html: String::new(),
        };
        let html = mermaid.render(&ctx).unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
