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

/// The local editor that `docanvil serve` links each page's source to.
///
/// Links use the editor's URL scheme, so the browser hands the file to the
/// editor and the dev server never launches anything itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Editor {
    Vscode,
    Cursor,
    Zed,
    Idea,
    /// A URL template containing `{path}`, e.g. `subl://open?url=file://{path}`.
    Custom(String),
}

impl Editor {
    /// Pick the editor from `--editor`, then `DOCANVIL_EDITOR`, then a guess from
    /// `$VISUAL` / `$EDITOR`, defaulting to VS Code. `None` means links are off
    /// (`none`). The message is a user-facing warning for an unknown name.
    pub fn resolve(flag: Option<&str>) -> (Option<Self>, Option<String>) {
        Self::resolve_with(flag, |key| std::env::var(key).ok())
    }

    fn resolve_with(
        flag: Option<&str>,
        env: impl Fn(&str) -> Option<String>,
    ) -> (Option<Self>, Option<String>) {
        let chosen = flag
            .map(str::to_string)
            .or_else(|| env("DOCANVIL_EDITOR"))
            .filter(|name| !name.trim().is_empty());
        let Some(name) = chosen else {
            let guess = ["VISUAL", "EDITOR"]
                .iter()
                .filter_map(|key| env(key))
                .find_map(|command| Self::from_command(&command));
            return (Some(guess.unwrap_or(Self::Vscode)), None);
        };

        let name = name.trim();
        if name.contains("{path}") {
            return (Some(Self::Custom(name.to_string())), None);
        }
        match name.to_ascii_lowercase().as_str() {
            "none" | "off" => (None, None),
            "vscode" | "code" => (Some(Self::Vscode), None),
            "cursor" => (Some(Self::Cursor), None),
            "zed" => (Some(Self::Zed), None),
            "idea" | "jetbrains" => (Some(Self::Idea), None),
            _ => (
                Some(Self::Vscode),
                Some(format!(
                    "unknown editor \"{name}\", using VS Code. Choose vscode, cursor, zed, \
                     idea or none, or give a URL template containing {{path}}"
                )),
            ),
        }
    }

    /// The editor an `$EDITOR`-style command runs, e.g. `code --wait` → VS Code.
    fn from_command(command: &str) -> Option<Self> {
        let program = command.split_whitespace().next()?;
        let name = Path::new(program)
            .file_stem()?
            .to_str()?
            .to_ascii_lowercase();
        match name.as_str() {
            "code" => Some(Self::Vscode),
            "cursor" => Some(Self::Cursor),
            "zed" | "zeditor" => Some(Self::Zed),
            "idea" | "idea64" => Some(Self::Idea),
            _ => None,
        }
    }

    /// The link that opens `source_path` in this editor, or `None` if the path
    /// can't be made absolute or isn't valid UTF-8.
    pub fn url_for(&self, source_path: &Path) -> Option<String> {
        let path = url_file_path(&std::path::absolute(source_path).ok()?)?;
        Some(match self {
            Self::Vscode => format!("vscode://file{path}"),
            Self::Cursor => format!("cursor://file{path}"),
            Self::Zed => format!("zed://file{path}"),
            Self::Idea => format!("idea://open?file={path}"),
            Self::Custom(template) => template.replace("{path}", &path),
        })
    }
}

