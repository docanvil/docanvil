//! Isolated Git snapshots for documentation versions.
//!
//! Snapshot the repository, rather than just the documentation directory: nav,
//! includes and assets may live elsewhere in the same commit. No checkout,
//! worktree registration or network access is needed.

use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::Config;
use crate::error::{Error, Result};

/// Expand local tag patterns once when loading the working configuration.
/// Downstream build, serve, doctor and PDF code sees ordinary enabled versions
/// and explicit ref mappings, keeping source resolution identical in every mode.
pub(crate) fn discover_tags(root: &Path, config: &mut Config) -> Result<()> {
    let git = &mut config.version.git;
    if git.tag_patterns.is_empty() {
        return Ok(());
    }
    for pattern in &git.tag_patterns {
        if pattern.is_empty()
            || !pattern
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b".-_+*?".contains(&c))
        {
            return Err(Error::General(format!(
                "Invalid version.git.tag_patterns entry '{pattern}': use tag names with '*' and '?' wildcards"
            )));
        }
    }
    let mut explicit = std::collections::HashSet::new();
    for version in &config.version.enabled {
        if !explicit.insert(version.to_ascii_lowercase()) {
            return Err(Error::General(format!(
                "Duplicate enabled version '{version}'"
            )));
        }
    }
    let listed = git_text(root, &["for-each-ref", "--format=%(refname:strip=2)", "refs/tags"])
        .map_err(|e| Error::General(format!("Cannot discover documentation tags: {e}. Ensure Git is installed and release tags exist locally; builds do not fetch tags.")))?;
    let tags: Vec<_> = listed.lines().collect();
    let mut discovered = std::collections::HashSet::new();
    for pattern in &git.tag_patterns {
        let matches: Vec<_> = tags
            .iter()
            .copied()
            .filter(|tag| {
                matches_tag(pattern, tag)
                    && (git.include_prereleases
                        || tag_version(tag).is_none_or(|version| version.pre.is_empty()))
            })
            .collect();
        if matches.is_empty() {
            return Err(Error::General(format!(
                "Git tag pattern '{pattern}' matched no eligible local tags. Fetch release tags, adjust the pattern, or set include_prereleases = true if only prereleases match."
            )));
        }
        discovered.extend(matches);
    }
    let mut discovered: Vec<_> = discovered.into_iter().collect();
    discovered.sort_by(|a, b| match (tag_version(a), tag_version(b)) {
        (Some(a_version), Some(b_version)) => a_version.cmp(&b_version).then_with(|| a.cmp(b)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.cmp(b),
    });
    let mut enabled: Vec<String> = discovered.iter().map(|tag| (*tag).to_owned()).collect();
    for tag in discovered {
        git.refs
            .entry(tag.to_owned())
            .or_insert_with(|| format!("refs/tags/{tag}"));
    }
    for version in &config.version.enabled {
        if !enabled.contains(version) {
            enabled.push(version.clone());
        }
    }
    config.version.enabled = enabled;
    Ok(())
}

fn tag_version(tag: &str) -> Option<semver::Version> {
    semver::Version::parse(tag.strip_prefix('v').unwrap_or(tag)).ok()
}

