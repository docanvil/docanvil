pub mod watcher;
pub mod websocket;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use axum::Router;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

use crate::error::Result;

/// Start the dev server with file watching and hot reload.
pub async fn start(host: &str, port: u16, output_dir: &Path, project_root: &Path) -> Result<()> {
    let (tx, _rx) = broadcast::channel::<()>(16);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| crate::error::Error::Render(format!("invalid address: {e}")))?;

    // Initial build with live_reload enabled
    let out = output_dir.to_path_buf();
    crate::cli::build::run_with_options(project_root, &out, true)?;

    // Start file watcher
    let tx_clone = tx.clone();
    let watch_out = out.clone();
    let watch_root = project_root.to_path_buf();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = watcher::watch(tx_clone, &watch_out, &watch_root) {
            eprintln!("watcher error: {e}");
        }
    });

    // Build axum router
    let app = build_router(tx, out);

    eprintln!("Serving at http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(crate::error::Error::Io)?;
    axum::serve(listener, app)
        .await
        .map_err(crate::error::Error::Io)?;

    Ok(())
}

fn build_router(tx: broadcast::Sender<()>, output_dir: PathBuf) -> Router {
    Router::new()
        .route(
            "/__docanvil_ws",
            axum::routing::get(move |ws| websocket::handler(ws, tx)),
        )
        .fallback_service(ServeDir::new(output_dir))
}
