pub mod attributes;
pub mod directives;
pub mod markdown;
pub mod popovers;
pub mod syntax;
pub mod wikilinks;

use std::path::Path;

use crate::components::ComponentRegistry;
use crate::error::Result;
use crate::project::PageInventory;

use self::syntax::SyntaxHighlighter;

/// Full pipeline: directives → popovers → markdown → syntax highlight → wiki-links → attributes.
pub fn process(
    source: &str,
    inventory: &PageInventory,
    source_file: &Path,
    registry: &ComponentRegistry,
    base_url: &str,
    highlighter: Option<&SyntaxHighlighter>,
) -> Result<String> {
    // 1. Pre-comrak: process directives (:::name{attrs})
    let source = directives::process_directives(source, &mut |block| registry.render_block(block));

    // 2. Pre-comrak: convert ^[content] to popover spans
    let source = popovers::process_popovers(&source);

    // 3. Render Markdown to HTML
    let html = markdown::render(&source);

    // 4. Syntax-highlight code blocks (if enabled)
    let html = match highlighter {
        Some(h) => syntax::highlight_code_blocks(&html, h),
        None => html,
    };

    // 5. Resolve wiki-links
    let html = wikilinks::resolve(&html, inventory, source_file, base_url);

    // 6. Post-comrak: inject inline attributes ({.class})
    let html = attributes::inject_attributes(&html);

    Ok(html)
}
