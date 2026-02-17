use std::path::Path;
use std::time::Duration;

use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use tokio::sync::broadcast;

/// Watch for file changes and trigger rebuilds.
pub fn watch(tx: broadcast::Sender<()>, project_root: &Path) -> crate::error::Result<()> {
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(200), notify_tx)
        .map_err(|e| crate::error::Error::General(format!("watcher setup failed: {e}")))?;

    // Watch the docs/ and theme/ directories
    let watch_dirs = ["docs", "theme", "assets", "docanvil.toml", "nav.toml"];
    for dir in &watch_dirs {
        let path = project_root.join(dir);
        if path.exists() {
            let mode = if path.is_dir() {
                notify::RecursiveMode::Recursive
            } else {
                notify::RecursiveMode::NonRecursive
            };
            let _ = debouncer.watcher().watch(&path, mode);
        }
    }

    eprintln!("Watching for changes...");

    loop {
        match notify_rx.recv() {
            Ok(Ok(events)) => {
                let has_changes = events.iter().any(|e| e.kind == DebouncedEventKind::Any);

                if has_changes {
                    eprintln!("Change detected, rebuilding...");
                    match crate::cli::build::run_with_options(project_root, true) {
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
