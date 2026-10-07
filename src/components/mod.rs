pub mod builtin;
pub mod templates;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use tera::Tera;

use crate::diagnostics::warn_component_render;
use crate::error::{Error, Result};
use crate::pipeline::directives::{self, DirectiveBlock};
use crate::pipeline::{headings, markdown, popovers};
use crate::util::html_escape;

/// What a component's data provider sees.
pub struct ComponentContext<'a> {
    pub attributes: &'a HashMap<String, String>,
    /// The raw body text (before Markdown rendering).
    pub body_raw: &'a str,
    /// True for inline `:::name{attrs}` use.
    pub inline: bool,
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

/// A builtin component's Rust half: extracts extra template variables from the
/// directive. The markup lives in the component's template.
pub trait Component: Send + Sync {
    fn name(&self) -> &str;

    /// Extra template variables beyond `attrs`, `body`, `body_raw`, `name`, `inline`.
    fn data(&self, _ctx: &ComponentContext) -> Result<tera::Context> {
        Ok(tera::Context::new())
    }

    /// Whether the body is Markdown to render into `body`. Components whose body is
    /// something else (diagram source, tab children, code blocks) return false.
    fn renders_body(&self) -> bool {
        true
    }
}

/// Maps directive names to templates and (for some builtins) data providers.
pub struct ComponentRegistry {
    providers: HashMap<String, Box<dyn Component>>,
    tera: Tera,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}

/// Escape a value printed by a component template. Same as `html_escape`, plus
/// single quotes. Tera's own escaper also rewrites `/` as `&#x2F;`, which is
/// noisy in output and isn't decoded by the search indexer.
fn escape_template_value(s: &str) -> String {
    html_escape(s).replace('\'', "&#39;")
}

