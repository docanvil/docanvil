//! Isolated Git snapshots for documentation versions.
//!
//! Snapshot the repository, rather than just the documentation directory: nav,
//! includes and assets may live elsewhere in the same commit. No checkout,
//! worktree registration or network access is needed.

use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::Config;
use crate::error::{Error, Result};

/// A version's files, with its snapshot owned for the duration of the build.
pub(crate) struct VersionSource {
    pub project_root: PathBuf,
    pub content_dir: PathBuf,
    pub snapshot: Option<GitSnapshot>,
    /// Git and working-tree versions publish their content assets under the version.
    pub versioned_assets: bool,
}

impl VersionSource {
    pub fn resolve(root: &Path, config: &Config, version: &str) -> Result<Self> {
        if let Some(reference) = config.version.git.refs.get(version) {
            let snapshot = GitSnapshot::extract(root, reference)?;
            let historical = Config::load(snapshot.project_root())?;
            let content_path = &historical.project.content_dir;
            if content_path.is_absolute()
                || content_path
                    .components()
                    .any(|c| matches!(c, Component::ParentDir))
            {
                return Err(Error::General(format!(
                    "Git version '{version}' content_dir must stay inside its project"
                )));
            }
            let project_root = snapshot.project_root().to_owned();
            let content_dir = project_root.join(content_path);
            Ok(Self {
                project_root,
                content_dir,
                snapshot: Some(snapshot),
                versioned_assets: true,
            })
        } else {
            let working = config.version.git.working_tree.as_deref() == Some(version);
            let content_dir = root.join(&config.project.content_dir);
            Ok(Self {
                project_root: root.to_owned(),
                content_dir: if working {
                    content_dir
                } else {
                    content_dir.join(version)
                },
                snapshot: None,
                versioned_assets: working,
            })
        }
    }
}

/// Owns a temporary repository snapshot. Dropping it removes the extracted files.
#[derive(Debug)]
pub struct GitSnapshot {
    _directory: tempfile::TempDir,
    repository_root: PathBuf,
    project_root: PathBuf,
    commit: String,
}

impl GitSnapshot {
    /// Extract a local Git ref, resolved once to an immutable commit ID.
    ///
    /// `project_root` may be a documentation project nested in the repository.
    /// Git must be installed and the ref must already exist locally. Like
    /// `git archive`, this respects export-ignore/export-subst attributes and
    /// does not include submodule contents. Symlinks are rejected so extracted
    /// sources cannot read through links outside the snapshot.
    pub fn extract(project_root: &Path, reference: &str) -> Result<Self> {
        if reference.is_empty() || reference.starts_with('-') {
            return Err(Error::General(
                "Git version ref must be non-empty and cannot start with '-'".into(),
            ));
        }
        let project_root = project_root.canonicalize()?;
        // Git uses normal drive paths on Windows while canonicalize returns
        // extended paths; normalize both before comparing project locations.
        let repository = PathBuf::from(git_text(&project_root, &["rev-parse", "--show-toplevel"])?)
            .canonicalize()?;
        let relative_project = project_root.strip_prefix(&repository).map_err(|_| {
            Error::General("Documentation project is outside the Git repository".into())
        })?;
        let revision = format!("{reference}^{{commit}}");
        let commit = git_text(&repository, &["rev-parse", "--verify", "--end-of-options", &revision])
            .map_err(|e| Error::General(format!("Cannot resolve Git version ref '{reference}': {e}. Ensure the ref exists locally; fetch release tags in CI.")))?;

        let directory = tempfile::tempdir()?;
        let archive_file = tempfile::tempfile()?;
        let output = Command::new("git")
            .current_dir(&repository)
            .args(["archive", "--format=tar", &commit])
            .stdout(Stdio::from(archive_file.try_clone()?))
            .output()?;
        if !output.status.success() {
            return Err(git_error("archive", &output.stderr));
        }
        use std::io::{Seek, SeekFrom};
        let mut archive_file = archive_file;
        archive_file.seek(SeekFrom::Start(0))?;
        let mut archive = tar::Archive::new(archive_file);
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.into_owned();
            let kind = entry.header().entry_type();
            // Git records the commit ID in a global PAX metadata header.
            if kind.is_pax_global_extensions() {
                continue;
            }
            if !kind.is_file() && !kind.is_dir() {
                return Err(Error::General(format!(
                    "Git snapshot contains unsupported entry '{}': only files and directories are supported",
                    path.display()
                )));
            }
            if path
                .components()
                .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
                || !entry.unpack_in(directory.path())?
            {
                return Err(Error::General(format!(
                    "Unsafe path in Git snapshot: {}",
                    path.display()
                )));
            }
        }
        let repository_root = directory.path().canonicalize()?;
        let snapshot_project = repository_root.join(relative_project);
        if !snapshot_project.join("docanvil.toml").is_file() {
            return Err(Error::General(format!(
                "Git version ref '{reference}' has no docanvil.toml at '{}'",
                relative_project.display()
            )));
        }
        Ok(Self {
            _directory: directory,
            repository_root,
            project_root: snapshot_project,
            commit,
        })
    }

    /// Documentation project directory inside the snapshot.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Extracted repository root, including files used by relative includes.
    pub fn repository_root(&self) -> &Path {
        &self.repository_root
    }

    /// Resolved commit, also suitable for release edit links and date lookup.
    pub fn commit(&self) -> &str {
        &self.commit
    }
}

