use std::io::IsTerminal;

use owo_colors::OwoColorize;
use semver::Version;

use crate::error::{Error, Result};
use crate::update::{
    CHECK_TIMEOUT, InstallKind, Source, cargo_bin_dir, current_target, current_version,
    download_verified, ensure_release_exists, extract_binary, install_binary, install_kind,
    latest_version, parse_version, release_url, update_error,
};

/// What `docanvil update` should do.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    UpToDate,
    Install(Version),
}

/// An explicit `--version` always wins (allowing rollbacks); otherwise only a
/// newer release is offered.
pub fn decide(current: &Version, latest: Option<&Version>, requested: Option<&Version>) -> Action {
    match (requested, latest) {
        (Some(want), _) if want == current => Action::UpToDate,
        (Some(want), _) => Action::Install(want.clone()),
        (None, Some(latest)) if latest > current => Action::Install(latest.clone()),
        _ => Action::UpToDate,
    }
}

pub fn run(check: bool, yes: bool, version: Option<&str>, quiet: bool) -> Result<()> {
    let current = current_version();
    let requested = version
        .map(|v| {
            parse_version(v).ok_or_else(|| {
                update_error(
                    format!("'{v}' isn't a valid version"),
                    Some("Use a version like 1.2.0."),
                )
            })
        })
        .transpose()?;

    let source = Source::github();
    let latest = match requested {
        Some(_) => None,
        None => Some(latest_version(&source, CHECK_TIMEOUT)?),
    };

    let target_version = match decide(&current, latest.as_ref(), requested.as_ref()) {
        Action::UpToDate => {
            if !quiet {
                println!("✅ You're on the latest version (v{current})");
            }
            return Ok(());
        }
        Action::Install(v) => v,
    };
    if requested.is_some() {
        ensure_release_exists(&source, &target_version)?;
    }

    if !quiet {
        println!(
            "📦 docanvil {} → {}",
            format!("v{current}").dimmed(),
            format!("v{target_version}").green().bold()
        );
        println!("   Release notes: {}", release_url(&target_version));
    }
    if check {
        return Ok(());
    }

    let exe = std::env::current_exe()?;
    if install_kind(&exe, cargo_bin_dir().as_deref()) == InstallKind::Cargo {
        println!(
            "This copy of docanvil was installed with cargo. To upgrade, run:\n  cargo install docanvil --force"
        );
        return Ok(());
    }

    let target = current_target().ok_or_else(|| {
        update_error(
            "there's no prebuilt docanvil release for this platform",
            Some("Install from source with 'cargo install docanvil --force'."),
        )
    })?;

    if !yes {
        if !std::io::stdin().is_terminal() {
            return Err(update_error(
                "can't ask for confirmation without a terminal",
                Some("Re-run with --yes to upgrade without a prompt."),
            ));
        }
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(format!("Upgrade to v{target_version}?"))
            .default(true)
            .interact()
            .map_err(|e| Error::General(format!("prompt failed: {e}")))?;
        if !confirmed {
            return Ok(());
        }
    }

    if !quiet {
        println!("⬇️  Downloading docanvil v{target_version} ({target})");
    }
    let archive = download_verified(&source, &target_version, target)?;
    let binary = extract_binary(&archive, target)?;
    install_binary(&binary)?;

    if !quiet {
        println!("✅ Updated docanvil v{current} → v{target_version}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    #[test]
    fn newer_latest_installs() {
        assert_eq!(
            decide(&v("1.1.3"), Some(&v("1.2.0")), None),
            Action::Install(v("1.2.0"))
        );
    }

    #[test]
    fn same_or_older_latest_is_up_to_date() {
        assert_eq!(
            decide(&v("1.2.0"), Some(&v("1.2.0")), None),
            Action::UpToDate
        );
        // A local build ahead of the latest release is not "downgraded".
        assert_eq!(
            decide(&v("1.3.0"), Some(&v("1.2.0")), None),
            Action::UpToDate
        );
    }

    #[test]
    fn requested_version_wins_including_downgrade() {
        assert_eq!(
            decide(&v("1.2.0"), None, Some(&v("1.1.0"))),
            Action::Install(v("1.1.0"))
        );
        assert_eq!(
            decide(&v("1.2.0"), None, Some(&v("1.2.0"))),
            Action::UpToDate
        );
    }
}
