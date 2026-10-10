use rust_embed::Embed;
use std::collections::HashMap;
use std::path::Path;

use crate::config::Config;
use crate::diagnostics::warn_custom_css_not_found;

#[derive(Embed)]
#[folder = "src/theme/default/"]
struct DefaultTheme;

/// Resolved theme resources ready for rendering.
pub struct Theme {
    pub layout_template: String,
    pub default_css: String,
    pub default_js: String,
    pub css_overrides: Option<String>,
    pub custom_css_path: Option<String>,
    /// Custom CSS file content, read at build time for inline injection.
    pub custom_css: Option<String>,
}

impl Theme {
    /// Resolve theme from config and project directory.
    pub fn resolve(config: &Config, project_root: &Path) -> Self {
        // Load default embedded files
        let default_css = DefaultTheme::get("style.css")
            .map(|f| String::from_utf8_lossy(&f.data).into_owned())
            .unwrap_or_default();

        let default_js = DefaultTheme::get("docanvil.js")
            .map(|f| String::from_utf8_lossy(&f.data).into_owned())
            .unwrap_or_default();

        let default_layout = DefaultTheme::get("layout.html")
            .map(|f| String::from_utf8_lossy(&f.data).into_owned())
            .unwrap_or_default();

        // Check for user template override
        let user_layout_path = project_root.join("theme/templates/layout.html");
        let layout_template = if user_layout_path.exists() {
            std::fs::read_to_string(&user_layout_path).unwrap_or(default_layout)
        } else {
            default_layout
        };

        // Check for user JS override
        let user_js_path = project_root.join("theme/docanvil.js");
        let default_js = if user_js_path.exists() {
            std::fs::read_to_string(&user_js_path).unwrap_or(default_js)
        } else {
            default_js
        };

        // Build CSS variable overrides from config
        let css_overrides = build_css_overrides(&config.theme.variables);

        // Custom CSS: resolve path and read content
        let (custom_css_path, custom_css) = match config.theme.custom_css.clone() {
            Some(css_path) => {
                let full_path = project_root.join(&css_path);
                if full_path.exists() {
                    let content = std::fs::read_to_string(&full_path).ok();
                    (Some(css_path), content)
                } else {
                    warn_custom_css_not_found(&css_path);
                    (None, None)
                }
            }
            None => (None, None),
        };

        Self {
            layout_template,
            default_css,
            default_js,
            css_overrides,
            custom_css_path,
            custom_css,
        }
    }
}

/// Build `:root { --key: val; }` overrides from config variables.
fn build_css_overrides(variables: &HashMap<String, String>) -> Option<String> {
    if variables.is_empty() {
        return None;
    }
    let pairs: Vec<String> = variables
        .iter()
        .map(|(k, v)| format!("--{k}: {v};"))
        .collect();
    Some(pairs.join(" "))
}

/// Ship bundled icon assets and their licences beside the theme script.
pub fn write_icon_assets(output_dir: &Path) -> crate::error::Result<()> {
    for name in DefaultTheme::iter().filter(|name| name.starts_with("icons/devicon/")) {
        if let Some(asset) = DefaultTheme::get(&name) {
            let dest = output_dir.join("js").join(name.as_ref());
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(dest, asset.data)?;
        }
    }
    let lucide_dir = output_dir.join("js/icons/lucide");
    std::fs::create_dir_all(&lucide_dir)?;
    std::fs::write(lucide_dir.join("LICENSE.txt"), crate::icons::LICENSE)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapped_code_icons_and_licence_are_shipped() {
        let output = tempfile::tempdir().unwrap();
        write_icon_assets(output.path()).unwrap();
        let js = DefaultTheme::get("docanvil.js").unwrap();
        let js = std::str::from_utf8(&js.data).unwrap();
        let svg_names = regex::Regex::new(r"svg: '([^']+\.svg)'").unwrap();
        let mut count = 0;
        for capture in svg_names.captures_iter(js) {
            let svg =
                std::fs::read_to_string(output.path().join("js/icons/devicon").join(&capture[1]))
                    .unwrap();
            assert!(svg.contains("<svg"), "{} is not an SVG", &capture[1]);
            count += 1;
        }
        assert!(count >= 20, "expected the common-language icon set");
        let licence =
            std::fs::read_to_string(output.path().join("js/icons/devicon/LICENSE.txt")).unwrap();
        assert!(licence.contains("Copyright (c) 2015 konpa"));
        let lucide_licence =
            std::fs::read_to_string(output.path().join("js/icons/lucide/LICENSE.txt")).unwrap();
        assert_eq!(lucide_licence, crate::icons::LICENSE);
    }
}
