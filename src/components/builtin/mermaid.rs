use crate::components::Component;

/// Mermaid diagrams: the body is diagram source, not Markdown.
pub struct Mermaid;

impl Component for Mermaid {
    fn name(&self) -> &str {
        "mermaid"
    }

    fn renders_body(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::components::ComponentRegistry;
    use crate::pipeline::directives::DirectiveBlock;
    use std::collections::HashMap;
    use std::path::Path;

    fn render(body: &str) -> String {
        let block = DirectiveBlock {
            name: "mermaid".to_string(),
            attributes: HashMap::new(),
            body: body.to_string(),
            inline: false,
        };
        ComponentRegistry::with_builtins().render_block(&block, Path::new("t.md"))
    }

    #[test]
    fn renders_mermaid_block() {
        let html = render("graph TD\n    A --> B");
        assert!(html.contains("<pre class=\"mermaid\">"));
        assert!(html.contains("graph TD"));
        assert!(html.contains("A --> B"));
    }

    #[test]
    fn preserves_mermaid_syntax_unescaped() {
        let html = render("graph TD\n    A[Write Markdown] --> B[Build]");
        // Content must not be HTML-escaped — mermaid v11 reads innerHTML,
        // so entities like &gt; would be passed literally to the parser.
        assert!(html.contains("-->"));
        assert!(!html.contains("&gt;"));
    }
}
