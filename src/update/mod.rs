//! Self-update support: finding releases, verified downloads and replacing
//! the running binary. Shared by `docanvil update` and the `serve` notice.

pub mod notice;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use semver::Version;
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

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

pub(crate) fn update_error(message: impl Into<String>, hint: Option<&str>) -> Error {
    Error::Update {
        message: message.into(),
        hint: hint.map(str::to_string),
    }
}

/// Timeout for a version check from `docanvil update`.
pub const CHECK_TIMEOUT: Duration = Duration::from_secs(10);
/// Overall deadline for downloading a release archive or checksum file.
/// Generous so slow links can still finish; dead hosts fail fast on connect.
pub const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);
/// How long to wait for a TCP/TLS connection before giving up.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Release archives are ~5 MB; anything near this is not ours.
const MAX_DOWNLOAD_BYTES: u64 = 200 * 1024 * 1024;

const NETWORK_HINT: &str = "Check your internet connection, or try again in a minute.";

/// Where releases come from. Tests point this at a local server.
pub struct Source {
    /// e.g. `https://github.com/docanvil/docanvil`
    pub web_base: String,
    /// e.g. `https://api.github.com/repos/docanvil/docanvil`
    pub api_base: String,
}

impl Source {
    pub fn github() -> Self {
        Source {
            web_base: REPO_WEB.to_string(),
            api_base: REPO_API.to_string(),
        }
    }
}

fn agent(timeout: Duration, follow_redirects: bool) -> ureq::Agent {
    let mut config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .timeout_connect(Some(CONNECT_TIMEOUT.min(timeout)))
        .user_agent(concat!("docanvil/", env!("CARGO_PKG_VERSION")));
    if !follow_redirects {
        config = config.max_redirects(0).http_status_as_error(false);
    }
    config.build().into()
}

fn network_error(url: &str, e: ureq::Error) -> Error {
    update_error(format!("couldn't reach {url}: {e}"), Some(NETWORK_HINT))
}

/// GET `url`, following redirects. `Ok(None)` on 404.
fn fetch(url: &str, timeout: Duration) -> Result<Option<Vec<u8>>> {
    match agent(timeout, true).get(url).call() {
        Ok(mut resp) => resp
            .body_mut()
            .with_config()
            .limit(MAX_DOWNLOAD_BYTES)
            .read_to_vec()
            .map(Some)
            .map_err(|e| network_error(url, e)),
        Err(ureq::Error::StatusCode(404)) => Ok(None),
        Err(ureq::Error::StatusCode(code @ (403 | 429))) => Err(update_error(
            format!("GitHub refused the request to {url} (HTTP {code})"),
            Some("GitHub's API has a rate limit of 60 requests an hour; try again later."),
        )),
        Err(ureq::Error::StatusCode(code)) => Err(update_error(
            format!("GitHub returned HTTP {code} for {url}"),
            Some("GitHub may be having trouble; try again in a few minutes."),
        )),
        Err(e) => Err(network_error(url, e)),
    }
}

/// Latest released version, read from the `releases/latest` redirect so we
/// never touch the rate-limited API.
pub fn latest_version(source: &Source, timeout: Duration) -> Result<Version> {
    let url = format!("{}/releases/latest", source.web_base);
    let resp = agent(timeout, false)
        .get(&url)
        .call()
        .map_err(|e| network_error(&url, e))?;
    resp.headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .and_then(parse_tag_from_location)
        .ok_or_else(|| {
            update_error(
                format!(
                    "couldn't work out the latest DocAnvil release from {url} (status {})",
                    resp.status()
                ),
                Some(NETWORK_HINT),
            )
        })
}

/// SHA-256 for `asset` from a GitHub release API response (`sha256:<hex>`).
pub fn parse_api_digest(json: &[u8], asset: &str) -> Option<String> {
    let release: serde_json::Value = serde_json::from_slice(json).ok()?;
    release["assets"]
        .as_array()?
        .iter()
        .find(|a| a["name"] == asset)?["digest"]
        .as_str()?
        .strip_prefix("sha256:")
        .map(str::to_ascii_lowercase)
}

