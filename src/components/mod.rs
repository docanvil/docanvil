pub mod builtin;

use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::error::Result;
use crate::pipeline::directives::{self, DirectiveBlock};
use crate::pipeline::{headings, markdown, popovers};

/// Context passed to a component when rendering.
pub struct ComponentContext<'a> {
    pub attributes: HashMap<String, String>,
    /// The raw body text (before Markdown rendering).
    pub body_raw: String,
    /// The body rendered as HTML, nested components included.
    pub body_html: String,
    /// Renders a Markdown fragment exactly like a page body (nested components included).
    pub render_markdown: &'a dyn Fn(&str) -> String,
}

// Component HTML is swapped out for placeholders before comrak runs, then swapped
// back straight after. Otherwise a blank line inside a component's HTML would end
// comrak's raw HTML block and the rest would be re-parsed as Markdown.
// Block placeholders are HTML comments (a one-line HTML block); inline ones are
// private-use characters, so an inline component at the start of a line stays
// inside its paragraph.
static PLACEHOLDER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<!--da-component-(\d+)-->|\x{E000}(\d+)\x{E001}").unwrap());

fn block_placeholder(index: usize) -> String {
    format!("<!--da-component-{index}-->")
}

fn inline_placeholder(index: usize) -> String {
    format!("\u{E000}{index}\u{E001}")
}

