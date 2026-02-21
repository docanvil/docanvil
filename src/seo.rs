use crate::project::PageInventory;

/// Generate a `robots.txt` file.
/// Includes a `Sitemap:` directive when an absolute sitemap URL is available.
pub fn generate_robots_txt(sitemap_url: Option<&str>) -> String {
    let mut content = String::from("User-agent: *\nAllow: /\n");
    if let Some(url) = sitemap_url {
        content.push_str(&format!("\nSitemap: {url}\n"));
    }
    content
}

/// Generate a `sitemap.xml` file from the page inventory.
/// Uses `site_url` for absolute URLs when available, otherwise falls back to
/// `base_url` for relative paths.
pub fn generate_sitemap_xml(
    inventory: &PageInventory,
    base_url: &str,
    site_url: Option<&str>,
) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );

    for slug in &inventory.ordered {
        let page = &inventory.pages[slug];
        let path = page.output_path.to_string_lossy().replace('\\', "/");

        let loc = if let Some(url) = site_url {
            format!("{url}{path}")
        } else {
            format!("{base_url}{path}")
        };

        xml.push_str(&format!("  <url>\n    <loc>{loc}</loc>\n  </url>\n"));
    }

    xml.push_str("</urlset>\n");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn robots_txt_without_sitemap() {
        let txt = generate_robots_txt(None);
        assert_eq!(txt, "User-agent: *\nAllow: /\n");
    }

    #[test]
    fn robots_txt_with_sitemap() {
        let txt = generate_robots_txt(Some("https://example.com/sitemap.xml"));
        assert!(txt.contains("User-agent: *"));
        assert!(txt.contains("Allow: /"));
        assert!(txt.contains("Sitemap: https://example.com/sitemap.xml"));
    }

    #[test]
    fn sitemap_xml_with_site_url() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.md"), "# Home").unwrap();
        fs::write(docs.join("guide.md"), "# Guide").unwrap();

        let inv = PageInventory::scan(&docs, None, None).unwrap();
        let xml = generate_sitemap_xml(&inv, "/", Some("https://example.com/"));

        assert!(xml.contains("<loc>https://example.com/guide.html</loc>"));
        assert!(xml.contains("<loc>https://example.com/index.html</loc>"));
        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("<urlset"));
        assert!(xml.ends_with("</urlset>\n"));
    }

    #[test]
    fn sitemap_xml_without_site_url() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.md"), "# Home").unwrap();

        let inv = PageInventory::scan(&docs, None, None).unwrap();
        let xml = generate_sitemap_xml(&inv, "/docs/", None);

        assert!(xml.contains("<loc>/docs/index.html</loc>"));
    }

    #[test]
    fn sitemap_xml_with_nested_pages() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(docs.join("guides")).unwrap();
        fs::write(docs.join("index.md"), "# Home").unwrap();
        fs::write(docs.join("guides/setup.md"), "# Setup").unwrap();

        let inv = PageInventory::scan(&docs, None, None).unwrap();
        let xml = generate_sitemap_xml(&inv, "/", Some("https://example.com/"));

        assert!(xml.contains("<loc>https://example.com/guides/setup.html</loc>"));
    }
}