fn release_not_found(version: &Version) -> Error {
    update_error(
        format!("DocAnvil release v{version} not found"),
        Some("Check the version number at https://github.com/docanvil/docanvil/releases"),
    )
}

/// Confirm a specific release exists before offering it, so `--check` and the
/// confirmation prompt never describe a version that isn't there.
pub fn ensure_release_exists(source: &Source, version: &Version) -> Result<()> {
    let sums_url = format!(
        "{}/releases/download/v{version}/SHA256SUMS",
        source.web_base
    );
    if fetch(&sums_url, CHECK_TIMEOUT)?.is_some() {
        return Ok(());
    }
    let api_url = format!("{}/releases/tags/v{version}", source.api_base);
    match fetch(&api_url, CHECK_TIMEOUT)? {
        Some(_) => Ok(()),
        None => Err(release_not_found(version)),
    }
}

/// The published SHA-256 for `asset`: `SHA256SUMS` first, then GitHub's
/// per-asset digest for releases that predate the sums file.
pub fn expected_digest(source: &Source, version: &Version, asset: &str) -> Result<String> {
    let sums_url = format!(
        "{}/releases/download/v{version}/SHA256SUMS",
        source.web_base
    );
    if let Some(sums) = fetch(&sums_url, DOWNLOAD_TIMEOUT)? {
        return parse_sha256sums(&String::from_utf8_lossy(&sums), asset).ok_or_else(|| {
            update_error(
                format!("SHA256SUMS for v{version} has no entry for {asset}"),
                None,
            )
        });
    }

    let api_url = format!("{}/releases/tags/v{version}", source.api_base);
    let json = fetch(&api_url, CHECK_TIMEOUT)?.ok_or_else(|| release_not_found(version))?;
    parse_api_digest(&json, asset).ok_or_else(|| {
        update_error(
            format!("release v{version} has no checksum for {asset}, so it can't be verified"),
            None,
        )
    })
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Download the archive for `version`/`target` and verify its checksum.
/// Nothing is written to disk.
pub fn download_verified(source: &Source, version: &Version, target: &str) -> Result<Vec<u8>> {
    let asset = asset_name(version, target);
    let expected = expected_digest(source, version, &asset)?;

    let url = format!("{}/releases/download/v{version}/{asset}", source.web_base);
    let bytes = fetch(&url, DOWNLOAD_TIMEOUT)?.ok_or_else(|| {
        update_error(
            format!("release v{version} has no download for {target}"),
            None,
        )
    })?;

    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(update_error(
            format!("checksum mismatch for {asset} (expected {expected}, got {actual})"),
            Some(
                "The download may be corrupted. Try again; if it keeps failing, please open an issue.",
            ),
        ));
    }
    Ok(bytes)
}

/// Pull the `docanvil` binary out of a release archive.
pub fn extract_binary(archive: &[u8], target: &str) -> Result<Vec<u8>> {
    let windows = is_windows_target(target);
    let bin_name = if windows { "docanvil.exe" } else { "docanvil" };
    let is_bin = |path: &Path| path.file_name().is_some_and(|n| n == bin_name);
    let corrupt = |e: &dyn std::fmt::Display| {
        update_error(format!("couldn't unpack the downloaded archive: {e}"), None)
    };

    let mut found = None;
    if windows {
        let mut zip =
            zip::ZipArchive::new(std::io::Cursor::new(archive)).map_err(|e| corrupt(&e))?;
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).map_err(|e| corrupt(&e))?;
            if file.is_file() && is_bin(Path::new(file.name())) {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes).map_err(|e| corrupt(&e))?;
                found = Some(bytes);
                break;
            }
        }
    } else {
        let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(archive));
        for entry in tar.entries().map_err(|e| corrupt(&e))? {
            let mut entry = entry.map_err(|e| corrupt(&e))?;
            let path = entry.path().map_err(|e| corrupt(&e))?.into_owned();
            if entry.header().entry_type().is_file() && is_bin(&path) {
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes).map_err(|e| corrupt(&e))?;
                found = Some(bytes);
                break;
            }
        }
    }

    found.ok_or_else(|| {
        update_error(
            format!("the downloaded archive didn't contain {bin_name}"),
            None,
        )
    })
}

