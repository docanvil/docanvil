use std::path::Path;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use tokio::sync::broadcast;

/// Watch for file changes and trigger rebuilds.
pub fn watch(tx: broadcast::Sender<()>, output_dir: &Path) -> crate::error::Result<()> {
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(200), notify_tx)
        .map_err(|e| crate::error::Error::Render(format!("watcher setup failed: {e}")))?;

    // Watch the docs/ and theme/ directories
    let watch_dirs = ["docs", "theme", "docanvil.toml", "nav.toml"];
    for dir in &watch_dirs {
        let path = Path::new(dir);
        if path.exists() {
            let mode = if path.is_dir() {
                notify::RecursiveMode::Recursive
            } else {
                notify::RecursiveMode::NonRecursive
            };
            let _ = debouncer.watcher().watch(path, mode);
        }
    }

    eprintln!("Watching for changes...");

    loop {
        match notify_rx.recv() {
            Ok(Ok(events)) => {
                let has_changes = events.iter().any(|e| e.kind == DebouncedEventKind::Any);

                if has_changes {
                    eprintln!("Change detected, rebuilding...");
                    match crate::cli::build::run_with_options(output_dir, true) {
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
