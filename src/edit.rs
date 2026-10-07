//! "Edit this page" links pointing at a page's source on its Git host.

use std::path::{Path, PathBuf};

use crate::config::{EditConfig, EditProvider};

/// Resolved edit-link settings for a build.
#[derive(Debug)]
pub struct EditLinks {
    repo: String,
    branch: String,
    provider: EditProvider,
    /// The project's location within the repository ("" when it is the repo root).
    repo_prefix: String,
    project_root: PathBuf,
}

impl EditLinks {
    /// Resolve edit-link settings. Returns `Ok(None)` when no `repo` is configured,
    /// and `Err` with a user-facing message when the config can't produce links.
    pub fn from_config(
        config: &EditConfig,
        project_root: &Path,
    ) -> std::result::Result<Option<Self>, String> {
        let Some(repo) = config.repo.as_deref() else {
            return Ok(None);
        };
        let repo = repo.trim().trim_end_matches('/');
        let repo = repo.strip_suffix(".git").unwrap_or(repo).to_string();

        let Some(host) = host_of(&repo) else {
            return Err(format!(
                "[edit] repo \"{repo}\" must be the repository's web address, \
                 an https:// URL like \"https://github.com/org/repo\""
            ));
        };
        let provider = match config.provider.or_else(|| provider_for_host(&host)) {
            Some(p) => p,
            None => {
                return Err(format!(
                    "[edit] can't tell which Git host \"{host}\" is. \
                     Set provider = \"github\", \"gitlab\" or \"bitbucket\""
                ));
            }
        };

        let repo_prefix = match &config.root {
            Some(root) => root.trim_matches('/').to_string(),
            None => detect_repo_prefix(project_root),
        };

        Ok(Some(Self {
            repo,
            branch: config.branch.clone(),
            provider,
            repo_prefix,
            project_root: project_root.to_path_buf(),
        }))
    }

    /// The edit URL for a page's source file, or `None` if it lies outside the project.
    pub fn url_for(&self, source_path: &Path) -> Option<String> {
        let relative = source_path.strip_prefix(&self.project_root).ok()?;
        let mut segments: Vec<String> = Vec::new();
        if !self.repo_prefix.is_empty() {
            segments.extend(self.repo_prefix.split('/').map(encode_segment));
        }
        for component in relative.components() {
            segments.push(encode_segment(component.as_os_str().to_str()?));
        }
        let path = segments.join("/");
        let branch = self
            .branch
            .split('/')
            .map(encode_segment)
            .collect::<Vec<_>>()
            .join("/");

        Some(match self.provider {
            EditProvider::Github => format!("{}/edit/{branch}/{path}", self.repo),
            EditProvider::Gitlab => format!("{}/-/edit/{branch}/{path}", self.repo),
            EditProvider::Bitbucket => format!("{}/src/{branch}/{path}?mode=edit", self.repo),
        })
    }
}

/// The lowercased host of an http(s) URL, without a leading `www.`.
fn host_of(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host = rest.split('/').next()?.to_ascii_lowercase();
    if host.is_empty() {
        return None;
    }
    Some(host.strip_prefix("www.").unwrap_or(&host).to_string())
}

fn provider_for_host(host: &str) -> Option<EditProvider> {
    match host {
        "github.com" => Some(EditProvider::Github),
        "gitlab.com" => Some(EditProvider::Gitlab),
        "bitbucket.org" => Some(EditProvider::Bitbucket),
        _ => None,
    }
}

