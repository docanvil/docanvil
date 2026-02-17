use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Valid color mode for the theme.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    #[default]
    Light,
    Dark,
    Both,
}

impl ColorMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ColorMode::Light => "light",
            ColorMode::Dark => "dark",
            ColorMode::Both => "both",
        }
    }
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ColorMode {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "light" => Ok(ColorMode::Light),
            "dark" => Ok(ColorMode::Dark),
            "both" => Ok(ColorMode::Both),
            other => Err(serde::de::Error::custom(format!(
                "invalid color_mode \"{other}\" — expected \"light\", \"dark\", or \"both\""
            ))),
        }
    }
}

/// Top-level docanvil.toml configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
    pub project: ProjectConfig,
    pub build: BuildConfig,
    pub theme: ThemeConfig,
    pub syntax: SyntaxConfig,
    pub charts: ChartsConfig,
    pub search: SearchConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub name: String,
    pub content_dir: PathBuf,
    pub logo: Option<String>,
    pub favicon: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct BuildConfig {
    pub output_dir: PathBuf,
    pub base_url: String,
    pub site_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub name: Option<String>,
    pub custom_css: Option<String>,
    pub color_mode: ColorMode,
    pub variables: HashMap<String, String>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: None,
            custom_css: None,
            color_mode: ColorMode::Light,
            variables: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SyntaxConfig {
    pub enabled: bool,
    pub theme: String,
}

impl Default for SyntaxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: String::from("base16-ocean.dark"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ChartsConfig {
    pub enabled: bool,
    pub mermaid_version: String,
}

impl Default for ChartsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mermaid_version: String::from("11"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    pub enabled: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: String::from("My Documentation"),
            content_dir: PathBuf::from("docs"),
            logo: None,
            favicon: None,
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("dist"),
            base_url: "/".to_string(),
            site_url: None,
        }
    }
}

/// Normalize a base_url: ensure leading and trailing `/`.
fn normalize_base_url(url: &str) -> String {
    let trimmed = url.trim().trim_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        format!("/{trimmed}/")
    }
}

impl Config {
    /// Return the normalized base_url (ensures leading + trailing `/`).
    pub fn base_url(&self) -> String {
        normalize_base_url(&self.build.base_url)
    }

    /// Return the normalized site_url (ensures trailing `/`), if configured.
    pub fn site_url(&self) -> Option<String> {
        self.build.site_url.as_ref().map(|url| {
            let trimmed = url.trim().trim_end_matches('/');
            format!("{trimmed}/")
        })
    }

    /// Load config from a `docanvil.toml` file in the given directory.
    /// Returns default config if the file doesn't exist.
    pub fn load(project_root: &Path) -> Result<Self> {
        let config_path = project_root.join("docanvil.toml");
        if !config_path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents).map_err(|e| Error::ConfigParse {
            path: config_path,
            source: e,
        })?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_uses_defaults() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.project.name, "My Documentation");
        assert_eq!(config.project.content_dir, PathBuf::from("docs"));
        assert_eq!(config.build.output_dir, PathBuf::from("dist"));
        assert_eq!(config.build.base_url, "/");
        assert_eq!(config.theme.color_mode, ColorMode::Light);
        assert!(config.syntax.enabled);
        assert!(config.charts.enabled);
        assert!(config.search.enabled);
    }

    #[test]
    fn partial_config_fills_defaults() {
        let toml = r#"
[project]
name = "My Docs"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.project.name, "My Docs");
        // Missing sections use defaults
        assert_eq!(config.build.output_dir, PathBuf::from("dist"));
        assert_eq!(config.theme.color_mode, ColorMode::Light);
        assert!(config.syntax.enabled);
    }

    #[test]
    fn invalid_toml_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("docanvil.toml"), "not valid [[ toml").unwrap();
        let result = Config::load(dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::ConfigParse { .. }));
    }

    #[test]
    fn missing_config_file_returns_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load(dir.path()).unwrap();
        assert_eq!(config.project.name, "My Documentation");
    }

    #[test]
    fn base_url_normalization() {
        assert_eq!(normalize_base_url("/"), "/");
        assert_eq!(normalize_base_url(""), "/");
        assert_eq!(normalize_base_url("  "), "/");
        assert_eq!(normalize_base_url("/docs"), "/docs/");
        assert_eq!(normalize_base_url("/docs/"), "/docs/");
        assert_eq!(normalize_base_url("docs"), "/docs/");
        assert_eq!(normalize_base_url("docs/"), "/docs/");
        assert_eq!(normalize_base_url("/a/b/c"), "/a/b/c/");
        assert_eq!(normalize_base_url("///"), "/");
    }

    #[test]
    fn color_mode_valid_values() {
        let toml = r#"
[theme]
color_mode = "light"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.theme.color_mode, ColorMode::Light);

        let toml = r#"
[theme]
color_mode = "dark"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.theme.color_mode, ColorMode::Dark);

        let toml = r#"
[theme]
color_mode = "both"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.theme.color_mode, ColorMode::Both);
    }

    #[test]
    fn color_mode_invalid_value_errors() {
        let toml = r#"
[theme]
color_mode = "purple"
"#;
        let result: std::result::Result<Config, _> = toml::from_str(toml);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("purple"),
            "error should mention the bad value: {msg}"
        );
    }

    #[test]
    fn site_url_normalization() {
        let config = Config {
            build: BuildConfig {
                site_url: Some("https://example.com".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(config.site_url(), Some("https://example.com/".to_string()));

        let config = Config {
            build: BuildConfig {
                site_url: Some("https://example.com/".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(config.site_url(), Some("https://example.com/".to_string()));
    }

    #[test]
    fn color_mode_display_and_serialize() {
        assert_eq!(ColorMode::Light.to_string(), "light");
        assert_eq!(ColorMode::Dark.to_string(), "dark");
        assert_eq!(ColorMode::Both.to_string(), "both");

        // Serialize for Tera templates
        assert_eq!(serde_json::to_string(&ColorMode::Both).unwrap(), "\"both\"");
    }
}
