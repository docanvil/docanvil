use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Top-level docanvil.toml configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
    pub project: ProjectConfig,
    pub build: BuildConfig,
    pub theme: ThemeConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub name: String,
    pub content_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct BuildConfig {
    pub output_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct ThemeConfig {
    pub name: Option<String>,
    pub custom_css: Option<String>,
    pub variables: HashMap<String, String>,
}


impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: String::from("My Documentation"),
            content_dir: PathBuf::from("docs"),
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("dist"),
        }
    }
}


impl Config {
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
