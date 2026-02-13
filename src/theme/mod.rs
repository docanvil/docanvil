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

        // Build CSS variable overrides from config
        let css_overrides = build_css_overrides(&config.theme.variables);

        // Custom CSS: resolve path and read content
        let (custom_css_path, custom_css) =
            match config.theme.custom_css.clone() {
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
