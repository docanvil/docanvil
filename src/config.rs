use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
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

/// Versioning configuration for multi-version documentation sites.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct VersionConfig {
    /// The current/latest version (used for the root redirect and version banner).
    /// Defaults to the last entry in `enabled` if not set.
    pub current: Option<String>,
    /// All versions to build (e.g. ["v1", "v2"]). Each must have a matching
    /// subdirectory inside the content directory.
    pub enabled: Vec<String>,
    /// Human-readable display names for versions (e.g. {"v1": "v1.0", "v2": "v2.0 (latest)"}).
    pub display_names: HashMap<String, String>,
}

/// Localisation configuration for multi-language documentation sites.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LocaleConfig {
    /// Default locale code (e.g. "en"). When set, i18n is enabled.
    pub default: Option<String>,
    /// Enabled locale codes (e.g. ["en", "fr", "de"]).
    pub enabled: Vec<String>,
    /// Human-readable display names for locales (e.g. {"en": "English", "fr": "Français"}).
    pub display_names: HashMap<String, String>,
    /// Whether to auto-detect the user's browser language and redirect (default: true).
    pub auto_detect: bool,
    /// Flag emoji overrides for locales (e.g. {"en": "🇺🇸"} to use US flag instead of default GB).
    pub flags: HashMap<String, String>,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            default: None,
            enabled: Vec::new(),
            display_names: HashMap::new(),
            auto_detect: true,
            flags: HashMap::new(),
        }
    }
}

/// PDF export configuration.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct PdfConfig {
    /// Author name shown on the cover page.
    pub author: Option<String>,
    /// Whether to prepend a cover page before the TOC (default: false).
    pub cover_page: bool,
    /// Path to a custom CSS file injected into the PDF (relative to project root).
    pub custom_css: Option<String>,
    /// Paper size for PDF export. Recognised values (case-insensitive):
    /// "A3", "A4" (default), "A5", "Letter", "Legal", "Tabloid".
    pub paper_size: Option<String>,
}

/// Doctor / linting configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DoctorConfig {
    /// Maximum number of words in a single paragraph before a readability
    /// warning is emitted. Set to 0 to disable the check. (Default: 150)
    pub max_paragraph_words: usize,
    /// Warn when a heading is directly adjacent to a horizontal rule.
    /// Separators next to headings are usually redundant.
    /// Set to `false` to disable. (Default: true)
    pub heading_adjacent_separator: bool,
}

impl Default for DoctorConfig {
    fn default() -> Self {
        Self {
            max_paragraph_words: 150,
            heading_adjacent_separator: true,
        }
    }
}

/// Git hosting provider for "Edit this page" links.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EditProvider {
    Github,
    Gitlab,
    Bitbucket,
}

/// "Edit this page" link configuration. Links are shown when `repo` is set.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct EditConfig {
    /// Repository web URL (e.g. "https://github.com/org/repo").
    pub repo: Option<String>,
    /// Branch the edit links point at (default: "main").
    pub branch: String,
    /// Hosting provider. Inferred from github.com, gitlab.com and bitbucket.org;
    /// required for self-hosted instances.
    pub provider: Option<EditProvider>,
    /// Location of the project within the repository (e.g. "docs").
    /// Auto-detected from the nearest `.git` when unset.
    pub root: Option<String>,
}

impl Default for EditConfig {
    fn default() -> Self {
        Self {
            repo: None,
            branch: "main".to_string(),
            provider: None,
            root: None,
        }
    }
}

/// Where "last updated" dates come from. Front matter `last_updated` always
/// overrides the computed date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LastUpdatedSource {
    /// The newest commit touching the page or a file it includes.
    Git,
    /// Only front matter `last_updated` dates; Git is never run.
    FrontMatter,
}

/// "Last updated" dates on pages, in `article:modified_time` and in the sitemap.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LastUpdatedConfig {
    pub enabled: bool,
    pub source: LastUpdatedSource,
}

impl Default for LastUpdatedConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            source: LastUpdatedSource::Git,
        }
    }
}

/// `[redirects]`: old paths that send readers on to where a page lives now.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct RedirectsConfig {
    /// Also redirect each `page.html` to the current version and default language.
    pub unprefixed: bool,
    /// Old path → new path, as written in `[redirects]`.
    #[serde(flatten)]
    pub paths: BTreeMap<String, String>,
}

