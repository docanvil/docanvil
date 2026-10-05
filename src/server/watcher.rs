use std::path::{Path, PathBuf};
use std::time::Duration;

use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use tokio::sync::broadcast;

use crate::config::Config;

/// What the dev server watches for changes.
struct WatchSet {
    /// The project root, watched non-recursively for config and nav files.
    root: PathBuf,
    /// Directories watched recursively: content, theme, assets, static.
    dirs: Vec<PathBuf>,
}

impl WatchSet {
    fn new(project_root: &Path) -> Self {
        // Event paths are canonical on some platforms (e.g. /private/var on
        // macOS), so compare against canonical paths.
        let canonical = |p: PathBuf| p.canonicalize().unwrap_or(p);
        let root = canonical(project_root.to_path_buf());

        let content_dir = Config::load(project_root)
            .map(|c| c.project.content_dir)
            .unwrap_or_else(|_| PathBuf::from("docs"));
        let dirs = [
            content_dir.as_path(),
            Path::new("theme"),
            Path::new("assets"),
            Path::new("static"),
        ]
        .iter()
        .map(|d| root.join(d))
        .filter(|d| d.is_dir())
        .map(canonical)
        .collect();

        Self { root, dirs }
    }

    /// Whether a change to `path` should trigger a rebuild.
    fn is_relevant(&self, path: &Path) -> bool {
        if self.dirs.iter().any(|d| path.starts_with(d)) {
            return true;
        }
        // Root-level files: docanvil.toml and nav.toml / nav.{locale,version}.toml.
        // Anything else in the root (dist/, staging dirs, editor files) is ignored.
        if path.parent() == Some(self.root.as_path())
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
        {
            return name == "docanvil.toml" || (name.starts_with("nav") && name.ends_with(".toml"));
        }
        false
    }
}

/// Watch for file changes and trigger rebuilds.
pub fn watch(
    tx: broadcast::Sender<()>,
    project_root: &Path,
    output_dir: &Path,
) -> crate::error::Result<()> {
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(200), notify_tx)
        .map_err(|e| crate::error::Error::General(format!("watcher setup failed: {e}")))?;

    let watch_set = WatchSet::new(project_root);
    for dir in &watch_set.dirs {
        let _ = debouncer
            .watcher()
            .watch(dir, notify::RecursiveMode::Recursive);
    }
    let _ = debouncer
        .watcher()
        .watch(&watch_set.root, notify::RecursiveMode::NonRecursive);

    eprintln!("Watching for changes...");

    loop {
        match notify_rx.recv() {
            Ok(Ok(events)) => {
                let has_changes = events
                    .iter()
                    .any(|e| e.kind == DebouncedEventKind::Any && watch_set.is_relevant(&e.path));

                if has_changes {
                    eprintln!("Change detected, rebuilding...");
                    match crate::cli::build::run_with_options(project_root, output_dir, true) {
                        Ok(()) => {
                            let _ = tx.send(());
                        }
                        Err(e) => {
                            eprintln!("rebuild error: {e}");
                        }
                    }
                }
            }
            Ok(Err(error)) => {
                eprintln!("watch error: {error}");
            }
            Err(_) => break,
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn project(config: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("docanvil.toml"), config).unwrap();
        dir
    }

    #[test]
    fn watches_custom_content_dir() {
        let dir = project("[project]\nname = \"T\"\ncontent_dir = \"content\"\n");
        fs::create_dir_all(dir.path().join("content/guides")).unwrap();
        fs::create_dir_all(dir.path().join("theme")).unwrap();
        let set = WatchSet::new(dir.path());

        assert!(set.is_relevant(&set.root.join("content/guides/setup.md")));
        assert!(set.is_relevant(&set.root.join("theme/custom.css")));
        assert!(!set.is_relevant(&set.root.join("docs/index.md")));
    }

    #[test]
    fn watches_config_and_all_nav_files() {
        let dir = project("[project]\nname = \"T\"\n");
        let set = WatchSet::new(dir.path());

        for name in ["docanvil.toml", "nav.toml", "nav.fr.toml", "nav.v2.toml"] {
            assert!(set.is_relevant(&set.root.join(name)), "{name}");
        }
    }

    #[test]
    fn ignores_output_and_other_root_files() {
        let dir = project("[project]\nname = \"T\"\n");
        let set = WatchSet::new(dir.path());

        assert!(!set.is_relevant(&set.root.join("dist")));
        assert!(!set.is_relevant(&set.root.join("dist/index.html")));
        assert!(!set.is_relevant(&set.root.join(".dist.docanvil-staging")));
        assert!(!set.is_relevant(&set.root.join("README.md")));
    }
}
