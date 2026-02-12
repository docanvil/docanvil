use std::path::Path;

use regex::Regex;

/// Rewrite relative `<img src="...">` paths in rendered HTML to include the base URL.
///
/// Skips absolute paths (`/`), URLs (`http://`, `https://`), and data URIs (`data:`).
/// For relative paths, checks if the file exists at project root; if not, tries under `assets/`.
pub fn rewrite_image_paths(html: &str, base_url: &str, project_root: &Path) -> String {
    let re = Regex::new(r#"<img\b([^>]*)\bsrc\s*=\s*"([^"]*)"([^>]*)>"#).unwrap();

    re.replace_all(html, |caps: &regex::Captures| {
        let before = &caps[1];
        let src = &caps[2];
        let after = &caps[3];

        if src.starts_with('/')
            || src.starts_with("http://")
            || src.starts_with("https://")
            || src.starts_with("data:")
        {
            return caps[0].to_string();
        }

        // Check if the path exists directly relative to project root
        let resolved = if project_root.join(src).exists() {
            Some(src.to_string())
        } else {
            // Try under assets/ as a fallback
            let assets_path = format!("assets/{src}");
            if project_root.join(&assets_path).exists() {
                Some(assets_path)
            } else {
                None
            }
        };

        match resolved {
            Some(path) => {
                format!(r#"<img{before}src="{base_url}{path}"{after}>"#)
            }
            None => {
                // Path not found — still prepend base_url to the original src
                format!(r#"<img{before}src="{base_url}{src}"{after}>"#)
            }
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
}