/// `[llms]`: `llms.txt` and `llms-full.txt` for AI tools (<https://llmstxt.org>).
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LlmsConfig {
    pub enabled: bool,
    /// The one-line summary under the title, written as a `>` blockquote.
    pub description: Option<String>,
    /// Also write `llms-full.txt`, with every page's Markdown in one file.
    pub full: bool,
}

impl Default for LlmsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            description: None,
            full: true,
        }
    }
}

/// Returns `true` for right-to-left locales.
pub fn is_rtl_locale(code: &str) -> bool {
    matches!(code, "ar" | "he" | "ur" | "fa" | "ug")
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
    pub locale: LocaleConfig,
    pub version: VersionConfig,
    pub pdf: PdfConfig,
    pub doctor: DoctorConfig,
    pub edit: EditConfig,
    pub last_updated: LastUpdatedConfig,
    pub redirects: RedirectsConfig,
    pub llms: LlmsConfig,
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
    /// How a published page's wiki-links to a draft page render when drafts are left out.
    pub draft_links: DraftLinks,
}

/// What happens to a wiki-link pointing at a draft page in a build that leaves drafts out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DraftLinks {
    /// Render the link text as plain text, quietly.
    #[default]
    Text,
    /// Render plain text and print a warning (so `--strict` fails).
    Warn,
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
    /// Number the lines of every code block (blocks can opt out with `numbers="false"`).
    pub line_numbers: bool,
}

impl Default for SyntaxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: String::from("base16-ocean.dark"),
            line_numbers: false,
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
            draft_links: DraftLinks::Text,
        }
    }
}