/// An absolute path as a percent-encoded URL path: `/home/me/a%20b.md`, or
/// `/C:/docs/a.md` on Windows (the drive's colon is kept).
fn url_file_path(path: &Path) -> Option<String> {
    use std::path::Component;

    let mut out = String::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => {
                let drive = prefix.as_os_str().to_str()?;
                // Verbatim prefixes (`\\?\C:`) carry the drive after the marker.
                let drive = drive.trim_start_matches(r"\\?\");
                out.push('/');
                out.push_str(drive);
            }
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => out.push_str("/.."),
            Component::Normal(segment) => {
                out.push('/');
                out.push_str(&encode_segment(segment.to_str()?));
            }
        }
    }
    Some(out)
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

    fn resolve(flag: Option<&str>, env: &[(&str, &str)]) -> (Option<Editor>, Option<String>) {
        Editor::resolve_with(flag, |key| {
            env.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.to_string())
        })
    }

    #[test]
    fn editor_defaults_to_vscode() {
        assert_eq!(resolve(None, &[]), (Some(Editor::Vscode), None));
    }

    #[test]
    fn editor_flag_beats_env() {
        let env = [("DOCANVIL_EDITOR", "zed"), ("EDITOR", "cursor")];
        assert_eq!(resolve(Some("idea"), &env).0, Some(Editor::Idea));
        assert_eq!(resolve(None, &env).0, Some(Editor::Zed));
    }

    #[test]
    fn editor_guessed_from_visual_then_editor() {
        let env = [
            ("VISUAL", "/usr/local/bin/cursor --wait"),
            ("EDITOR", "code"),
        ];
        assert_eq!(resolve(None, &env).0, Some(Editor::Cursor));
        // A terminal editor tells us nothing, so the next variable is tried.
        let env = [("VISUAL", "vim"), ("EDITOR", "code -w")];
        assert_eq!(resolve(None, &env).0, Some(Editor::Vscode));
        let env = [("EDITOR", "nano")];
        assert_eq!(resolve(None, &env).0, Some(Editor::Vscode));
    }

    #[test]
    fn editor_none_turns_links_off() {
        assert_eq!(resolve(Some("none"), &[]), (None, None));
        assert_eq!(resolve(None, &[("DOCANVIL_EDITOR", "None")]), (None, None));
    }

    #[test]
    fn editor_custom_template() {
        let (editor, warning) = resolve(Some("subl://open?url=file://{path}"), &[]);
        assert!(warning.is_none());
        let editor = editor.unwrap();
        assert_eq!(
            editor,
            Editor::Custom("subl://open?url=file://{path}".to_string())
        );
    }

    #[test]
    fn editor_unknown_name_warns_and_uses_vscode() {
        let (editor, warning) = resolve(Some("emacs"), &[]);
        assert_eq!(editor, Some(Editor::Vscode));
        assert!(warning.unwrap().contains("unknown editor \"emacs\""));
    }

    #[cfg(unix)]
    #[test]
    fn editor_urls_per_scheme() {
        let path = Path::new("/home/me/docs/my page.md");
        let url = |editor: Editor| editor.url_for(path).unwrap();
        assert_eq!(
            url(Editor::Vscode),
            "vscode://file/home/me/docs/my%20page.md"
        );
        assert_eq!(
            url(Editor::Cursor),
            "cursor://file/home/me/docs/my%20page.md"
        );
        assert_eq!(url(Editor::Zed), "zed://file/home/me/docs/my%20page.md");
        assert_eq!(
            url(Editor::Idea),
            "idea://open?file=/home/me/docs/my%20page.md"
        );
        assert_eq!(
            url(Editor::Custom("subl://open?url=file://{path}".into())),
            "subl://open?url=file:///home/me/docs/my%20page.md"
        );
    }

    #[cfg(unix)]
    #[test]
    fn editor_url_makes_relative_paths_absolute() {
        let url = Editor::Vscode.url_for(Path::new("docs/a.md")).unwrap();
        let expected = url_file_path(&std::env::current_dir().unwrap().join("docs/a.md"));
        assert_eq!(url, format!("vscode://file{}", expected.unwrap()));
    }

    #[cfg(windows)]
    #[test]
    fn editor_url_keeps_windows_drive() {
        assert_eq!(
            Editor::Vscode
                .url_for(Path::new(r"C:\Users\me\docs\a b.md"))
                .unwrap(),
            "vscode://file/C:/Users/me/docs/a%20b.md"
        );
        assert_eq!(
            Editor::Vscode
                .url_for(Path::new(r"\\?\C:\Users\me\docs\a.md"))
                .unwrap(),
            "vscode://file/C:/Users/me/docs/a.md"
        );
    }
}