/// Flatten a Tera error and its causes into one line.
pub(crate) fn tera_error_message(e: &tera::Error) -> String {
    let mut message = e.to_string();
    let mut source = std::error::Error::source(e);
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

impl ComponentRegistry {
    /// A registry with the builtin components and their embedded templates.
    pub fn with_builtins() -> Self {
        let mut tera = Tera::default();
        tera.set_escape_fn(escape_template_value);
        let mut registry = Self {
            providers: HashMap::new(),
            tera,
        };
        for name in templates::builtin_names() {
            let source = templates::builtin_source(&name).unwrap_or_default();
            registry
                .add_template(&name, &source)
                .expect("embedded component templates are valid");
        }
        registry.register(Box::new(builtin::tabs::Tabs));
        registry.register(Box::new(builtin::code_group::CodeGroup));
        registry.register(Box::new(builtin::mermaid::Mermaid));
        registry
    }

    /// Where a project's component templates live.
    pub fn user_template_dir(project_root: &Path) -> PathBuf {
        project_root.join("theme/components")
    }

    /// Builtins plus the project's `theme/components/*.html`. A user template with a
    /// builtin's name replaces its markup; any other name is a new component.
    pub fn load(project_root: &Path) -> Result<Self> {
        let mut registry = Self::with_builtins();
        let dir = Self::user_template_dir(project_root);
        if !dir.is_dir() {
            return Ok(registry);
        }

        let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "html"))
            .collect();
        paths.sort();

        // Read every template before registering any of them, then register them
        // in one batch. Tera resolves `{% import %}` / `{% extends %}` against
        // whatever's already in its template map, so adding files one at a time
        // (in alphabetical order) breaks a template that references one that
        // sorts later — e.g. `badge.html` importing macros from `zz-macros.html`.
        let mut sources = Vec::with_capacity(paths.len());
        for path in &paths {
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let source = std::fs::read_to_string(path)?;
            sources.push((path.clone(), stem.to_string(), source));
        }

        let batch: Vec<(String, &str)> = sources
            .iter()
            .map(|(_, stem, source)| (template_name(stem), source.as_str()))
            .collect();

        registry.tera.add_raw_templates(batch).map_err(|e| {
            let message = tera_error_message(&e);
            // Tera names the offending template in the message (e.g.
            // "Failed to parse 'badge.html'"); match it back to its file.
            let path = sources
                .iter()
                .find(|(_, stem, _)| message.contains(&template_name(stem)))
                .map(|(path, _, _)| path.clone())
                .unwrap_or_else(|| dir.clone());
            Error::ComponentTemplate { path, message }
        })?;

        Ok(registry)
    }

    pub fn register(&mut self, component: Box<dyn Component>) {
        self.providers
            .insert(component.name().to_string(), component);
    }

    /// Add (or replace) the template for component `name`.
    pub fn add_template(&mut self, name: &str, source: &str) -> std::result::Result<(), String> {
        self.tera
            .add_raw_template(&template_name(name), source)
            .map_err(|e| tera_error_message(&e))
    }

    fn has_template(&self, name: &str) -> bool {
        let wanted = template_name(name);
        self.tera.get_template_names().any(|n| n == wanted)
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

    /// Render one directive. Uses the component's template (plus provider data);
    /// falls back to a generic div when the name has no template.
    pub fn render_block(&self, block: &DirectiveBlock, source_file: &Path) -> String {
        let provider: Option<&dyn Component> = self.providers.get(&block.name).map(|p| p.as_ref());

        if provider.is_none() && !self.has_template(&block.name) {
            let body = self.render_markdown(&block.body, source_file);
            let attrs = attr_string(&block.attributes);
            return format!("<div class=\"{}\"{}>\n{}</div>", block.name, attrs, body);
        }

        let renders_body = provider.is_none_or(|p| p.renders_body());
        let body = if renders_body && !block.body.is_empty() {
            self.render_markdown(&block.body, source_file)
        } else {
            String::new()
        };

        let render_markdown = |s: &str| self.render_markdown(s, source_file);
        let ctx = ComponentContext {
            attributes: &block.attributes,
            body_raw: &block.body,
            inline: block.inline,
            render_markdown: &render_markdown,
        };

        match self.render_template(&block.name, &ctx, &body, provider) {
            Ok(html) => html.trim_matches('\n').to_string(),
            Err(message) => {
                warn_component_render(source_file, &block.name, &message);
                format!(
                    "<div class=\"directive-error\">Error rendering {}: {}</div>",
                    block.name,
                    html_escape(&message)
                )
            }
        }
    }

    fn render_template(
        &self,
        name: &str,
        ctx: &ComponentContext,
        body: &str,
        provider: Option<&dyn Component>,
    ) -> std::result::Result<String, String> {
        let mut context = tera::Context::new();
        context.insert("attrs", ctx.attributes);
        context.insert("body", body);
        context.insert("body_raw", ctx.body_raw);
        context.insert("name", name);
        context.insert("inline", &ctx.inline);
        if let Some(provider) = provider {
            context.extend(provider.data(ctx).map_err(|e| e.to_string())?);
        }
        self.tera
            .render(&template_name(name), &context)
            .map_err(|e| tera_error_message(&e))
    }
}

fn template_name(name: &str) -> String {
    format!("{name}.html")
}

