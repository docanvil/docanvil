//! Remote code sources. Cache lifetime is one synchronous build/doctor/export run.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Duration;

const MAX_BYTES: u64 = 2 * 1024 * 1024;
type Cache = BTreeMap<String, Result<String, String>>;
thread_local! {
    static CACHE: RefCell<Option<Cache>> = const { RefCell::new(None) };
}

/// A fresh request cache, restored on exit (including errors and panics).
/// Builds currently render pages synchronously on the calling thread.
pub(crate) struct Scope(Option<Cache>);
impl Scope {
    pub(crate) fn new() -> Self {
        Self(CACHE.with(|cache| cache.replace(Some(BTreeMap::new()))))
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        CACHE.with(|cache| cache.replace(self.0.take()));
    }
}

pub(super) fn validate_url(url: &str) -> Result<ureq::http::Uri, String> {
    let uri = url
        .parse::<ureq::http::Uri>()
        .map_err(|_| "url must be an absolute HTTPS URL".to_string())?;
    if uri.scheme_str() != Some("https")
        || uri.host().is_none()
        || uri.authority().is_some_and(|a| a.as_str().contains('@'))
        || url.contains('#')
    {
        return Err(
            "url must be an absolute HTTPS URL without credentials or a fragment".to_string(),
        );
    }
    Ok(uri)
}