/// Map common language codes to flag emoji using regional indicator symbols.
fn default_flag_for_locale(code: &str) -> String {
    match code {
        "en" => "🇬🇧",
        "fr" => "🇫🇷",
        "de" => "🇩🇪",
        "es" => "🇪🇸",
        "it" => "🇮🇹",
        "pt" => "🇵🇹",
        "nl" => "🇳🇱",
        "ja" => "🇯🇵",
        "zh" => "🇨🇳",
        "ko" => "🇰🇷",
        "ru" => "🇷🇺",
        "ar" => "🇸🇦",
        "hi" => "🇮🇳",
        "sv" => "🇸🇪",
        "da" => "🇩🇰",
        "fi" => "🇫🇮",
        "no" => "🇳🇴",
        "pl" => "🇵🇱",
        "tr" => "🇹🇷",
        "uk" => "🇺🇦",
        "cs" => "🇨🇿",
        "el" => "🇬🇷",
        "he" => "🇮🇱",
        "th" => "🇹🇭",
        "vi" => "🇻🇳",
        "id" => "🇮🇩",
        "ms" => "🇲🇾",
        "ro" => "🇷🇴",
        "hu" => "🇭🇺",
        "bg" => "🇧🇬",
        _ => "🌐",
    }
    .to_string()
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

    /// Returns `true` when versioning is enabled (at least one version is configured).
    pub fn is_versioning_enabled(&self) -> bool {
        !self.version.enabled.is_empty()
    }

    /// Return the current/latest version code.
    ///
    /// Uses `version.current` if set; otherwise falls back to the last entry in
    /// `version.enabled`. Returns `None` if versioning is not enabled.
    pub fn current_version(&self) -> Option<&str> {
        self.version
            .current
            .as_deref()
            .or_else(|| self.version.enabled.last().map(|s| s.as_str()))
    }

    /// Return the display name for a version code, or the code itself if no name is configured.
    pub fn version_display_name(&self, code: &str) -> String {
        self.version
            .display_names
            .get(code)
            .cloned()
            .unwrap_or_else(|| code.to_string())
    }

    /// Returns `true` when i18n is enabled (default locale is set and enabled list is non-empty).
    pub fn is_i18n_enabled(&self) -> bool {
        self.locale.default.is_some() && !self.locale.enabled.is_empty()
    }

    /// Return the default locale code, if i18n is enabled.
    pub fn default_locale(&self) -> Option<&str> {
        self.locale.default.as_deref()
    }

    /// Return the flag emoji for a locale code. Uses `[locale.flags]` override if set,
    /// otherwise falls back to a built-in language-to-country mapping.
    pub fn locale_flag(&self, code: &str) -> String {
        self.locale
            .flags
            .get(code)
            .cloned()
            .unwrap_or_else(|| default_flag_for_locale(code))
    }

    /// Return the display name for a locale code, or the code uppercased if no name is configured.
    pub fn locale_display_name(&self, code: &str) -> String {
        self.locale
            .display_names
            .get(code)
            .cloned()
            .unwrap_or_else(|| code.to_uppercase())
    }

    /// Validate version configuration. Returns an error if the config is inconsistent.
    fn validate_version(&self, config_path: &Path) -> Result<()> {
        if let Some(ref current) = self.version.current {
            if self.version.enabled.is_empty() {
                return Err(Error::General(format!(
                    "{}: version.current is set to '{}' but version.enabled is empty — \
                     add enabled versions or remove the current setting",
                    config_path.display(),
                    current
                )));
            }
            if !self.version.enabled.contains(current) {
                return Err(Error::General(format!(
                    "{}: version.current '{}' is not in version.enabled {:?}",
                    config_path.display(),
                    current,
                    self.version.enabled
                )));
            }
        }
        Ok(())
    }

    /// Validate locale configuration. Returns an error if the config is inconsistent.
    fn validate_locale(&self, config_path: &Path) -> Result<()> {
        if let Some(ref default) = self.locale.default {
            if self.locale.enabled.is_empty() {
                return Err(Error::General(format!(
                    "{}: locale.default is set to '{}' but locale.enabled is empty — \
                     add enabled locales or remove the default",
                    config_path.display(),
                    default
                )));
            }
            if !self.locale.enabled.contains(default) {
                return Err(Error::General(format!(
                    "{}: locale.default '{}' is not in locale.enabled {:?}",
                    config_path.display(),
                    default,
                    self.locale.enabled
                )));
            }
        } else if !self.locale.enabled.is_empty() {
            return Err(Error::General(format!(
                "{}: locale.enabled is set but locale.default is missing — \
                 set a default locale",
                config_path.display()
            )));
        }
        Ok(())
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
            path: config_path.clone(),
            source: e,
        })?;
        config.validate_locale(&config_path)?;
        config.validate_version(&config_path)?;
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
    fn syntax_line_numbers_default_off() {
        let config: Config = toml::from_str("").unwrap();
        assert!(!config.syntax.line_numbers);
        let config: Config = toml::from_str("[syntax]\nline_numbers = true\n").unwrap();
        assert!(config.syntax.line_numbers);
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
    fn locale_config_defaults() {
        let config: Config = toml::from_str("").unwrap();
        assert!(!config.is_i18n_enabled());
        assert!(config.locale.default.is_none());
        assert!(config.locale.enabled.is_empty());
        assert!(config.locale.auto_detect);
    }

    #[test]
    fn locale_config_full() {
        let toml = r#"
[locale]
default = "en"
enabled = ["en", "fr"]
auto_detect = false

[locale.display_names]
en = "English"
fr = "Français"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.is_i18n_enabled());
        assert_eq!(config.default_locale(), Some("en"));
        assert_eq!(config.locale_display_name("en"), "English");
        assert_eq!(config.locale_display_name("fr"), "Français");
        assert_eq!(config.locale_display_name("de"), "DE");
        assert!(!config.locale.auto_detect);
    }

    #[test]
    fn locale_validation_default_not_in_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let toml = r#"
[locale]
default = "es"
enabled = ["en", "fr"]
"#;
        std::fs::write(dir.path().join("docanvil.toml"), toml).unwrap();
        let result = Config::load(dir.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("es"),
            "error should mention the bad default: {msg}"
        );
    }

    #[test]
    fn locale_validation_enabled_without_default() {
        let dir = tempfile::tempdir().unwrap();
        let toml = r#"
[locale]
enabled = ["en", "fr"]
"#;
        std::fs::write(dir.path().join("docanvil.toml"), toml).unwrap();
        let result = Config::load(dir.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("default"),
            "error should mention missing default: {msg}"
        );
    }

    #[test]
    fn locale_validation_default_without_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let toml = r#"
