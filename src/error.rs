use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse config {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("config file not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("no content directory found at {0}")]
    ContentDirNotFound(PathBuf),

    #[error("markdown rendering failed: {0}")]
    Render(String),

    #[error("{0} warning(s) emitted during strict-mode build")]
    StrictWarnings(usize)
}

pub type Result<T> = std::result::Result<T, Error>;
