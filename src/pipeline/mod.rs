pub mod attributes;
pub mod directives;
pub mod markdown;
pub mod wikilinks;

use std::path::Path;

use crate::components::ComponentRegistry;
use crate::error::Result;
use crate::project::PageInventory;

/// Full pipeline: directives → markdown → wiki-links → attributes.
pub fn process(
    source: &str,
    inventory: &PageInventory,
    source_file: &Path,
    registry: &ComponentRegistry,
) -> Result<String> {
    // 1. Pre-comrak: process directives (:::name{attrs})
    let source = directives::process_directives(source, &mut |block| registry.render_block(block));

    // 2. Render Markdown to HTML
    let html = markdown::render(&source);

    // 3. Resolve wiki-links
    let html = wikilinks::resolve(&html, inventory, source_file);

    // 4. Post-comrak: inject inline attributes ({.class})
    let html = attributes::inject_attributes(&html);

    Ok(html)
}
