pub mod builtin;

use std::collections::HashMap;

use crate::error::Result;
use crate::pipeline::directives::DirectiveBlock;

/// Context passed to a component when rendering.
pub struct ComponentContext {
    pub attributes: HashMap<String, String>,
    /// The raw body text (before Markdown rendering).
    pub body_raw: String,
    /// The body rendered as HTML (via comrak).
    pub body_html: String,
}

/// Trait for custom components that handle directive blocks.
pub trait Component: Send + Sync {
    fn name(&self) -> &str;
    fn render(&self, ctx: &ComponentContext) -> Result<String>;
}

/// Registry mapping directive names to component implementations.
pub struct ComponentRegistry {
    components: HashMap<String, Box<dyn Component>>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Create a registry with all built-in components pre-registered.
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(builtin::note::Note));
        registry.register(Box::new(builtin::warning::Warning));
        registry.register(Box::new(builtin::tabs::Tabs));
        registry.register(Box::new(builtin::code_group::CodeGroup));
        registry.register(Box::new(builtin::mermaid::Mermaid));
        registry.register(Box::new(builtin::lozenge::Lozenge));
        registry
    }

    pub fn register(&mut self, component: Box<dyn Component>) {
        self.components
            .insert(component.name().to_string(), component);
    }

    /// Render a directive block using the registered component.
    /// Falls back to a generic div wrapper if no component is registered.
    pub fn render_block(&self, block: &DirectiveBlock) -> String {
        let body_html = crate::pipeline::markdown::render(&block.body);
        let ctx = ComponentContext {
            attributes: block.attributes.clone(),
            body_raw: block.body.clone(),
            body_html,
        };

        if let Some(component) = self.components.get(&block.name) {
            match component.render(&ctx) {
                Ok(html) => html,
                Err(e) => {
                    format!(
                        "<div class=\"directive-error\">Error rendering {}: {}</div>",
                        block.name, e
                    )
                }
            }
        } else {
            // Default: wrap in a div with the directive name as class
            let attrs = attr_string(&block.attributes);
            format!(
                "<div class=\"{}\"{}>\n{}</div>",
                block.name, attrs, ctx.body_html
            )
        }
    }
}

/// Convert attributes map to HTML attribute string.
fn attr_string(attrs: &HashMap<String, String>) -> String {
    let mut s = String::new();
    for (k, v) in attrs {
        if k == "class" || k == "id" {
            continue; // handled separately
        }
        s.push_str(&format!(" data-{k}=\"{v}\""));
    }
    if let Some(id) = attrs.get("id") {
        s.push_str(&format!(" id=\"{id}\""));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::directives::DirectiveBlock;

    #[test]
    fn registry_renders_note() {
        let registry = ComponentRegistry::with_builtins();
        let block = DirectiveBlock {
            name: "note".to_string(),
            attributes: HashMap::new(),
            body: "This is important.".to_string(),
        };
        let html = registry.render_block(&block);
        assert!(html.contains("note"));
        assert!(html.contains("This is important."));
    }

    #[test]
    fn registry_renders_lozenge() {
        let registry = ComponentRegistry::with_builtins();
        let block = DirectiveBlock {
            name: "lozenge".to_string(),
            attributes: HashMap::from([
                ("type".to_string(), "yellow".to_string()),
                ("text".to_string(), "Not Done".to_string()),
            ]),
            body: "".to_string(),
        };
        let html = registry.render_block(&block);
        assert!(html.contains("<span class=\"lozenge yellow\">Not Done</span>"));
    }

    #[test]
    fn registry_renders_unknown_as_div() {
        let registry = ComponentRegistry::with_builtins();
        let block = DirectiveBlock {
            name: "custom-thing".to_string(),
            attributes: HashMap::new(),
            body: "Body text".to_string(),
        };
        let html = registry.render_block(&block);
        assert!(html.contains("<div class=\"custom-thing\">"));
    }

    #[test]
    fn custom_component_registration() {
        struct MyComponent;
        impl Component for MyComponent {
            fn name(&self) -> &str {
                "my-comp"
            }
            fn render(&self, ctx: &ComponentContext) -> crate::error::Result<String> {
                Ok(format!("<custom>{}</custom>", ctx.body_raw))
            }
        }

        let mut registry = ComponentRegistry::new();
        registry.register(Box::new(MyComponent));

        let block = DirectiveBlock {
            name: "my-comp".to_string(),
            attributes: HashMap::new(),
            body: "hello".to_string(),
        };
        let html = registry.render_block(&block);
        assert_eq!(html, "<custom>hello</custom>");
    }
}