/// Convert attributes map to HTML attribute string (generic fallback div).
fn attr_string(attrs: &HashMap<String, String>) -> String {
    let mut s = String::new();
    for (k, v) in attrs {
        if k == "class" || k == "id" {
            continue; // handled separately
        }
        s.push_str(&format!(" data-{k}=\"{}\"", html_escape(v)));
    }
    if let Some(id) = attrs.get("id") {
        s.push_str(&format!(" id=\"{}\"", html_escape(id)));
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
            inline: false,
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
            inline: false,
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
            inline: false,
        };
        let html = registry.render_block(&block, Path::new("test.md"));
        assert!(html.contains("<div class=\"custom-thing\">"));
    }

    #[test]
    fn custom_template_component() {
        let mut registry = ComponentRegistry::with_builtins();
        registry
            .add_template(
                "card",
                "<div class=\"card\"><h3>{{ attrs.title }}</h3>{{ body | safe }}</div>",
            )
            .unwrap();
        let html =
            registry.render_markdown(":::card{title=\"Hi\"}\nBody\n:::\n", Path::new("t.md"));
        assert!(
            html.contains("<div class=\"card\"><h3>Hi</h3><p>Body</p>\n</div>"),
            "{html}"
        );
    }

    #[test]
    fn template_with_blank_lines_is_not_reparsed() {
        let mut registry = ComponentRegistry::with_builtins();
        registry
            .add_template(
                "spaced",
                "<div class=\"spaced\">\n\n    <p>{{ attrs.text }}</p>\n\n</div>\n",
            )
            .unwrap();
        let html = registry.render_markdown(
            ":::spaced{text=\"indented\"}\n:::\n\nAfter\n",
            Path::new("t.md"),
        );
        assert!(html.contains("    <p>indented</p>"), "{html}");
        assert!(!html.contains("<pre>"), "{html}");
        assert!(html.contains("<p>After</p>"), "{html}");
    }

    #[test]
    fn override_keeps_builtin_data() {
        let mut registry = ComponentRegistry::with_builtins();
        registry
            .add_template("tabs", "{% for tab in tabs %}[{{ tab.title }}]{% endfor %}")
            .unwrap();
        let html = registry.render_markdown(
            "::::tabs\n:::tab{title=\"A\"}\nx\n:::\n:::tab{title=\"B\"}\ny\n:::\n::::\n",
            Path::new("t.md"),
        );
        assert!(html.contains("[A][B]"), "{html}");
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
            inline: false,
        };
        registry.render_block(&block, Path::new("test.md"))
    }

    fn render_page(source: &str) -> String {
        ComponentRegistry::with_builtins().render_markdown(source, Path::new("test.md"))
    }

    #[test]
    fn attribute_values_are_escaped() {
        assert_eq!(
            render("note", &[("title", "<b>Fish & \"Chips\"</b>")], "x"),
            "<div class=\"admonition note\">\n<p class=\"admonition-title\">&lt;b&gt;Fish &amp; &quot;Chips&quot;&lt;/b&gt;</p>\n<p>x</p>\n</div>"
        );
    }

    #[test]
    fn slashes_and_apostrophes_stay_readable() {
        assert_eq!(
            render("lozenge", &[("text", "Docs / Guide's")], ""),
            "<span class=\"lozenge default\">Docs / Guide&#39;s</span>"
        );
    }

    #[test]
    fn fallback_div_escapes_attribute_values() {
        let html = render("custom-thing", &[("x", "a\"b")], "");
        assert!(html.contains("data-x=\"a&quot;b\""), "{html}");
    }

    #[test]
    fn template_sees_inline_flag_and_name() {
        let mut registry = ComponentRegistry::with_builtins();
        registry
            .add_template(
                "probe",
                "{{ name }}:{% if inline %}inline{% else %}block{% endif %}",
            )
            .unwrap();
        let html = registry.render_markdown("a :::probe{} b\n\n:::probe\n:::\n", Path::new("t.md"));
        assert!(html.contains("a probe:inline b"), "{html}");
        assert!(html.contains("probe:block"), "{html}");
    }

    #[test]
    fn render_error_shows_escaped_error_box_and_warns() {
        crate::diagnostics::reset_warnings();
        let mut registry = ComponentRegistry::with_builtins();
        registry
            .add_template("needs-title", "<h3>{{ attrs.title }}</h3>")
            .unwrap();
        let html = registry.render_markdown(":::needs-title\n:::\n", Path::new("t.md"));
        assert!(html.contains("class=\"directive-error\""), "{html}");
        assert!(html.contains("needs-title"), "{html}");
        assert!(!html.contains("<h3>"), "{html}");
        assert_eq!(crate::diagnostics::warning_count(), 1);
    }

    #[test]
    fn load_without_component_dir_is_builtins() {
        let dir = tempfile::tempdir().unwrap();
        let registry = ComponentRegistry::load(dir.path()).unwrap();
        let html = registry.render_markdown(":::note\nx\n:::\n", Path::new("t.md"));
        assert!(html.contains("admonition note"));
    }

    #[test]
    fn load_reads_custom_and_override_templates() {
        let dir = tempfile::tempdir().unwrap();
        let comps = dir.path().join("theme/components");
        std::fs::create_dir_all(&comps).unwrap();
        std::fs::write(
            comps.join("card.html"),
            "<div class=\"card\">{{ body | safe }}</div>",
        )
        .unwrap();
        std::fs::write(comps.join("note.html"), "<aside>{{ body | safe }}</aside>").unwrap();
        std::fs::write(comps.join("README.md"), "not a template").unwrap();
        let registry = ComponentRegistry::load(dir.path()).unwrap();
        let html = registry.render_markdown("::::card\n:::note\nx\n:::\n::::\n", Path::new("t.md"));
        assert!(
            html.contains("<div class=\"card\"><aside><p>x</p>\n</aside>"),
            "{html}"
        );
    }

    #[test]
    fn load_resolves_cross_file_import_regardless_of_file_name_order() {
        let dir = tempfile::tempdir().unwrap();
        let comps = dir.path().join("theme/components");
        std::fs::create_dir_all(&comps).unwrap();
        // "badge.html" sorts before "zz-macros.html" alphabetically, so a
        // one-at-a-time load would try to register badge.html (which imports
        // zz-macros.html) before zz-macros.html exists.
        std::fs::write(
            comps.join("badge.html"),
            "{% import \"zz-macros.html\" as m %}<span>{{ m::shout(text=attrs.text) }}</span>",
        )
        .unwrap();
        std::fs::write(
            comps.join("zz-macros.html"),
            "{% macro shout(text) %}{{ text | upper }}{% endmacro shout %}",
        )
        .unwrap();
        let registry = ComponentRegistry::load(dir.path()).unwrap();
        let html = registry.render_markdown(":::badge{text=\"hi\"}\n:::\n", Path::new("t.md"));
        assert!(html.contains("HI"), "{html}");
    }

    #[test]
    fn load_resolves_extends_from_a_user_template() {
        let dir = tempfile::tempdir().unwrap();
        let comps = dir.path().join("theme/components");
        std::fs::create_dir_all(&comps).unwrap();
        std::fs::write(comps.join("alert.html"), "{% extends \"note.html\" %}").unwrap();
        let registry = ComponentRegistry::load(dir.path()).unwrap();
        let html = registry.render_markdown(":::alert\nHi\n:::\n", Path::new("t.md"));
        assert!(html.contains("admonition note"), "{html}");
        assert!(html.contains("Hi"), "{html}");
    }

    #[test]
    fn load_reports_syntax_errors_with_path() {
        let dir = tempfile::tempdir().unwrap();
        let comps = dir.path().join("theme/components");
        std::fs::create_dir_all(&comps).unwrap();
        std::fs::write(comps.join("broken.html"), "{% if %}").unwrap();
        let err = ComponentRegistry::load(dir.path()).err().unwrap();
        match err {
            crate::error::Error::ComponentTemplate { path, .. } => {
                assert!(path.ends_with("theme/components/broken.html"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn add_template_rejects_syntax_errors() {
        let mut registry = ComponentRegistry::with_builtins();
        let err = registry.add_template("broken", "{% if %}").unwrap_err();
        assert!(err.contains("broken.html"), "{err}");
    }

    #[test]
    fn every_builtin_has_a_template_with_a_header() {
        for name in crate::components::templates::builtin_names() {
            let source = crate::components::templates::builtin_source(&name).unwrap();
            assert!(
                source.starts_with("{#"),
                "{name} template should start with a {{# #}} header"
            );
            assert!(
                source.contains("Variables:"),
                "{name} header should list variables"
            );
        }
        assert_eq!(
            crate::components::templates::builtin_names(),
            vec![
                "code-group",
                "lozenge",
                "mermaid",
                "note",
                "tabs",
                "warning"
            ]
        );
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
