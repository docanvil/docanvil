use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::error::Result;

pub fn run(host: &str, port: u16, project_root: &Path, quiet: bool) -> Result<()> {
    let output_dir = dev_output_dir(project_root);
    crate::update::notice::spawn(quiet);

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| crate::error::Error::General(format!("failed to start async runtime: {e}")))?;

    rt.block_on(async { crate::server::start(host, port, &output_dir, project_root).await })
}

/// Where the dev server builds to: a per-project directory under the system
/// temp dir, kept apart from `[build] output_dir`.
///
/// Dev builds inject the live-reload script and serve from `/`, so they must
/// never land in `dist/`, where they could be deployed by mistake.
pub fn dev_output_dir(project_root: &Path) -> PathBuf {
    let root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let mut hasher = DefaultHasher::new();
    root.hash(&mut hasher);
    std::env::temp_dir().join(format!("docanvil-serve-{:016x}", hasher.finish()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_output_dir_is_outside_project() {
        let dir = tempfile::tempdir().unwrap();
        let out = dev_output_dir(dir.path());
        assert!(!out.starts_with(dir.path()));
        assert!(out.starts_with(std::env::temp_dir()));
    }

    #[test]
    fn dev_output_dir_is_stable_per_project() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        assert_eq!(dev_output_dir(a.path()), dev_output_dir(a.path()));
        assert_ne!(dev_output_dir(a.path()), dev_output_dir(b.path()));
    }
}
