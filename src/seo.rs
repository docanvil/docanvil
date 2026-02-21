use std::collections::{HashMap, HashSet};

use crate::project::PageInventory;

/// Locale configuration for sitemap hreflang annotations.
pub struct SitemapLocaleConfig {
    pub enabled: Vec<String>,
    pub default_locale: String,
    pub slug_coverage: HashMap<String, HashSet<String>>,
}

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
/// `base_url` for relative paths. When `locale_config` is provided, emits
/// `xhtml:link` hreflang annotations for multilingual pages.
pub fn generate_sitemap_xml(
    inventory: &PageInventory,
    base_url: &str,
    site_url: Option<&str>,
    locale_config: Option<&SitemapLocaleConfig>,
) -> String {
    let has_i18n = locale_config.is_some();
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    if has_i18n {
        xml.push_str(
            "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"\n\
             \x20       xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
        );
    } else {
        xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    }

    for slug in &inventory.ordered {
        let page = &inventory.pages[slug];
        let path = page.output_path.to_string_lossy().replace('\\', "/");

        let loc = if let Some(url) = site_url {
            format!("{url}{path}")
        } else {
            format!("{base_url}{path}")
        };

        xml.push_str(&format!("  <url>\n    <loc>{loc}</loc>\n"));

        // Emit hreflang alternates for i18n pages
        if let Some(lc) = locale_config {
            let base_slug = &page.slug;
            if let Some(locales_with_page) = lc.slug_coverage.get(base_slug) {
                for alt_locale in &lc.enabled {
                    if locales_with_page.contains(alt_locale) {
                        let alt_path = format!("{alt_locale}/{base_slug}.html");
                        let alt_href = if let Some(url) = site_url {
                            format!("{url}{alt_path}")
                        } else {
                            format!("{base_url}{alt_path}")
                        };
                        xml.push_str(&format!(
                            "    <xhtml:link rel=\"alternate\" hreflang=\"{alt_locale}\" href=\"{alt_href}\"/>\n"
                        ));
                    }
                }
                // x-default points to the default locale's version (if it exists)
                if locales_with_page.contains(&lc.default_locale) {
                    let default_path = format!("{}/{base_slug}.html", lc.default_locale);
                    let default_href = if let Some(url) = site_url {
                        format!("{url}{default_path}")
                    } else {
                        format!("{base_url}{default_path}")
                    };
                    xml.push_str(&format!(
                        "    <xhtml:link rel=\"alternate\" hreflang=\"x-default\" href=\"{default_href}\"/>\n"
                    ));
                }
            }
        }

        xml.push_str("  </url>\n");
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
        let xml = generate_sitemap_xml(&inv, "/", Some("https://example.com/"), None);

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
        let xml = generate_sitemap_xml(&inv, "/docs/", None, None);

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
        let xml = generate_sitemap_xml(&inv, "/", Some("https://example.com/"), None);

        assert!(xml.contains("<loc>https://example.com/guides/setup.html</loc>"));
    }

    #[test]
    fn sitemap_xml_with_i18n_hreflang() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.en.md"), "# Home").unwrap();
        fs::write(docs.join("index.fr.md"), "# Accueil").unwrap();
        fs::write(docs.join("guide.en.md"), "# Guide").unwrap();

        let locales = &["en".to_string(), "fr".to_string()];
        let inv = PageInventory::scan(&docs, Some(locales), Some("en")).unwrap();
        let slug_coverage = inv.slug_locale_coverage();

        let locale_config = SitemapLocaleConfig {
            enabled: vec!["en".to_string(), "fr".to_string()],
            default_locale: "en".to_string(),
            slug_coverage,
        };

        let xml = generate_sitemap_xml(
            &inv,
            "/",
            Some("https://example.com/"),
            Some(&locale_config),
        );

        // Should have xhtml namespace
        assert!(xml.contains("xmlns:xhtml="));
        // Should have hreflang entries for index (which exists in both locales)
        assert!(xml.contains("hreflang=\"en\" href=\"https://example.com/en/index.html\""));
        assert!(xml.contains("hreflang=\"fr\" href=\"https://example.com/fr/index.html\""));
        assert!(xml.contains("hreflang=\"x-default\" href=\"https://example.com/en/index.html\""));
        // Guide only exists in en, so no fr hreflang for it
        assert!(xml.contains("hreflang=\"en\" href=\"https://example.com/en/guide.html\""));
        assert!(!xml.contains("hreflang=\"fr\" href=\"https://example.com/fr/guide.html\""));
    }

    #[test]
    fn sitemap_xml_without_i18n_no_xhtml_namespace() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.md"), "# Home").unwrap();

        let inv = PageInventory::scan(&docs, None, None).unwrap();
        let xml = generate_sitemap_xml(&inv, "/", None, None);

        assert!(!xml.contains("xmlns:xhtml="));
        assert!(!xml.contains("xhtml:link"));
    }
}
