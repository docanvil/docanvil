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

    #[error("config not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("no content directory found at {0}")]
    ContentDirNotFound(PathBuf),

    #[error("markdown rendering failed: {0}")]
    Render(String),

    #[error("{0}")]
    General(String),

    #[error("{0} warning(s) emitted during strict-mode build")]
    StrictWarnings(usize),

    #[error("doctor found {errors} error(s) and {warnings} warning(s)")]
    DoctorFailed { warnings: usize, errors: usize },
}

impl Error {
    /// Return a structured exit code for CI pipelines.
    ///
    /// | Code | Meaning                  |
    /// |------|--------------------------|
    /// | 1    | General / IO failure     |
    /// | 2    | Configuration error      |
    /// | 3    | Content validation error  |
    /// | 4    | Theme / rendering error   |
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Io(_) | Error::General(_) => 1,
            Error::ConfigParse { .. } | Error::ConfigNotFound(_) => 2,
            Error::ContentDirNotFound(_) | Error::StrictWarnings(_) | Error::DoctorFailed { .. } => {
                3
            }
            Error::Render(_) => 4,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_general_and_io() {
        let io_err = Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "gone"));
        assert_eq!(io_err.exit_code(), 1);

        let general = Error::General("something broke".into());
        assert_eq!(general.exit_code(), 1);
    }

    #[test]
    fn exit_code_config() {
        let parse_err = Error::ConfigParse {
            path: PathBuf::from("docanvil.toml"),
            source: toml::from_str::<toml::Value>("not valid").unwrap_err(),
        };
        assert_eq!(parse_err.exit_code(), 2);

        let not_found = Error::ConfigNotFound(PathBuf::from("docanvil.toml"));
        assert_eq!(not_found.exit_code(), 2);
    }

    #[test]
    fn exit_code_content_validation() {
        let content = Error::ContentDirNotFound(PathBuf::from("docs"));
        assert_eq!(content.exit_code(), 3);

        let strict = Error::StrictWarnings(2);
        assert_eq!(strict.exit_code(), 3);

        let doctor = Error::DoctorFailed {
            warnings: 1,
            errors: 2,
        };
        assert_eq!(doctor.exit_code(), 3);
    }

    #[test]
    fn exit_code_render() {
        let render = Error::Render("template broke".into());
        assert_eq!(render.exit_code(), 4);
    }
}
