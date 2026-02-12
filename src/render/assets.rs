use std::path::Path;

use crate::error::Result;

/// Copy static assets (custom CSS, images, etc.) from the project to the output directory.
pub fn copy_assets(project_root: &Path, output_dir: &Path, custom_css: Option<&str>) -> Result<()> {
    // Copy custom CSS if specified
    if let Some(css_path) = custom_css {
        let src = project_root.join(css_path);
        if src.exists() {
            let dest = output_dir.join(css_path);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&src, &dest)?;
        }
    }

    // Copy any files from a static/ directory if it exists
    let static_dir = project_root.join("static");
    if static_dir.exists() {
        copy_dir_recursive(&static_dir, output_dir)?;
    }

    // Copy assets/ directory (preserving it as a subdirectory)
    let assets_dir = project_root.join("assets");
    if assets_dir.exists() {
        let dest = output_dir.join("assets");
        std::fs::create_dir_all(&dest)?;
        copy_dir_recursive(&assets_dir, &dest)?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(src).unwrap();
        let target = dest.join(relative);

        if path.is_dir() {
            std::fs::create_dir_all(&target)?;
            copy_dir_recursive(&path, &dest.join(relative))?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&path, &target)?;
        }
    }
    Ok(())
}
