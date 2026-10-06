//! Self-update support: finding releases, verified downloads and replacing
//! the running binary. Shared by `docanvil update` and the `serve` notice.

use semver::Version;

use crate::error::Error;

/// Web root of the DocAnvil repository; release downloads hang off this.
pub const REPO_WEB: &str = "https://github.com/docanvil/docanvil";
/// GitHub REST API root for the repository (checksum fallback only).
pub const REPO_API: &str = "https://api.github.com/repos/docanvil/docanvil";

/// The version of the running binary.
pub fn current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("CARGO_PKG_VERSION is valid semver")
}

/// Map an OS / architecture / libc combination to a published release target.
pub fn target_for(os: &str, arch: &str, musl: bool) -> Option<&'static str> {
    match (os, arch, musl) {
        ("linux", "x86_64", true) => Some("x86_64-unknown-linux-musl"),
        ("linux", "x86_64", false) => Some("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64", false) => Some("aarch64-unknown-linux-gnu"),
        ("macos", "aarch64", _) => Some("aarch64-apple-darwin"),
        ("macos", "x86_64", _) => Some("x86_64-apple-darwin"),
        ("windows", "x86_64", _) => Some("x86_64-pc-windows-msvc"),
        _ => None,
    }
}

/// The release target this binary was built for, so a musl build updates to
/// musl and an Intel Mac build updates to Intel. `None` for self-built
/// binaries on platforms we don't publish.
pub fn current_target() -> Option<&'static str> {
    target_for(
        std::env::consts::OS,
        std::env::consts::ARCH,
        cfg!(target_env = "musl"),
    )
}

pub fn is_windows_target(target: &str) -> bool {
    target.contains("windows")
}

/// Release asset file name, e.g. `docanvil-v1.2.0-aarch64-apple-darwin.tar.gz`.
pub fn asset_name(version: &Version, target: &str) -> String {
    let ext = if is_windows_target(target) {
        "zip"
    } else {
        "tar.gz"
    };
    format!("docanvil-v{version}-{target}.{ext}")
}

/// Parse a user- or GitHub-supplied version, with or without a leading `v`.
pub fn parse_version(s: &str) -> Option<Version> {
    let s = s.trim();
    Version::parse(s.strip_prefix('v').unwrap_or(s)).ok()
}

/// Pull the version out of the `releases/latest` redirect target
/// (`.../releases/tag/v1.2.0`).
pub fn parse_tag_from_location(location: &str) -> Option<Version> {
    let tag = location.trim_end_matches('/').rsplit('/').next()?;
    parse_version(tag)
}

/// Find `asset`'s hash in a `sha256sum`-style file. Returns lowercase hex.
pub fn parse_sha256sums(contents: &str, asset: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?.trim_start_matches('*');
        let valid = hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit());
        (name == asset && valid).then(|| hash.to_ascii_lowercase())
    })
}

/// Human-facing release notes page for a version.
pub fn release_url(version: &Version) -> String {
    format!("{REPO_WEB}/releases/tag/v{version}")
}

#[allow(dead_code)] // used from Task 2 onwards
pub(crate) fn update_error(message: impl Into<String>, hint: Option<&str>) -> Error {
    Error::Update {
        message: message.into(),
        hint: hint.map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    #[test]
    fn target_mapping_covers_published_targets() {
        assert_eq!(
            target_for("linux", "x86_64", true),
            Some("x86_64-unknown-linux-musl")
        );
        assert_eq!(
            target_for("linux", "x86_64", false),
            Some("x86_64-unknown-linux-gnu")
        );
        assert_eq!(
            target_for("linux", "aarch64", false),
            Some("aarch64-unknown-linux-gnu")
        );
        assert_eq!(
            target_for("macos", "aarch64", false),
            Some("aarch64-apple-darwin")
        );
        assert_eq!(
            target_for("macos", "x86_64", false),
            Some("x86_64-apple-darwin")
        );
        assert_eq!(
            target_for("windows", "x86_64", false),
            Some("x86_64-pc-windows-msvc")
        );
    }

    #[test]
    fn target_mapping_rejects_unpublished_targets() {
        // No aarch64 musl build is published; a gnu binary won't run there.
        assert_eq!(target_for("linux", "aarch64", true), None);
        assert_eq!(target_for("freebsd", "x86_64", false), None);
        assert_eq!(target_for("windows", "aarch64", false), None);
    }

    #[test]
    fn current_target_is_known_on_supported_platforms() {
        // CI and dev machines are all on published targets.
        assert!(current_target().is_some());
    }

    #[test]
    fn asset_names_match_release_workflow() {
        let v = Version::new(1, 2, 0);
        assert_eq!(
            asset_name(&v, "x86_64-unknown-linux-musl"),
            "docanvil-v1.2.0-x86_64-unknown-linux-musl.tar.gz"
        );
        assert_eq!(
            asset_name(&v, "x86_64-pc-windows-msvc"),
            "docanvil-v1.2.0-x86_64-pc-windows-msvc.zip"
        );
    }

    #[test]
    fn parse_version_accepts_optional_v_prefix() {
        assert_eq!(parse_version("1.2.0"), Some(Version::new(1, 2, 0)));
        assert_eq!(parse_version("v1.2.0"), Some(Version::new(1, 2, 0)));
        assert_eq!(parse_version(" v1.2.0\n"), Some(Version::new(1, 2, 0)));
    }

    #[test]
    fn parse_version_rejects_junk() {
        assert_eq!(parse_version("latest"), None);
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn parse_tag_from_location_reads_last_segment() {
        assert_eq!(
            parse_tag_from_location("https://github.com/docanvil/docanvil/releases/tag/v1.1.3"),
            Some(Version::new(1, 1, 3))
        );
        assert_eq!(
            parse_tag_from_location("/releases/tag/v1.2.0/"),
            Some(Version::new(1, 2, 0))
        );
        // GitHub redirects to /releases when there is no release at all.
        assert_eq!(
            parse_tag_from_location("https://github.com/docanvil/docanvil/releases"),
            None
        );
    }

    #[test]
    fn parse_sha256sums_finds_asset() {
        let hash = "6638463c0ffb197302d10959eff181767630db3f644140d17250dd7324391456";
        let sums = format!(
            "{hash}  docanvil-v1.1.3-aarch64-apple-darwin.tar.gz\n\
             {}  docanvil-v1.1.3-x86_64-pc-windows-msvc.zip\n",
            "a".repeat(64)
        );
        assert_eq!(
            parse_sha256sums(&sums, "docanvil-v1.1.3-aarch64-apple-darwin.tar.gz").as_deref(),
            Some(hash)
        );
    }

    #[test]
    fn parse_sha256sums_handles_binary_marker_and_case() {
        let sums = format!("{}  *docanvil.zip\n", "AB".repeat(32));
        assert_eq!(
            parse_sha256sums(&sums, "docanvil.zip"),
            Some("ab".repeat(32))
        );
    }

    #[test]
    fn parse_sha256sums_rejects_missing_or_malformed() {
        let sums = format!(
            "{}  other.tar.gz\nnothex  docanvil.tar.gz\n",
            "a".repeat(64)
        );
        assert_eq!(parse_sha256sums(&sums, "docanvil.tar.gz"), None);
        assert_eq!(parse_sha256sums(&sums, "missing.tar.gz"), None);
    }

    #[test]
    fn current_version_matches_cargo() {
        assert_eq!(current_version().to_string(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn release_url_points_at_tag() {
        assert_eq!(
            release_url(&Version::new(1, 2, 0)),
            "https://github.com/docanvil/docanvil/releases/tag/v1.2.0"
        );
    }
}