/// The project's path within its Git repository, found by walking up to the
/// nearest `.git` (a directory, or a file for worktrees and submodules).
/// Returns "" when no repository is found.
fn detect_repo_prefix(project_root: &Path) -> String {
    let root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let Some(repo_root) = root.ancestors().find(|dir| dir.join(".git").exists()) else {
        return String::new();
    };
    root.strip_prefix(repo_root)
        .map(|prefix| {
            prefix
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_default()
}

/// Percent-encode one URL path segment, keeping RFC 3986 unreserved characters.
fn encode_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(repo: &str) -> EditConfig {
        EditConfig {
            repo: Some(repo.to_string()),
            root: Some(String::new()),
            ..EditConfig::default()
        }
    }

    fn links(config: &EditConfig, root: &Path) -> EditLinks {
        EditLinks::from_config(config, root).unwrap().unwrap()
    }

    #[test]
    fn no_repo_means_no_links() {
        let result = EditLinks::from_config(&EditConfig::default(), Path::new("/p")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn github_url() {
        let root = Path::new("/p");
        let l = links(&config("https://github.com/org/repo"), root);
        assert_eq!(
            l.url_for(&root.join("docs/guides/setup.md")).unwrap(),
            "https://github.com/org/repo/edit/main/docs/guides/setup.md"
        );
    }

    #[test]
    fn gitlab_url_with_subgroup() {
        let root = Path::new("/p");
        let l = links(&config("https://gitlab.com/group/sub/repo"), root);
        assert_eq!(
            l.url_for(&root.join("docs/index.md")).unwrap(),
            "https://gitlab.com/group/sub/repo/-/edit/main/docs/index.md"
        );
    }

    #[test]
    fn bitbucket_url() {
        let root = Path::new("/p");
        let l = links(&config("https://bitbucket.org/team/repo"), root);
        assert_eq!(
            l.url_for(&root.join("docs/index.md")).unwrap(),
            "https://bitbucket.org/team/repo/src/main/docs/index.md?mode=edit"
        );
    }

    #[test]
    fn provider_host_match_ignores_case_and_www() {
        let root = Path::new("/p");
        let l = links(&config("https://www.GitHub.com/org/repo"), root);
        assert!(
            l.url_for(&root.join("docs/a.md"))
                .unwrap()
                .contains("/edit/main/")
        );
    }

    #[test]
    fn self_hosted_uses_configured_provider() {
        let root = Path::new("/p");
        let mut c = config("https://git.example.com/team/docs");
        c.provider = Some(EditProvider::Gitlab);
        c.branch = "develop".to_string();
        assert_eq!(
            links(&c, root).url_for(&root.join("docs/a.md")).unwrap(),
            "https://git.example.com/team/docs/-/edit/develop/docs/a.md"
        );
    }

    #[test]
    fn unknown_host_without_provider_errors() {
        let err = EditLinks::from_config(
            &config("https://git.example.com/team/docs"),
            Path::new("/p"),
        )
        .unwrap_err();
        assert!(err.contains("provider"), "{err}");
    }

    #[test]
    fn non_https_repo_errors() {
        let err = EditLinks::from_config(&config("git@github.com:org/repo.git"), Path::new("/p"))
            .unwrap_err();
        assert!(err.contains("https://"), "{err}");
    }

    #[test]
    fn repo_trailing_slash_and_git_suffix_are_stripped() {
        let root = Path::new("/p");
        let l = links(&config("https://github.com/org/repo.git/"), root);
        assert_eq!(
            l.url_for(&root.join("docs/a.md")).unwrap(),
            "https://github.com/org/repo/edit/main/docs/a.md"
        );
    }

    #[test]
    fn configured_root_prefixes_path() {
        let root = Path::new("/p");
        let mut c = config("https://github.com/org/repo");
        c.root = Some("/site/docs/".to_string());
        assert_eq!(
            links(&c, root).url_for(&root.join("docs/a.md")).unwrap(),
            "https://github.com/org/repo/edit/main/site/docs/docs/a.md"
        );
    }

    #[test]
    fn root_is_detected_from_git_dir() {
        let repo = tempfile::tempdir().unwrap();
        std::fs::create_dir(repo.path().join(".git")).unwrap();
        let project = repo.path().join("site");
        std::fs::create_dir(&project).unwrap();

        let mut c = config("https://github.com/org/repo");
        c.root = None;
        let l = links(&c, &project);
        assert_eq!(
            l.url_for(&project.join("docs/a.md")).unwrap(),
            "https://github.com/org/repo/edit/main/site/docs/a.md"
        );
    }

    #[test]
    fn root_detection_accepts_git_file() {
        // Worktrees and submodules have a `.git` file rather than a directory.
        let repo = tempfile::tempdir().unwrap();
        std::fs::write(repo.path().join(".git"), "gitdir: elsewhere").unwrap();

        let mut c = config("https://github.com/org/repo");
        c.root = None;
        let l = links(&c, repo.path());
        assert_eq!(
            l.url_for(&repo.path().join("docs/a.md")).unwrap(),
            "https://github.com/org/repo/edit/main/docs/a.md"
        );
    }

    #[test]
    fn path_segments_are_percent_encoded() {
        let root = Path::new("/p");
        let l = links(&config("https://github.com/org/repo"), root);
        assert_eq!(
            l.url_for(&root.join("docs/my page#1.md")).unwrap(),
            "https://github.com/org/repo/edit/main/docs/my%20page%231.md"
        );
    }

    #[test]
    fn branch_with_slash_is_kept() {
        let root = Path::new("/p");
        let mut c = config("https://github.com/org/repo");
        c.branch = "release/v2".to_string();
        assert_eq!(
            links(&c, root).url_for(&root.join("docs/a.md")).unwrap(),
            "https://github.com/org/repo/edit/release/v2/docs/a.md"
        );
    }

    #[test]
    fn source_outside_project_has_no_url() {
        let l = links(&config("https://github.com/org/repo"), Path::new("/p"));
        assert!(l.url_for(Path::new("/elsewhere/a.md")).is_none());
    }
}