/// How the running binary was installed.
#[derive(Debug, PartialEq, Eq)]
pub enum InstallKind {
    /// Installer script, manual download, or anything else we can replace.
    Standalone,
    /// `cargo install`: replacing it would leave cargo's records stale.
    Cargo,
}

/// `$CARGO_HOME/bin`, defaulting to `~/.cargo/bin`.
pub fn cargo_bin_dir() -> Option<PathBuf> {
    std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".cargo")))
        .map(|cargo_home| cargo_home.join("bin"))
}

pub fn install_kind(exe: &Path, cargo_bin: Option<&Path>) -> InstallKind {
    let Some(cargo_bin) = cargo_bin else {
        return InstallKind::Standalone;
    };
    let exe = exe.canonicalize().unwrap_or_else(|_| exe.to_path_buf());
    let cargo_bin = cargo_bin
        .canonicalize()
        .unwrap_or_else(|_| cargo_bin.to_path_buf());
    if exe.starts_with(&cargo_bin) {
        InstallKind::Cargo
    } else {
        InstallKind::Standalone
    }
}

fn permission_hint(windows: bool) -> &'static str {
    if windows {
        "Run 'docanvil update' from an Administrator terminal, or reinstall to a user directory with the install script."
    } else {
        "Run 'sudo docanvil update', or reinstall to a user directory with the install script."
    }
}

/// Fail early, with a useful hint, if we can't write next to the binary.
pub fn check_writable(dir: &Path) -> Result<()> {
    let probe = dir.join(format!(".docanvil-write-probe-{}", std::process::id()));
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
    {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => Err(update_error(
            format!("no permission to replace docanvil in {}", dir.display()),
            Some(permission_hint(cfg!(windows))),
        )),
        Err(e) => Err(e.into()),
    }
}

/// A staged binary that is deleted when dropped, so every exit path, including
/// a write that fails halfway, cleans up after itself.
struct StagedFile(PathBuf);

