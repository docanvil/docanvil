//! "Last updated" dates for pages: from Git history, overridable in front matter.

use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::path::{Path, PathBuf};

/// A calendar date (UTC), shown as `YYYY-MM-DD`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Date {
    /// The UTC calendar date of a Unix timestamp (Howard Hinnant's civil-from-days).
    pub fn from_unix(secs: i64) -> Self {
        let days = secs.div_euclid(86_400);
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
        Self { year, month, day }
    }

    /// Parse a strict `YYYY-MM-DD` date, rejecting days that don't exist.
    pub fn parse(s: &str) -> Option<Self> {
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let digits = |range: std::ops::Range<usize>| -> Option<u32> {
            let part = &s[range];
            if !part.bytes().all(|c| c.is_ascii_digit()) {
                return None;
            }
            part.parse().ok()
        };
        let year = digits(0..4)? as i32;
        let month = digits(5..7)? as u8;
        let day = digits(8..10)? as u8;
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// A page's front matter `last_updated` value.
#[derive(Debug, PartialEq, Eq)]
pub enum Override {
    /// `"YYYY-MM-DD"`: use this date instead of the computed one.
    Date(Date),
    /// `false`: show no date on this page.
    Hide,
    /// Anything else. Ignored by the build; reported by `docanvil doctor`.
    Invalid,
}

/// Interpret a front matter `last_updated` value.
pub fn parse_override(value: &serde_json::Value) -> Override {
    match value {
        serde_json::Value::Bool(false) => Override::Hide,
        serde_json::Value::String(s) => Date::parse(s).map_or(Override::Invalid, Override::Date),
        _ => Override::Invalid,
    }
}

/// Where computed dates come from. Front matter overrides are applied by the caller,
/// so a source only answers for files.
pub trait DateSource {
    fn date_for(&mut self, path: &Path) -> Option<Date>;
}

/// The source for `source = "front-matter"`: never has a date of its own.
pub struct NoDates;

impl DateSource for NoDates {
    fn date_for(&mut self, _path: &Path) -> Option<Date> {
        None
    }
}

/// Dates from Git history: the newest author date of each file, read with one
/// `git log` over the project directory. Files outside the project (pulled in
/// with `file="../…"`) are looked up one at a time, then remembered.
pub struct GitDates {
    project_root: PathBuf,
    dates: HashMap<PathBuf, Date>,
    looked_up: HashMap<PathBuf, Option<Date>>,
    shallow: bool,
}

impl GitDates {
    /// Read the project's Git history. `Err` holds a user-facing message when
    /// there's no repository or no `git` binary.
    pub fn collect(project_root: &Path) -> std::result::Result<Self, String> {
        let project_root = canonical(project_root);
        let top = run_git(&project_root, &["rev-parse", "--show-toplevel"])
            .map_err(|e| format!("[last_updated] can't read Git history: {e}"))?;
        let repo_root = canonical(Path::new(top.trim()));
        let shallow = run_git(&repo_root, &["rev-parse", "--is-shallow-repository"])
            .is_ok_and(|s| s.trim() == "true");
        // Run with cwd = project_root and pathspec "." rather than an absolute path:
        // Git for Windows may not match a `\\?\…` canonicalised path as a pathspec.
        // `--name-only` output stays relative to the repository top level regardless of cwd.
        // A repository without commits yet has no history to read: no dates, no error.
        let log = run_git(
            &project_root,
            &["log", "--format=%x00%at", "--name-only", "-z", "--", "."],
        )
        .unwrap_or_default();
        Ok(Self {
            dates: parse_log(&log, &repo_root),
            project_root,
            looked_up: HashMap::new(),
            shallow,
        })
    }

    /// Whether the clone is shallow, so every file looks last changed in the newest commit.
    pub fn is_shallow(&self) -> bool {
        self.shallow
    }

    fn look_up(&self, path: &Path) -> Option<Date> {
        let dir = path.parent()?;
        let name = path.file_name()?.to_string_lossy().into_owned();
        let out = run_git(dir, &["log", "-1", "--format=%at", "--", &name]).ok()?;
        out.trim().parse::<i64>().ok().map(Date::from_unix)
    }
}

impl DateSource for GitDates {
    fn date_for(&mut self, path: &Path) -> Option<Date> {
        let path = canonical(path);
        if path.starts_with(&self.project_root) {
            return self.dates.get(&path).copied();
        }
        if let Some(found) = self.looked_up.get(&path) {
            return *found;
        }
        let found = self.look_up(&path);
        self.looked_up.insert(path, found);
        found
    }
}

/// Parse `git log --format=%x00%at --name-only -z` output into each path's newest date.
fn parse_log(output: &str, repo_root: &Path) -> HashMap<PathBuf, Date> {
    let mut dates = HashMap::new();
    let mut current: Option<Date> = None;
    let mut expect_timestamp = true;
    for token in output.split('\0') {
        let token = token.strip_prefix('\n').unwrap_or(token);
        if token.is_empty() {
            expect_timestamp = true;
            continue;
        }
        if expect_timestamp {
            current = token.parse::<i64>().ok().map(Date::from_unix);
            expect_timestamp = false;
        } else if let Some(date) = current {
            let mut path = repo_root.to_path_buf();
            path.extend(token.split('/'));
            dates.entry(path).or_insert(date);
        }
    }
    dates
}

/// Run `git` in `dir`; `Err` carries git's stderr, or why it couldn't start.
fn run_git(dir: &Path, args: &[&str]) -> std::result::Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| format!("couldn't run git ({e})"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

/// The newest date across a page's source file and the files it pulls in.
pub fn page_date(
    source: &mut dyn DateSource,
    page: &Path,
    deps: &BTreeSet<PathBuf>,
) -> Option<Date> {
    std::iter::once(page)
        .chain(deps.iter().map(PathBuf::as_path))
        .filter_map(|path| source.date_for(path))
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn d(s: &str) -> Date {
        Date::parse(s).unwrap()
    }

    #[test]
    fn from_unix_known_dates() {
        assert_eq!(Date::from_unix(0).to_string(), "1970-01-01");
        assert_eq!(Date::from_unix(-86_400).to_string(), "1969-12-31");
        assert_eq!(Date::from_unix(951_782_400).to_string(), "2000-02-29");
        assert_eq!(Date::from_unix(1_791_590_400).to_string(), "2026-10-10");
        // Late in the day stays on the same UTC date
        assert_eq!(
            Date::from_unix(1_791_590_400 + 86_399).to_string(),
            "2026-10-10"
        );
        assert_eq!(Date::from_unix(1_767_225_599).to_string(), "2025-12-31");
        assert_eq!(Date::from_unix(1_767_225_600).to_string(), "2026-01-01");
    }

    #[test]
    fn parse_accepts_valid_dates() {
        assert_eq!(d("2026-10-09").to_string(), "2026-10-09");
        assert_eq!(d("2024-02-29").to_string(), "2024-02-29");
        assert_eq!(d("2000-02-29").to_string(), "2000-02-29");
    }

    #[test]
    fn parse_rejects_invalid_dates() {
        for bad in [
            "2026-02-30",
            "2025-02-29",
            "1900-02-29",
            "2026-13-01",
            "2026-00-10",
            "2026-04-31",
            "2026-01-00",
            "26-1-1",
            "2026-1-01",
            "2026-01-01x",
            "2026/01/01",
            "",
            "abcd-ef-gh",
            " 2026-01-01",
        ] {
            assert!(Date::parse(bad).is_none(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn dates_order_chronologically() {
        assert!(d("2026-01-31") < d("2026-02-01"));
        assert!(d("2025-12-31") < d("2026-01-01"));
        assert_eq!(d("2026-03-04").max(d("2026-03-05")), d("2026-03-05"));
    }

    #[test]
    fn override_values() {
        assert_eq!(
            parse_override(&json!("2026-09-30")),
            Override::Date(d("2026-09-30"))
        );
        assert_eq!(parse_override(&json!(false)), Override::Hide);
        assert_eq!(parse_override(&json!(true)), Override::Invalid);
        assert_eq!(parse_override(&json!("2026-9-30")), Override::Invalid);
        assert_eq!(parse_override(&json!(20260930)), Override::Invalid);
        assert_eq!(parse_override(&json!(null)), Override::Invalid);
    }

    struct Fixed(HashMap<PathBuf, Date>);

    impl DateSource for Fixed {
        fn date_for(&mut self, path: &Path) -> Option<Date> {
            self.0.get(path).copied()
        }
    }

    #[test]
    fn page_date_takes_newest_of_page_and_deps() {
        let mut source = Fixed(HashMap::from([
            (PathBuf::from("/p/page.md"), d("2026-01-01")),
            (PathBuf::from("/p/_frag.md"), d("2026-03-01")),
            (PathBuf::from("/p/old.rs"), d("2025-06-01")),
        ]));
        let deps = BTreeSet::from([PathBuf::from("/p/_frag.md"), PathBuf::from("/p/old.rs")]);
        assert_eq!(
            page_date(&mut source, Path::new("/p/page.md"), &deps),
            Some(d("2026-03-01"))
        );
    }

    #[test]
    fn page_date_uses_deps_when_page_has_no_date() {
        let mut source = Fixed(HashMap::from([(
            PathBuf::from("/p/_frag.md"),
            d("2026-03-01"),
        )]));
        let deps = BTreeSet::from([PathBuf::from("/p/_frag.md")]);
        assert_eq!(
            page_date(&mut source, Path::new("/p/untracked.md"), &deps),
            Some(d("2026-03-01"))
        );
    }

    #[test]
    fn page_date_none_when_nothing_has_a_date() {
        let mut source = NoDates;
        assert_eq!(
            page_date(&mut source, Path::new("/p/page.md"), &BTreeSet::new()),
            None
        );
    }

    #[test]
    fn parse_log_newest_first_wins() {
        let out = "\x001770334200\0\ndocs/c.md\0\x001767348000\0\ndocs/a b.md\0docs/c.md\0";
        let map = parse_log(out, Path::new("/repo"));
        assert_eq!(map[Path::new("/repo/docs/c.md")].to_string(), "2026-02-05");
        assert_eq!(
            map[Path::new("/repo/docs/a b.md")].to_string(),
            "2026-01-02"
        );
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn parse_log_handles_commits_without_files_and_unicode() {
        // A merge commit with no files, then a commit touching a unicode path
        let out = "\x001770334200\0\n\x001767348000\0\ndocs/café ☕.md\0";
        let map = parse_log(out, Path::new("/repo"));
        assert_eq!(
            map[Path::new("/repo/docs/café ☕.md")].to_string(),
            "2026-01-02"
        );
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn parse_log_empty_output() {
        assert!(parse_log("", Path::new("/repo")).is_empty());
    }

    fn git_available() -> bool {
        std::process::Command::new("git")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
    }

    fn git(dir: &Path, args: &[&str], author_date: &str) {
        let status = std::process::Command::new("git")
            .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
            .args([
                "-c",
                "commit.gpgsign=false",
                "-c",
                "init.defaultBranch=main",
            ])
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_DATE", author_date)
            // A committer date far from the author date proves we read the author date
            .env("GIT_COMMITTER_DATE", "2030-06-15T12:00:00Z")
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }

    #[test]
    fn collect_reads_author_dates_for_a_project_in_a_subdirectory() {
        if !git_available() {
            return;
        }
        let repo = tempfile::tempdir().unwrap();
        let project = repo.path().join("site");
        std::fs::create_dir_all(project.join("docs")).unwrap();
        std::fs::write(project.join("docs/index.md"), "# Home").unwrap();
        std::fs::write(repo.path().join("README.md"), "outside").unwrap();
        git(repo.path(), &["init", "-q"], "2026-01-02T10:00:00Z");
        git(repo.path(), &["add", "-A"], "2026-01-02T10:00:00Z");
        git(
            repo.path(),
            &["commit", "-q", "-m", "one"],
            "2026-01-02T10:00:00Z",
        );
        std::fs::write(project.join("docs/index.md"), "# Home v2").unwrap();
        git(
            repo.path(),
            &["commit", "-q", "-am", "two"],
            "2026-02-05T23:30:00Z",
        );

        let mut dates = GitDates::collect(&project).unwrap();
        assert!(!dates.is_shallow());
        assert_eq!(
            dates
                .date_for(&project.join("docs/index.md"))
                .map(|d| d.to_string()),
            Some("2026-02-05".to_string())
        );
        // Outside the project dir: answered by the lazy fallback
        assert_eq!(
            dates
                .date_for(&repo.path().join("README.md"))
                .map(|d| d.to_string()),
            Some("2026-01-02".to_string())
        );
        // Untracked
        std::fs::write(project.join("docs/new.md"), "# New").unwrap();
        assert_eq!(dates.date_for(&project.join("docs/new.md")), None);
    }

    #[test]
    fn collect_in_a_repo_without_commits_has_no_dates() {
        if !git_available() {
            return;
        }
        let repo = tempfile::tempdir().unwrap();
        std::fs::write(repo.path().join("index.md"), "# Home").unwrap();
        git(repo.path(), &["init", "-q"], "2026-01-02T10:00:00Z");
        let mut dates = GitDates::collect(repo.path()).unwrap();
        assert_eq!(dates.date_for(&repo.path().join("index.md")), None);
    }

    #[test]
    fn collect_outside_a_repo_is_an_error() {
        if !git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let err = GitDates::collect(dir.path()).err().unwrap();
        assert!(err.contains("[last_updated]"), "{err}");
    }
}