[locale]
default = "en"
"#;
        std::fs::write(dir.path().join("docanvil.toml"), toml).unwrap();
        let result = Config::load(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn color_mode_display_and_serialize() {
        assert_eq!(ColorMode::Light.to_string(), "light");
        assert_eq!(ColorMode::Dark.to_string(), "dark");
        assert_eq!(ColorMode::Both.to_string(), "both");

        // Serialize for Tera templates
        assert_eq!(serde_json::to_string(&ColorMode::Both).unwrap(), "\"both\"");
    }

    #[test]
    fn edit_config_defaults() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.edit.repo.is_none());
        assert_eq!(config.edit.branch, "main");
        assert!(config.edit.provider.is_none());
        assert!(config.edit.root.is_none());
    }

    #[test]
    fn edit_config_full() {
        let toml = r#"
[edit]
repo = "https://git.example.com/team/docs"
branch = "develop"
provider = "gitlab"
root = "site"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(
            config.edit.repo.as_deref(),
            Some("https://git.example.com/team/docs")
        );
        assert_eq!(config.edit.branch, "develop");
        assert_eq!(config.edit.provider, Some(EditProvider::Gitlab));
        assert_eq!(config.edit.root.as_deref(), Some("site"));
    }

    #[test]
    fn edit_config_unknown_provider_errors() {
        let toml = "[edit]\nprovider = \"gitea\"\n";
        assert!(toml::from_str::<Config>(toml).is_err());
    }

    #[test]
    fn last_updated_defaults_off_with_git_source() {
        let config: Config = toml::from_str("").unwrap();
        assert!(!config.last_updated.enabled);
        assert_eq!(config.last_updated.source, LastUpdatedSource::Git);
    }

    #[test]
    fn last_updated_parses_sources() {
        let config: Config =
            toml::from_str("[last_updated]\nenabled = true\nsource = \"front-matter\"\n").unwrap();
        assert!(config.last_updated.enabled);
        assert_eq!(config.last_updated.source, LastUpdatedSource::FrontMatter);
    }

    #[test]
    fn llms_defaults_off_with_full_text() {
        let config: Config = toml::from_str("").unwrap();
        assert!(!config.llms.enabled);
        assert!(config.llms.full);
        assert!(config.llms.description.is_none());
    }

    #[test]
    fn llms_parses() {
        let config: Config = toml::from_str(
            "[llms]\nenabled = true\ndescription = \"Fast docs.\"\nfull = false\n",
        )
        .unwrap();
        assert!(config.llms.enabled);
        assert!(!config.llms.full);
        assert_eq!(config.llms.description.as_deref(), Some("Fast docs."));
    }

    #[test]
    fn last_updated_rejects_unknown_source() {
        let err = toml::from_str::<Config>("[last_updated]\nsource = \"svn\"\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("git") && err.contains("front-matter"), "{err}");
    }

    #[test]
    fn draft_links_default_to_text() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.build.draft_links, DraftLinks::Text);
        let config: Config = toml::from_str("[build]\ndraft_links = \"warn\"\n").unwrap();
        assert_eq!(config.build.draft_links, DraftLinks::Warn);
        assert!(toml::from_str::<Config>("[build]\ndraft_links = \"error\"\n").is_err());
    }

    #[test]
    fn redirects_default_to_none() {
        let config: Config = toml::from_str("").unwrap();
        assert!(!config.redirects.unprefixed);
        assert!(config.redirects.paths.is_empty());
    }

    #[test]
    fn redirects_parse_flag_and_paths() {
        let config: Config = toml::from_str(
            "[redirects]\nunprefixed = true\n\"old-faq\" = \"help/faq\"\n\"/blog.html\" = \"https://blog.example.com\"\n",
        )
        .unwrap();
        assert!(config.redirects.unprefixed);
        assert_eq!(config.redirects.paths.len(), 2);
        assert_eq!(config.redirects.paths["old-faq"], "help/faq");
        assert_eq!(
            config.redirects.paths["/blog.html"],
            "https://blog.example.com"
        );
    }

    #[test]
    fn redirects_reject_non_string_targets() {
        assert!(toml::from_str::<Config>("[redirects]\n\"old\" = 3\n").is_err());
    }
}