impl StagedFile {
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Write the new binary next to the old one. Permissions don't matter here:
/// `self_replace` copies it into place with the running binary's permissions.
fn stage_binary(dir: &Path, bytes: &[u8]) -> Result<StagedFile> {
    let staged = StagedFile(dir.join(format!(".docanvil-update-{}", std::process::id())));
    std::fs::write(staged.path(), bytes)?;
    Ok(staged)
}

/// Replace the running executable with `bytes`. The new binary is written
/// next to the old one first, so a failure leaves the original untouched.
pub fn install_binary(bytes: &[u8]) -> Result<()> {
    let exe = std::env::current_exe()?;
    let exe = exe.canonicalize().unwrap_or(exe);
    let dir = exe.parent().ok_or_else(|| {
        update_error("couldn't find the directory docanvil is installed in", None)
    })?;
    check_writable(dir)?;

    let staged = stage_binary(dir, bytes)?;
    let result = self_replace::self_replace(staged.path());
    result.map_err(|e| update_error(format!("couldn't replace {}: {e}", exe.display()), None))
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
    fn parse_api_digest_finds_asset() {
        let json = br#"{"tag_name":"v1.1.3","assets":[
            {"name":"a.zip","digest":"sha256:aaaa"},
            {"name":"docanvil-v1.1.3-x86_64-unknown-linux-gnu.tar.gz","uploader":{"login":"x"},
             "digest":"sha256:6F2ACCCF"}]}"#;
        assert_eq!(
            parse_api_digest(json, "docanvil-v1.1.3-x86_64-unknown-linux-gnu.tar.gz").as_deref(),
            Some("6f2acccf")
        );
        assert_eq!(parse_api_digest(json, "missing.tar.gz"), None);
        assert_eq!(parse_api_digest(b"not json", "a.zip"), None);
    }

    #[test]
    fn sha256_hex_matches_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn download_timeout_allows_slow_links() {
        // A ~5 MB archive at 256 kbit/s takes ~160 s.
        assert!(DOWNLOAD_TIMEOUT >= Duration::from_secs(300));
    }

    #[test]
    fn release_url_points_at_tag() {
        assert_eq!(
            release_url(&Version::new(1, 2, 0)),
            "https://github.com/docanvil/docanvil/releases/tag/v1.2.0"
        );
    }

    use std::io::Write;

    fn tar_gz(name: &str, data: &[u8]) -> Vec<u8> {
        let gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut builder = tar::Builder::new(gz);
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        builder.append_data(&mut header, name, data).unwrap();
        builder.into_inner().unwrap().finish().unwrap()
    }

    fn zip_file(name: &str, data: &[u8]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(data).unwrap();
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn extract_binary_from_tar_gz() {
        let archive = tar_gz("docanvil", b"unix-bin");
        assert_eq!(
            extract_binary(&archive, "x86_64-unknown-linux-musl").unwrap(),
            b"unix-bin"
        );
    }

    #[test]
    fn extract_binary_from_nested_tar_path() {
        let archive = tar_gz("docanvil-v1.2.0/docanvil", b"nested");
        assert_eq!(
            extract_binary(&archive, "aarch64-apple-darwin").unwrap(),
            b"nested"
        );
    }

    #[test]
    fn extract_binary_from_zip() {
        let archive = zip_file("docanvil.exe", b"win-bin");
        assert_eq!(
            extract_binary(&archive, "x86_64-pc-windows-msvc").unwrap(),
            b"win-bin"
        );
    }

    #[test]
    fn extract_binary_errors_when_binary_missing() {
        let archive = tar_gz("README.md", b"hello");
        let err = extract_binary(&archive, "x86_64-unknown-linux-gnu").unwrap_err();
        assert!(err.to_string().contains("didn't contain docanvil"), "{err}");
    }

    #[test]
    fn extract_binary_errors_on_garbage() {
        assert!(extract_binary(b"not an archive", "x86_64-unknown-linux-gnu").is_err());
        assert!(extract_binary(b"not an archive", "x86_64-pc-windows-msvc").is_err());
    }

    #[test]
    fn install_kind_detects_cargo_bin() {
        let bin = Path::new("/home/u/.cargo/bin");
        assert_eq!(
            install_kind(Path::new("/home/u/.cargo/bin/docanvil"), Some(bin)),
            InstallKind::Cargo
        );
        assert_eq!(
            install_kind(Path::new("/home/u/.local/bin/docanvil"), Some(bin)),
            InstallKind::Standalone
        );
        assert_eq!(
            install_kind(Path::new("/home/u/.cargo/bin/docanvil"), None),
            InstallKind::Standalone
        );
    }

    #[test]
    fn install_kind_resolves_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let cargo_bin = dir.path().join("cargo/bin");
        std::fs::create_dir_all(&cargo_bin).unwrap();
        std::fs::write(cargo_bin.join("docanvil"), b"x").unwrap();
        #[cfg(unix)]
        {
            let link = dir.path().join("docanvil-link");
            std::os::unix::fs::symlink(cargo_bin.join("docanvil"), &link).unwrap();
            assert_eq!(install_kind(&link, Some(&cargo_bin)), InstallKind::Cargo);
        }
    }

    #[test]
    fn check_writable_accepts_temp_dir() {
        let dir = tempfile::tempdir().unwrap();
        check_writable(dir.path()).unwrap();
        // The probe file is cleaned up.
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn check_writable_rejects_read_only_dir_with_sudo_hint() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
        // Root ignores permissions; nothing to assert there.
        if std::fs::write(dir.path().join("probe"), b"").is_ok() {
            return;
        }
        let err = check_writable(dir.path()).unwrap_err();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(err.hint().unwrap().contains("sudo"), "{:?}", err.hint());
    }

    #[test]
    fn staged_binary_is_written_then_removed_on_drop() {
        let dir = tempfile::tempdir().unwrap();
        let staged = stage_binary(dir.path(), b"new-bin").unwrap();
        assert_eq!(std::fs::read(staged.path()).unwrap(), b"new-bin");
        drop(staged);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn permission_hint_matches_platform() {
        assert!(permission_hint(false).contains("sudo"));
        let windows = permission_hint(true);
        assert!(windows.contains("Administrator") && !windows.contains("sudo"));
    }
}
