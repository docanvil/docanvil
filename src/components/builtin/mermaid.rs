use crate::components::{Component, ComponentContext};
use crate::error::Result;

pub struct Mermaid;

impl Component for Mermaid {
    fn name(&self) -> &str {
        "mermaid"
    }

    fn render(&self, ctx: &ComponentContext) -> Result<String> {
        Ok(format!("<pre class=\"mermaid\">{}</pre>", ctx.body_raw))
    }
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
        assert!(html.contains("A --> B"));
    }

    #[test]
    fn preserves_mermaid_syntax_unescaped() {
        let mermaid = Mermaid;
        let ctx = ComponentContext {
            attributes: HashMap::new(),
            body_raw: "graph TD\n    A[Write Markdown] --> B[Build]".to_string(),
            body_html: String::new(),
        };
        let html = mermaid.render(&ctx).unwrap();
        // Content must not be HTML-escaped — mermaid v11 reads innerHTML,
        // so entities like &gt; would be passed literally to the parser.
        assert!(html.contains("-->"));
        assert!(!html.contains("&gt;"));
    }
}
