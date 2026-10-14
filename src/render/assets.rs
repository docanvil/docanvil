use std::path::Path;

use crate::error::{Error, Result};

/// Copy a version's content assets without allowing its static files to
/// replace pages, search indexes or redirects already generated for it.
pub fn copy_version_assets(project_root: &Path, output_dir: &Path) -> Result<()> {
    for path in static_output_paths(project_root)? {
        if output_dir.join(&path).exists() {
            return Err(Error::General(format!(
                "Version static file '{path}' would overwrite generated documentation"
            )));
        }
    }
    copy_assets(project_root, output_dir, None)
}

/// Paths copied from `static/`, relative to the output root. Follow directories
/// the same way as the asset copier, including directory symlinks.
pub fn static_output_paths(project_root: &Path) -> Result<std::collections::HashSet<String>> {
    fn collect(
        dir: &Path,
        root: &Path,
        paths: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                collect(&path, root, paths)?;
            } else if let Ok(relative) = path.strip_prefix(root) {
                paths.insert(relative.to_string_lossy().replace('\\', "/"));
            }
        }
        Ok(())
    }
    let root = project_root.join("static");
    let mut paths = std::collections::HashSet::new();
    if root.exists() {
        collect(&root, &root, &mut paths)?;
    }
    Ok(paths)
}

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
        let Ok(relative) = path.strip_prefix(src) else {
            crate::diagnostics::warn_unexpected_asset_path(&path);
            continue;
        };
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