pub(super) fn fetch(url: &str) -> Result<String, String> {
    cached(url, || {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .https_only(true)
            .timeout_global(Some(Duration::from_secs(15)))
            .timeout_connect(Some(Duration::from_secs(5)))
            .max_redirects(5)
            .user_agent(concat!("docanvil/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();
        download(&agent, url)
    })
}

fn cached(url: &str, load: impl FnOnce() -> Result<String, String>) -> Result<String, String> {
    if let Some(found) =
        CACHE.with(|cache| cache.borrow().as_ref().and_then(|c| c.get(url).cloned()))
    {
        return found;
    }
    let result = load();
    CACHE.with(|cache| {
        if let Some(cache) = cache.borrow_mut().as_mut() {
            cache.insert(url.to_string(), result.clone());
        }
    });
    result
}

fn download(agent: &ureq::Agent, url: &str) -> Result<String, String> {
    let mut response = agent
        .get(url)
        .call()
        .map_err(|e| format!("can't fetch {url}: {e}"))?;
    // Repository browsing URLs often return HTML with a successful status.
    if response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(';').next().is_some_and(|t| {
                t.trim().eq_ignore_ascii_case("text/html")
                    || t.trim().eq_ignore_ascii_case("application/xhtml+xml")
            })
        })
    {
        return Err(format!("{url} returned HTML; use a raw file URL"));
    }
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_BYTES)
        .read_to_vec()
        .map_err(|e| format!("can't read {url} (maximum 2 MiB): {e}"))?;
    let text = String::from_utf8(bytes).map_err(|_| format!("{url} isn't valid UTF-8"))?;
    Ok(text.trim_start_matches('\u{feff}').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    fn only_absolute_https_source_urls() {
        assert!(validate_url("https://example.com/file.rs?raw=1").is_ok());
        for url in [
            "http://example.com/a",
            "file:///a",
            "a.rs",
            "https://u:p@example.com/a",
            "https://example.com/a#L1",
        ] {
            assert!(validate_url(url).is_err(), "{url}");
        }
    }

    #[test]
    fn cache_reuses_successes_and_failures_and_expires_with_scope() {
        for result in [Ok("source".to_string()), Err("unavailable".to_string())] {
            let scope = Scope::new();
            assert_eq!(cached("https://example.com/a", || result.clone()), result);
            assert_eq!(
                cached("https://example.com/a", || panic!("duplicate request")),
                result
            );
            drop(scope);
            assert_eq!(
                cached("https://example.com/a", || Ok("fresh".into())),
                Ok("fresh".into())
            );
        }
    }

    #[test]
    fn remote_regions_use_cached_source_and_original_line_numbers() {
        let _scope = Scope::new();
        let url = "https://example.com/regions.rs";
        cached(url, || Ok("before\n// {== region=\"sample\" ==}\n    let x = 1;\n// {== endregion=\"sample\" ==}\nafter\n".into())).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let out = super::super::expand(
            &format!("```rust url=\"{url}\" region=\"sample\"\n```\n"),
            &dir.path().join("page.md"),
            &super::super::IncludeContext {
                project_root: dir.path(),
                locale: None,
            },
        );
        assert!(out.problems.is_empty(), "{:?}", out.problems);
        assert!(out.source.contains("ranges=3-3"));
        assert!(out.source.contains("\nlet x = 1;\n"));
        assert!(!out.source.contains("{=="));
    }

    #[test]
    fn remote_attribute_errors_keep_authored_code() {
        let dir = tempfile::tempdir().unwrap();
        let out = super::super::expand(
            "```rust file=\"local.rs\" url=\"https://example.com/a.rs\"\n```\n```rust url=\"https://example.com/a.rs\"\nlet x = 1;\n```\n``` url=\"http://example.com/a\"\n```\n",
            &dir.path().join("page.md"),
            &super::super::IncludeContext {
                project_root: dir.path(),
                locale: None,
            },
        );
        assert_eq!(out.problems.len(), 3);
        assert!(
            out.problems
                .iter()
                .all(|p| p.check == super::super::CHECK_INVALID)
        );
        assert!(out.source.contains("\nlet x = 1;\n```"));
        assert!(out.problems[0].message.contains("both"));
    }

    fn serve(
        status: &str,
        content_type: &str,
        body: &[u8],
    ) -> (String, std::thread::JoinHandle<()>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/source.rs", listener.local_addr().unwrap());
        let response = [format!("HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).into_bytes(), body.to_vec()].concat();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            let read = stream.read(&mut request).unwrap();
            assert!(read > 0);
            let _ = stream.write_all(&response);
        });
        (url, handle)
    }

    #[test]
    fn http_responses_are_bounded_and_validated() {
        let oversized = vec![b'x'; MAX_BYTES as usize + 1];
        for (status, kind, body, expected) in [
            (
                "200 OK",
                "text/plain",
                b"\xef\xbb\xbffn main() {}\n".as_slice(),
                "fn main() {}\n",
            ),
            ("404 Not Found", "text/plain", b"missing".as_slice(), "404"),
            (
                "200 OK",
                "text/html; charset=utf-8",
                b"<html>".as_slice(),
                "raw file URL",
            ),
            ("200 OK", "text/plain", b"\xff".as_slice(), "UTF-8"),
            (
                "200 OK",
                "text/plain",
                oversized.as_slice(),
                "maximum 2 MiB",
            ),
        ] {
            let (url, handle) = serve(status, kind, body);
            let result = download(&ureq::Agent::new_with_defaults(), &url);
            handle.join().unwrap();
            if expected.starts_with("fn") {
                assert_eq!(result.unwrap(), expected);
            } else {
                assert!(result.unwrap_err().contains(expected));
            }
        }
    }

    #[test]
    fn remote_sources_use_existing_selection_and_captions() {
        let _scope = Scope::new();
        let url = "https://example.com/source.rs?raw=1";
        cached(
            url,
            || Ok("fn main() {\n    one();\n    two();\n}\n".into()),
        )
        .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let out = super::super::expand(
            &format!("``` url=\"{url}\" lines=\"2-3\" title=\"Example\"\n```\n"),
            &dir.path().join("page.md"),
            &super::super::IncludeContext {
                project_root: dir.path(),
                locale: None,
            },
        );
        assert!(out.problems.is_empty(), "{:?}", out.problems);
        assert!(out.dependencies.is_empty());
        assert!(
            out.source.starts_with("```rust docanvil ranges=2-3"),
            "{}",
            out.source
        );
        assert!(out.source.contains("\none();\ntwo();\n```"));
        let html = crate::pipeline::code_blocks::process(
            &crate::pipeline::markdown::render(&out.source),
            false,
        );
        assert!(html.contains("Example"));
        assert!(html.contains("data-line=\"2\""));
    }
}
