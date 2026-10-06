//! One-line "new version available" notice for `docanvil serve`. Cached for
//! a day, runs in the background, and never reports its own failures.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use owo_colors::OwoColorize;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::update::{Source, current_version, latest_version, parse_version};

const TTL_SECS: u64 = 24 * 60 * 60;
const NOTICE_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Serialize, Deserialize)]
struct Cache {
    checked_at: u64,
    latest: String,
}

pub fn enabled(quiet: bool, env_set: impl Fn(&str) -> bool) -> bool {
    !quiet && !env_set("CI") && !env_set("DOCANVIL_NO_UPDATE_CHECK")
}

fn cache_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|dir| dir.join("docanvil").join("update-check.json"))
}

/// The cached latest version, if the cache exists, parses and is under a day old.
pub fn cached_latest(path: &Path, now: u64) -> Option<Version> {
    let cache: Cache = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    if now.saturating_sub(cache.checked_at) > TTL_SECS {
        return None;
    }
    parse_version(&cache.latest)
}

/// Best effort; a read-only cache dir just means we check again next time.
pub fn write_cache(path: &Path, now: u64, latest: &Version) {
    let cache = Cache {
        checked_at: now,
        latest: latest.to_string(),
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_vec(&cache) {
        let _ = std::fs::write(path, json);
    }
}

pub fn message(current: &Version, latest: &Version) -> Option<String> {
    (latest > current).then(|| {
        format!(
            "✨ DocAnvil {} is available (you're on v{current}) — run {}",
            format!("v{latest}").green().bold(),
            "docanvil update".cyan()
        )
    })
}

fn check() -> Option<String> {
    let path = cache_path()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let latest = match cached_latest(&path, now) {
        Some(latest) => latest,
        None => {
            let latest = latest_version(&Source::github(), NOTICE_TIMEOUT).ok()?;
            write_cache(&path, now, &latest);
            latest
        }
    };
    message(&current_version(), &latest)
}

/// Check for a newer release on a background thread so `serve` never waits.
pub fn spawn(quiet: bool) {
    if !enabled(quiet, |key| std::env::var_os(key).is_some()) {
        return;
    }
    std::thread::spawn(|| {
        if let Some(msg) = check() {
            eprintln!("{msg}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_by_quiet_ci_or_opt_out() {
        assert!(enabled(false, |_| false));
        assert!(!enabled(true, |_| false));
        assert!(!enabled(false, |k| k == "CI"));
        assert!(!enabled(false, |k| k == "DOCANVIL_NO_UPDATE_CHECK"));
    }

    #[test]
    fn cache_round_trip_and_expiry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/update-check.json");
        let now = 1_000_000;
        assert_eq!(cached_latest(&path, now), None);

        write_cache(&path, now, &Version::new(1, 2, 0));
        assert_eq!(cached_latest(&path, now + 60), Some(Version::new(1, 2, 0)));
        assert_eq!(cached_latest(&path, now + TTL_SECS + 1), None);
    }

    #[test]
    fn corrupt_cache_is_treated_as_stale() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("update-check.json");
        std::fs::write(&path, "{ not json").unwrap();
        assert_eq!(cached_latest(&path, 0), None);
        std::fs::write(&path, r#"{"checked_at": 0, "latest": "banana"}"#).unwrap();
        assert_eq!(cached_latest(&path, 0), None);
    }

    #[test]
    fn message_only_when_newer() {
        let msg = message(&Version::new(1, 1, 3), &Version::new(1, 2, 0)).unwrap();
        assert!(msg.contains("1.2.0") && msg.contains("1.1.3") && msg.contains("docanvil update"));
        assert_eq!(
            message(&Version::new(1, 2, 0), &Version::new(1, 2, 0)),
            None
        );
        assert_eq!(
            message(&Version::new(1, 3, 0), &Version::new(1, 2, 0)),
            None
        );
    }
}
