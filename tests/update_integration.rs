use std::sync::mpsc;

use axum::Router;
use axum::http::{StatusCode, header};
use axum::routing::get;
use docanvil::error::Error;
use docanvil::update::{
    Source, download_verified, ensure_release_exists, latest_version, sha256_hex,
};
use semver::Version;

const TARGET: &str = "x86_64-unknown-linux-gnu";
const ARCHIVE: &[u8] = b"pretend this is a tarball";

/// Serve `router` on an ephemeral port from a background runtime and return
/// its base URL. The update code is blocking, so tests stay plain `#[test]`s.
fn serve(router: Router) -> String {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            tx.send(listener.local_addr().unwrap()).unwrap();
            axum::serve(listener, router).await.unwrap();
        });
    });
    format!("http://{}", rx.recv().unwrap())
}

fn source(base: &str) -> Source {
    Source {
        web_base: base.to_string(),
        api_base: format!("{base}/api"),
    }
}

fn asset() -> String {
    format!("docanvil-v1.2.0-{TARGET}.tar.gz")
}

fn release_router(sums: String) -> Router {
    Router::new()
        .route(
            "/releases/latest",
            get(|| async {
                (
                    StatusCode::FOUND,
                    [(header::LOCATION, "/releases/tag/v1.2.0")],
                )
            }),
        )
        .route(
            &format!("/releases/download/v1.2.0/{}", asset()),
            get(|| async { ARCHIVE.to_vec() }),
        )
        .route(
            "/releases/download/v1.2.0/SHA256SUMS",
            get(move || async move { sums }),
        )
}

#[test]
fn latest_version_reads_redirect() {
    let base = serve(release_router(String::new()));
    assert_eq!(
        latest_version(&source(&base), docanvil::update::CHECK_TIMEOUT).unwrap(),
        Version::new(1, 2, 0)
    );
}

#[test]
fn download_verified_accepts_matching_checksum() {
    let sums = format!("{}  {}\n", sha256_hex(ARCHIVE), asset());
    let base = serve(release_router(sums));
    let bytes = download_verified(&source(&base), &Version::new(1, 2, 0), TARGET).unwrap();
    assert_eq!(bytes, ARCHIVE);
}

#[test]
fn download_verified_rejects_checksum_mismatch() {
    let sums = format!("{}  {}\n", "0".repeat(64), asset());
    let base = serve(release_router(sums));
    let err = download_verified(&source(&base), &Version::new(1, 2, 0), TARGET).unwrap_err();
    assert!(matches!(err, Error::Update { .. }));
    assert!(err.to_string().contains("checksum mismatch"), "{err}");
}

#[test]
fn download_verified_falls_back_to_api_digest() {
    // Releases up to v1.1.3 have no SHA256SUMS; GitHub's asset digest is used.
    let digest = format!("sha256:{}", sha256_hex(ARCHIVE));
    let json = format!(
        r#"{{"assets":[{{"name":"{}","digest":"{digest}"}}]}}"#,
        asset()
    );
    let router = Router::new()
        .route(
            &format!("/releases/download/v1.2.0/{}", asset()),
            get(|| async { ARCHIVE.to_vec() }),
        )
        .route(
            "/api/releases/tags/v1.2.0",
            get(move || async move { json }),
        );
    let base = serve(router);
    let bytes = download_verified(&source(&base), &Version::new(1, 2, 0), TARGET).unwrap();
    assert_eq!(bytes, ARCHIVE);
}

#[test]
fn download_verified_reports_missing_release() {
    let base = serve(Router::new());
    let err = download_verified(&source(&base), &Version::new(9, 9, 9), TARGET).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("v9.9.9") && msg.contains("not found"), "{msg}");
}

#[test]
fn unreachable_server_gives_connection_hint() {
    // Bind then drop to get a port that refuses connections.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let err = latest_version(
        &source(&format!("http://127.0.0.1:{port}")),
        docanvil::update::CHECK_TIMEOUT,
    )
    .unwrap_err();
    assert!(matches!(err, Error::Update { .. }));
    assert!(err.hint().unwrap().contains("internet connection"));
}

#[test]
fn ensure_release_exists_rejects_missing_release() {
    let base = serve(Router::new());
    let err = ensure_release_exists(&source(&base), &Version::new(9, 9, 9)).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("v9.9.9") && msg.contains("not found"), "{msg}");
}

#[test]
fn ensure_release_exists_accepts_release_with_sums() {
    let base = serve(release_router(String::new()));
    ensure_release_exists(&source(&base), &Version::new(1, 2, 0)).unwrap();
}

#[test]
fn ensure_release_exists_accepts_older_release_via_api() {
    let router = Router::new().route(
        "/api/releases/tags/v1.1.0",
        get(|| async { r#"{"assets":[]}"# }),
    );
    let base = serve(router);
    ensure_release_exists(&source(&base), &Version::new(1, 1, 0)).unwrap();
}
