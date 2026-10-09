use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

static IMG_SRC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<img\b([^>]*)\bsrc\s*=\s*"([^"]*)"([^>]*)>"#).unwrap());

/// Where a relative image `src` is served from: `base_url` plus the path found at
/// the project root (or under `assets/`), or plus `src` as written if neither
/// exists. `None` for absolute paths, URLs and data URIs, which stay as they are.
pub fn rewrite_src(src: &str, base_url: &str, project_root: &Path) -> Option<String> {
    if src.starts_with('/')
        || src.starts_with("http://")
        || src.starts_with("https://")
        || src.starts_with("data:")
    {
        return None;
    }
    let path = if project_root.join(src).exists() {
        src.to_string()
    } else {
        let assets_path = format!("assets/{src}");
        if project_root.join(&assets_path).exists() {
            assets_path
        } else {
            src.to_string()
        }
    };
    Some(format!("{base_url}{path}"))
}

/// Rewrite relative `<img src="...">` paths in rendered HTML to include the base URL
/// (see [`rewrite_src`]).
pub fn rewrite_image_paths(html: &str, base_url: &str, project_root: &Path) -> String {
    IMG_SRC_RE
        .replace_all(html, |caps: &regex::Captures| {
            match rewrite_src(&caps[2], base_url, project_root) {
                Some(src) => format!(r#"<img{}src="{src}"{}>"#, &caps[1], &caps[3]),
                None => caps[0].to_string(),
            }
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rewrites_relative_path_with_base_url() {
        let dir = tempfile::tempdir().unwrap();
        let assets = dir.path().join("assets");
        fs::create_dir_all(&assets).unwrap();
        fs::write(assets.join("diagram.png"), b"fake").unwrap();

        let html = r#"<p><img src="assets/diagram.png" alt="diagram"></p>"#;
        let result = rewrite_image_paths(html, "/docs/", dir.path());
        assert_eq!(
            result,
            r#"<p><img src="/docs/assets/diagram.png" alt="diagram"></p>"#
        );
    }

    #[test]
    fn resolves_bare_filename_under_assets() {
        let dir = tempfile::tempdir().unwrap();
        let assets = dir.path().join("assets");
        fs::create_dir_all(&assets).unwrap();
        fs::write(assets.join("photo.jpg"), b"fake").unwrap();

        let html = r#"<img src="photo.jpg" alt="photo">"#;
        let result = rewrite_image_paths(html, "/", dir.path());
        assert_eq!(result, r#"<img src="/assets/photo.jpg" alt="photo">"#);
    }

    #[test]
    fn skips_absolute_paths() {
        let dir = tempfile::tempdir().unwrap();
        let html = r#"<img src="/absolute/image.png" alt="abs">"#;
        let result = rewrite_image_paths(html, "/docs/", dir.path());
        assert_eq!(result, html);
    }

    #[test]
    fn skips_http_urls() {
        let dir = tempfile::tempdir().unwrap();
        let html = r#"<img src="https://example.com/img.png" alt="ext">"#;
        let result = rewrite_image_paths(html, "/docs/", dir.path());
        assert_eq!(result, html);
    }

    #[test]
    fn skips_data_uris() {
        let dir = tempfile::tempdir().unwrap();
        let html = r#"<img src="data:image/png;base64,abc" alt="data">"#;
        let result = rewrite_image_paths(html, "/docs/", dir.path());
        assert_eq!(result, html);
    }

    #[test]
    fn rewrites_missing_file_with_base_url() {
        let dir = tempfile::tempdir().unwrap();
        let html = r#"<img src="missing.png" alt="gone">"#;
        let result = rewrite_image_paths(html, "/sub/", dir.path());
        assert_eq!(result, r#"<img src="/sub/missing.png" alt="gone">"#);
    }

    #[test]
    fn handles_root_base_url() {
        let dir = tempfile::tempdir().unwrap();
        let assets = dir.path().join("assets");
        fs::create_dir_all(&assets).unwrap();
        fs::write(assets.join("logo.svg"), b"fake").unwrap();

        let html = r#"<img src="assets/logo.svg" alt="logo">"#;
        let result = rewrite_image_paths(html, "/", dir.path());
        assert_eq!(result, r#"<img src="/assets/logo.svg" alt="logo">"#);
    }

    #[test]
    fn handles_multiple_images() {
        let dir = tempfile::tempdir().unwrap();
        let assets = dir.path().join("assets");
        fs::create_dir_all(&assets).unwrap();
        fs::write(assets.join("a.png"), b"fake").unwrap();
        fs::write(assets.join("b.png"), b"fake").unwrap();

        let html = r#"<img src="assets/a.png" alt="a"><img src="assets/b.png" alt="b">"#;
        let result = rewrite_image_paths(html, "/base/", dir.path());
        assert_eq!(
            result,
            r#"<img src="/base/assets/a.png" alt="a"><img src="/base/assets/b.png" alt="b">"#
        );
    }

    #[test]
    fn rewrite_src_resolves_like_html_images() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("assets")).unwrap();
        fs::write(dir.path().join("assets/logo.png"), b"x").unwrap();

        assert_eq!(
            rewrite_src("logo.png", "https://x.dev/", dir.path()).as_deref(),
            Some("https://x.dev/assets/logo.png")
        );
        assert_eq!(
            rewrite_src("missing.png", "/docs/", dir.path()).as_deref(),
            Some("/docs/missing.png")
        );
        for src in [
            "/abs.png",
            "http://e.com/a.png",
            "https://e.com/a.png",
            "data:image/png;base64,x",
        ] {
            assert_eq!(rewrite_src(src, "/", dir.path()), None);
        }
    }
}