fn git_text(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").current_dir(root).args(args).output()?;
    if !output.status.success() {
        return Err(git_error(args[0], &output.stderr));
    }
    String::from_utf8(output.stdout)
        .map(|text| text.trim_end_matches(['\r', '\n']).to_owned())
        .map_err(|_| Error::General("Git returned a non-UTF-8 path or ref".into()))
}

fn git_error(operation: &str, stderr: &[u8]) -> Error {
    Error::General(format!(
        "git {operation} failed: {}",
        String::from_utf8_lossy(stderr).trim()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn git(root: &Path, args: &[&str]) -> String {
        git_text(root, args).unwrap()
    }

    fn repository() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-b", "master"]);
        git(dir.path(), &["config", "user.email", "test@example.com"]);
        git(dir.path(), &["config", "user.name", "Test"]);
        fs::create_dir_all(dir.path().join("site/docs")).unwrap();
        fs::write(
            dir.path().join("site/docanvil.toml"),
            "[project]\nname = 'Test'\n",
        )
        .unwrap();
        fs::write(dir.path().join("site/docs/index.md"), "# Released\n").unwrap();
        fs::write(dir.path().join("site/nav.toml"), "# release navigation\n").unwrap();
        fs::write(dir.path().join("example.rs"), "// release example\n").unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "-m", "Release"]);
        git(dir.path(), &["tag", "v1.0.0"]);
        git(
            dir.path(),
            &["tag", "-a", "v1.0.1", "-m", "Annotated release"],
        );
        dir
    }

    #[test]
    fn tagged_snapshot_isolated_from_dirty_checkout_and_cleans_up() {
        let repo = repository();
        let head = git(repo.path(), &["rev-parse", "HEAD"]);
        fs::write(repo.path().join("site/docs/index.md"), "# Unreleased\n").unwrap();
        fs::write(repo.path().join("site/nav.toml"), "# new navigation\n").unwrap();
        fs::write(repo.path().join("untracked.md"), "local only").unwrap();
        let before = git(repo.path(), &["status", "--porcelain"]);
        let snapshot = GitSnapshot::extract(&repo.path().join("site"), "refs/tags/v1.0.0").unwrap();
        assert_eq!(snapshot.commit(), head);
        assert_eq!(
            fs::read_to_string(snapshot.project_root().join("docs/index.md")).unwrap(),
            "# Released\n"
        );
        assert_eq!(
            fs::read_to_string(snapshot.project_root().join("nav.toml")).unwrap(),
            "# release navigation\n"
        );
        assert!(snapshot.repository_root().join("example.rs").is_file());
        assert!(!snapshot.repository_root().join("untracked.md").exists());
        assert_eq!(git(repo.path(), &["status", "--porcelain"]), before);
        assert_eq!(
            git(repo.path(), &["symbolic-ref", "--short", "HEAD"]),
            "master"
        );
        let extracted = snapshot.repository_root().to_owned();
        drop(snapshot);
        assert!(!extracted.exists());
    }

    #[test]
    fn annotated_tag_resolves_to_commit() {
        let repo = repository();
        let snapshot = GitSnapshot::extract(&repo.path().join("site"), "refs/tags/v1.0.1").unwrap();
        assert_eq!(snapshot.commit(), git(repo.path(), &["rev-parse", "HEAD"]));
    }

    #[test]
    fn missing_ref_has_actionable_error() {
        let repo = repository();
        let error = GitSnapshot::extract(&repo.path().join("site"), "refs/tags/missing")
            .unwrap_err()
            .to_string();
        assert!(error.contains("refs/tags/missing"));
        assert!(error.contains("fetch release tags"));
    }

    #[test]
    fn rejects_option_like_refs() {
        let repo = repository();
        for reference in ["", "--help"] {
            assert!(GitSnapshot::extract(&repo.path().join("site"), reference).is_err());
        }
    }

    #[test]
    fn rejects_missing_project_in_tag() {
        let repo = repository();
        fs::create_dir(repo.path().join("new-site")).unwrap();
        let error = GitSnapshot::extract(&repo.path().join("new-site"), "v1.0.0")
            .unwrap_err()
            .to_string();
        assert!(error.contains("no docanvil.toml"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinks_in_snapshot() {
        let repo = repository();
        std::os::unix::fs::symlink("/etc/passwd", repo.path().join("site/docs/link.md")).unwrap();
        git(repo.path(), &["add", "."]);
        git(repo.path(), &["commit", "-m", "Add symlink"]);
        let error = GitSnapshot::extract(&repo.path().join("site"), "HEAD")
            .unwrap_err()
            .to_string();
        assert!(error.contains("unsupported entry"));
        assert!(error.contains("link.md"));
    }
}