/// Simple wildcard matching with constant memory. Tag names and patterns are
/// ASCII URL segments; '*' and '?' are the only pattern operators.
fn matches_tag(pattern: &str, tag: &str) -> bool {
    let (pattern, tag) = (pattern.as_bytes(), tag.as_bytes());
    let (mut p, mut t, mut star, mut retry) = (0, 0, None, 0);
    while t < tag.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == tag[t]) {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            retry = t;
        } else if let Some(last_star) = star {
            retry += 1;
            t = retry;
            p = last_star + 1;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

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
            let historical = Config::load_unresolved(snapshot.project_root())?;
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
            // Archive uses Git's checkout conversion rules. Ignore local or
            // global EOL preferences so the same tag renders consistently on
            // every OS; committed .gitattributes still take precedence.
            .args([
                "-c",
                "core.autocrlf=false",
                "-c",
                "core.eol=lf",
                "archive",
                "--format=tar",
                &commit,
            ])
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
    fn snapshot_ignores_checkout_line_ending_preferences() {
        let repo = repository();
        git(repo.path(), &["config", "core.autocrlf", "true"]);
        git(repo.path(), &["config", "core.eol", "crlf"]);
        let snapshot = GitSnapshot::extract(&repo.path().join("site"), "v1.0.0").unwrap();
        assert_eq!(
            fs::read(snapshot.project_root().join("docs/index.md")).unwrap(),
            b"# Released\n"
        );
        assert_eq!(git(repo.path(), &["config", "core.autocrlf"]), "true");
        assert_eq!(git(repo.path(), &["config", "core.eol"]), "crlf");
    }

    #[test]
    fn snapshot_respects_committed_line_ending_attributes() {
        let repo = repository();
        fs::write(
            repo.path().join(".gitattributes"),
            "site/docs/index.md text eol=crlf\n",
        )
        .unwrap();
        git(repo.path(), &["add", ".gitattributes"]);
        git(repo.path(), &["commit", "-m", "Specify line endings"]);
        let snapshot = GitSnapshot::extract(&repo.path().join("site"), "HEAD").unwrap();
        assert_eq!(
            fs::read(snapshot.project_root().join("docs/index.md")).unwrap(),
            b"# Released\r\n"
        );
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

    #[test]
    fn tag_wildcards_match_whole_names() {
        for (pattern, tag, expected) in [
            ("v1.*", "v1.2.3", true),
            ("v1.*", "v2.0.0", false),
            ("v1.*", "prefix-v1.2.3", false),
            ("v1.*", "v1.", true),
            ("v1.?.0", "v1.2.0", true),
            ("v1.?.0", "v1.12.0", false),
            ("*beta*", "v1.2.3-beta.1", true),
            ("v1.*.3", "v1.2.4.3", true),
            ("v1.*.3", "v1.2.4", false),
            ("**", "anything", true),
            ("v1.0.0", "v1.0.0-extra", false),
        ] {
            assert_eq!(matches_tag(pattern, tag), expected, "{pattern}: {tag}");
        }
    }

    fn pattern_config(repo: &Path, patterns: &str, extra: &str) -> Result<Config> {
        std::fs::write(
            repo.join("site/docanvil.toml"),
            format!(
                "[version]\ncurrent = 'v1.0.0'\nenabled = ['master']\n[version.git]\nworking_tree = 'master'\ntag_patterns = {patterns}\n{extra}"
            ),
        )?;
        Config::load(&repo.join("site"))
    }

    #[test]
    fn discovers_semver_ordered_tags_and_deduplicates_overlaps() {
        let repo = repository();
        for tag in ["v1.9.0", "v1.10.0", "v1.10.0-beta.1", "v2.0.0"] {
            git(repo.path(), &["tag", tag]);
        }
        let config = pattern_config(
            repo.path(),
            "['v1.*', 'v1.9.*']",
            "refs = { 'v1.9.0' = 'refs/tags/v1.0.0' }",
        )
        .unwrap();
        assert_eq!(
            config.version.enabled,
            ["v1.0.0", "v1.0.1", "v1.9.0", "v1.10.0", "master"]
        );
        assert_eq!(config.version.git.refs["v1.9.0"], "refs/tags/v1.0.0");
        assert_eq!(config.version.git.refs["v1.10.0"], "refs/tags/v1.10.0");
        let config = pattern_config(repo.path(), "['v1.*']", "include_prereleases = true").unwrap();
        assert_eq!(
            config.version.enabled,
            [
                "v1.0.0",
                "v1.0.1",
                "v1.9.0",
                "v1.10.0-beta.1",
                "v1.10.0",
                "master"
            ]
        );
    }

    #[test]
    fn errors_for_invalid_or_empty_patterns_and_missing_tags() {
        let repo = repository();
        git(repo.path(), &["tag", "v1.1.0-beta.1"]);
        for patterns in ["['']", "['v[1].*']", "['v9.*']", "['v1.1.0-beta.*']"] {
            assert!(
                pattern_config(repo.path(), patterns, "").is_err(),
                "{patterns}"
            );
        }
        let config = pattern_config(
            repo.path(),
            "['v1.1.0-beta.*']",
            "include_prereleases = true",
        );
        // Eligible prereleases were found, but the configured current release
        // is not selected by this pattern.
        assert!(config.unwrap_err().to_string().contains("version.current"));
    }

    #[test]
    fn snapshots_do_not_discover_their_historical_patterns() {
        let repo = repository();
        pattern_config(repo.path(), "['v1.*']", "").unwrap();
        git(repo.path(), &["add", "."]);
        git(
            repo.path(),
            &["commit", "-m", "Configure pattern discovery"],
        );
        git(repo.path(), &["tag", "v1.0.2"]);
        let config = Config::load(&repo.path().join("site")).unwrap();
        let source = VersionSource::resolve(&repo.path().join("site"), &config, "v1.0.2").unwrap();
        assert!(source.content_dir.join("index.md").is_file());
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
