pub mod attributes;
pub mod directives;
pub mod frontmatter;
pub mod headings;
pub mod images;
pub mod includes;
pub mod markdown;
pub mod popovers;
pub mod syntax;
pub mod wikilinks;

use std::path::Path;

use crate::components::ComponentRegistry;
use crate::error::Result;
use crate::project::PageInventory;

use self::syntax::SyntaxHighlighter;

/// Full pipeline: directives + popovers + markdown (via the component registry) → syntax highlight → wiki-links → attributes → heading IDs → image paths.
/// When `locale` is provided, wiki-links resolve within that locale only.
#[allow(clippy::too_many_arguments)]
pub fn process(
    source: &str,
    inventory: &PageInventory,
    source_file: &Path,
    registry: &ComponentRegistry,
    base_url: &str,
    highlighter: Option<&SyntaxHighlighter>,
    project_root: &Path,
    locale: Option<&str>,
) -> Result<String> {
    // 1. Directives (nested components), popovers, {#id} extraction, then comrak
    let html = registry.render_markdown(source, source_file);

    // 2. Syntax-highlight code blocks (if enabled)
    let html = match highlighter {
        Some(h) => syntax::highlight_code_blocks(&html, h),
        None => html,
    };

    // 3. Resolve wiki-links
    let html = wikilinks::resolve(&html, inventory, source_file, base_url, locale);

    // 4. Post-comrak: inject inline attributes ({.class})
    let html = attributes::inject_attributes(&html);

    // 5. Auto-generate heading IDs (after attributes so manual {#id} wins)
    let html = headings::inject_heading_ids(&html);

    // 6. Rewrite relative image paths with base_url
    let html = images::rewrite_image_paths(&html, base_url, project_root);

    Ok(html)
}
