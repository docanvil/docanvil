pub mod watcher;
pub mod websocket;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use axum::Router;
use tokio::sync::broadcast;
use tower_http::services::{ServeDir, ServeFile};

use crate::edit::Editor;
use crate::error::Result;

/// Start the dev server with file watching and hot reload. With an `editor`,
/// each page's edit link opens its source in that editor.
pub async fn start(
    host: &str,
    port: u16,
    output_dir: &Path,
    project_root: &Path,
    editor: Option<Editor>,
) -> Result<()> {
    let (tx, _rx) = broadcast::channel::<()>(16);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| crate::error::Error::General(format!("invalid address: {e}")))?;

    // Initial build with live_reload enabled
    let dependencies =
        crate::cli::build::run_with_options(project_root, output_dir, true, editor.as_ref())?;

    // Start file watcher
    let tx_clone = tx.clone();
    let watch_root = project_root.to_path_buf();
    let watch_output = output_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = watcher::watch(
            tx_clone,
            &watch_root,
            &watch_output,
            dependencies,
            editor.as_ref(),
        ) {
            eprintln!("watcher error: {e}");
        }
    });

    // Build axum router
    let app = build_router(tx, output_dir.to_path_buf());

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
        // Unknown paths get the generated 404.html with a 404 status, as static
        // hosts do in production.
        .fallback_service(
            ServeDir::new(&output_dir)
                .not_found_service(ServeFile::new(output_dir.join("404.html"))),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn site() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("index.html"), "home").unwrap();
        std::fs::write(dir.path().join("404.html"), "custom not found").unwrap();
        dir
    }

    async fn get(dir: &Path, uri: &str) -> (StatusCode, String) {
        let (tx, _rx) = broadcast::channel(1);
        let response = build_router(tx, dir.to_path_buf())
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8_lossy(&body).into_owned())
    }

    #[tokio::test]
    async fn serves_existing_pages() {
        let dir = site();
        assert_eq!(
            get(dir.path(), "/index.html").await,
            (StatusCode::OK, "home".into())
        );
    }

    #[tokio::test]
    async fn missing_pages_get_custom_404() {
        let dir = site();
        assert_eq!(
            get(dir.path(), "/no-such-page.html").await,
            (StatusCode::NOT_FOUND, "custom not found".into())
        );
    }
}