fn restore_placeholders(html: &str, rendered: &[String]) -> String {
    PLACEHOLDER_RE
        .replace_all(html, |caps: &regex::Captures| {
            let index = caps
                .get(1)
                .or_else(|| caps.get(2))
                .and_then(|m| m.as_str().parse::<usize>().ok());
            match index.and_then(|i| rendered.get(i)) {
                Some(html) => html.clone(),
                None => caps[0].to_string(),
            }
        })
        .into_owned()
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

    /// Render a Markdown fragment to HTML: directives (nested, via this registry),
    /// popovers, custom heading IDs, then comrak. Pages and component bodies both
    /// go through here, so they support exactly the same Markdown.
    pub fn render_markdown(&self, source: &str, source_file: &Path) -> String {
        let mut rendered: Vec<String> = Vec::new();

        let source = directives::process_directives(source, &mut |block| {
            rendered.push(self.render_block(block, source_file));
            block_placeholder(rendered.len() - 1)
        });
        let source = directives::process_inline_directives(&source, &mut |block| {
            rendered.push(self.render_block(block, source_file));
            inline_placeholder(rendered.len() - 1)
        });
        let source = popovers::process_popovers(&source);
        let source = headings::extract_custom_heading_ids(&source);
        let html = markdown::render(&source);

        restore_placeholders(&html, &rendered)
    }

    /// Render a directive block using the registered component.
    /// Falls back to a generic div wrapper if no component is registered.
    pub fn render_block(&self, block: &DirectiveBlock, source_file: &Path) -> String {
        let render_markdown = |s: &str| self.render_markdown(s, source_file);
        let ctx = ComponentContext {
            attributes: block.attributes.clone(),
            body_raw: block.body.clone(),
            body_html: render_markdown(&block.body),
            render_markdown: &render_markdown,
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
    use std::path::Path;

    #[test]
    fn registry_renders_note() {
        let registry = ComponentRegistry::with_builtins();
        let block = DirectiveBlock {
            name: "note".to_string(),
            attributes: HashMap::new(),
            body: "This is important.".to_string(),
        };
        let html = registry.render_block(&block, Path::new("test.md"));
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
        let html = registry.render_block(&block, Path::new("test.md"));
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
        let html = registry.render_block(&block, Path::new("test.md"));
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
        let html = registry.render_block(&block, Path::new("test.md"));
        assert_eq!(html, "<custom>hello</custom>");
    }

    /// Render one directive through the builtin registry.
    fn render(name: &str, attrs: &[(&str, &str)], body: &str) -> String {
        let registry = ComponentRegistry::with_builtins();
        let block = DirectiveBlock {
            name: name.to_string(),
            attributes: attrs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            body: body.to_string(),
        };
        registry.render_block(&block, Path::new("test.md"))
    }

    fn render_page(source: &str) -> String {
        ComponentRegistry::with_builtins().render_markdown(source, Path::new("test.md"))
    }

    #[test]
    fn nested_directives_render() {
        let html = render_page("::::note\nOuter\n\n:::warning\nInner\n:::\n::::\n");
        assert!(html.contains("admonition note"), "{html}");
        assert!(html.contains("admonition warning"), "{html}");
        assert!(!html.contains(":::warning"), "{html}");
    }

    #[test]
    fn inline_directive_inside_block_body_renders() {
        let html = render_page(":::note\nStatus: :::lozenge{type=\"green\" text=\"Done\"}\n:::\n");
        assert!(
            html.contains("<span class=\"lozenge green\">Done</span>"),
            "{html}"
        );
    }

    #[test]
    fn inline_directive_at_line_start_stays_in_paragraph() {
        let html = render_page(":::lozenge{type=\"x\" text=\"Done\"} and more\n");
        assert_eq!(
            html.trim(),
            "<p><span class=\"lozenge x\">Done</span> and more</p>"
        );
    }

    #[test]
    fn component_html_with_blank_lines_is_not_reparsed() {
        struct Spaced;
        impl Component for Spaced {
            fn name(&self) -> &str {
                "spaced"
            }
            fn render(&self, _ctx: &ComponentContext) -> crate::error::Result<String> {
                Ok("<div class=\"spaced\">\n\n    <p>indented</p>\n\n</div>".to_string())
            }
        }
        let mut registry = ComponentRegistry::with_builtins();
        registry.register(Box::new(Spaced));
        let html = registry.render_markdown(":::spaced\n:::\n\nAfter\n", Path::new("test.md"));
        assert!(html.contains("    <p>indented</p>"), "{html}");
        assert!(!html.contains("<pre>"), "{html}");
        assert!(html.contains("<p>After</p>"), "{html}");
    }

    #[test]
    fn popovers_work_inside_component_bodies() {
        let html = render_page(":::note\nSee ^[a tip]\n:::\n");
        assert!(html.contains("popover-trigger"), "{html}");
    }

    #[test]
    fn tabs_children_can_nest_components() {
        let html =
            render_page("::::tabs\n:::tab{title=\"A\"}\n:::lozenge{text=\"x\"}\n:::\n::::\n");
        assert!(
            html.contains("<span class=\"lozenge default\">x</span>"),
            "{html}"
        );
    }

    #[test]
    fn characterize_note() {
        assert_eq!(
            render("note", &[("title", "Heads up")], "Hello"),
            "<div class=\"admonition note\">\n<p class=\"admonition-title\">Heads up</p>\n<p>Hello</p>\n</div>"
        );
    }

    #[test]
    fn characterize_warning_default_title() {
        assert_eq!(
            render("warning", &[], "Careful"),
            "<div class=\"admonition warning\">\n<p class=\"admonition-title\">Warning</p>\n<p>Careful</p>\n</div>"
        );
    }

    #[test]
    fn characterize_lozenge() {
        assert_eq!(
            render("lozenge", &[("type", "yellow"), ("text", "Not Done")], ""),
            "<span class=\"lozenge yellow\">Not Done</span>"
        );
        assert_eq!(
            render("lozenge", &[], ""),
            "<span class=\"lozenge default\"></span>"
        );
    }

    #[test]
    fn characterize_mermaid() {
        assert_eq!(
            render("mermaid", &[], "graph TD\n    A --> B"),
            "<pre class=\"mermaid\">graph TD\n    A --> B</pre>"
        );
    }

    #[test]
    fn characterize_tabs() {
        let body = ":::tab{title=\"One\"}\nFirst\n:::\n:::tab\nSecond\n:::";
        assert_eq!(
            render("tabs", &[], body),
            "<div class=\"tabs\">\n<div class=\"tab-headers\">\n  <button class=\"tab-header active\" data-tab=\"0\">One</button>\n  <button class=\"tab-header\" data-tab=\"1\">Tab 2</button>\n</div>\n<div class=\"tab-content active\" data-tab=\"0\">\n<p>First</p>\n</div>\n<div class=\"tab-content\" data-tab=\"1\">\n<p>Second</p>\n</div>\n</div>"
        );
    }

    #[test]
    fn characterize_code_group() {
        let body = "```rust\nfn main() {}\n```\n```\na < b\n```";
        assert_eq!(
            render("code-group", &[], body),
            "<div class=\"code-group\">\n<div class=\"tab-headers\">\n  <button class=\"tab-header active\" data-tab=\"0\">rust</button>\n  <button class=\"tab-header\" data-tab=\"1\">text</button>\n</div>\n<div class=\"tab-content active\" data-tab=\"0\"><pre><code class=\"language-rust\">fn main() {}</code></pre></div>\n<div class=\"tab-content\" data-tab=\"1\"><pre><code class=\"language-text\">a &lt; b</code></pre></div>\n</div>"
        );
    }

    #[test]
    fn characterize_unknown_fallback() {
        assert_eq!(
            render("custom-thing", &[], "Body"),
            "<div class=\"custom-thing\">\n<p>Body</p>\n</div>"
        );
    }
}
