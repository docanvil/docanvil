use std::fs;
use std::path::Path;

use tempfile::TempDir;

/// Minimal docanvil.toml that produces a working build.
pub const DEFAULT_CONFIG: &str = r#"
[project]
name = "Test Docs"
"#;

/// Create a temporary project with a config and a set of markdown pages.
///
/// Each page is a tuple of `(filename, markdown_content)` where filename is
/// relative to the content directory (e.g. `"index.md"` or `"guide/setup.md"`).
pub fn create_project(config_toml: &str, pages: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("failed to create tempdir");
    let root = dir.path();

    // Write config
    fs::write(root.join("docanvil.toml"), config_toml).expect("failed to write config");

    // Write pages into docs/
    let docs = root.join("docs");
    for (filename, content) in pages {
        let path = docs.join(filename);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create page parent dir");
        }
        fs::write(&path, content).expect("failed to write page");
    }

    dir
}

/// Run the build pipeline on a project directory.
pub fn build_project(dir: &Path) -> docanvil::error::Result<()> {
    let out = dir.join("dist");
    docanvil::cli::build::run(dir, &out, false, true, false)
}

/// Run the build pipeline in strict mode.
pub fn build_project_strict(dir: &Path) -> docanvil::error::Result<()> {
    let out = dir.join("dist");
    docanvil::cli::build::run(dir, &out, false, true, true)
}

/// Read a file from the build output directory.
pub fn read_output(dir: &Path, path: &str) -> String {
    let full = dir.join("dist").join(path);
    fs::read_to_string(&full)
        .unwrap_or_else(|e| panic!("failed to read output file {}: {e}", full.display()))
}

/// Check whether a file exists in the build output directory.
pub fn output_exists(dir: &Path, path: &str) -> bool {
    dir.join("dist").join(path).exists()
}
