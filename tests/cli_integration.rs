mod integration_helpers;

use assert_cmd::Command;
use integration_helpers::{DEFAULT_CONFIG, create_project};
use predicates::prelude::*;

#[allow(deprecated)]
fn docanvil_cmd() -> Command {
    Command::cargo_bin("docanvil").expect("binary should exist")
}

#[test]
fn test_cli_build_success() {
    let dir = create_project(DEFAULT_CONFIG, &[("index.md", "# Hello\n\nWorld.")]);

    docanvil_cmd()
        .args(["build", "--path"])
        .arg(dir.path())
        .arg("--quiet")
        .assert()
        .success();

    assert!(dir.path().join("dist/index.html").exists());
}

#[test]
fn test_cli_build_missing_project() {
    docanvil_cmd()
        .args(["build", "--path", "/tmp/docanvil_nonexistent_project_dir"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_cli_build_strict_broken_link() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[("index.md", "# Home\n\nBroken [[nonexistent]] link.")],
    );

    docanvil_cmd()
        .args(["build", "--path"])
        .arg(dir.path())
        .args(["--strict", "--quiet"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("warning"));
}

const PUBLIC_OUTPUT_CONFIG: &str = r#"
[project]
name = "Test Docs"

[build]
output_dir = "public"
"#;

#[test]
fn test_cli_build_uses_config_output_dir_by_default() {
    let dir = create_project(PUBLIC_OUTPUT_CONFIG, &[("index.md", "# Hello")]);

    docanvil_cmd()
        .args(["build", "--path"])
        .arg(dir.path())
        .arg("--quiet")
        .assert()
        .success();

    assert!(dir.path().join("public/index.html").exists());
    assert!(!dir.path().join("dist").exists());
}

#[test]
fn test_cli_build_explicit_out_overrides_config() {
    let dir = create_project(PUBLIC_OUTPUT_CONFIG, &[("index.md", "# Hello")]);

    // `--out dist` matches the old clap default, which used to be ignored.
    docanvil_cmd()
        .current_dir(dir.path())
        .args(["build", "--out", "dist", "--quiet"])
        .assert()
        .success();

    assert!(dir.path().join("dist/index.html").exists());
    assert!(!dir.path().join("public").exists());
}

#[test]
fn test_cli_build_clean_refuses_project_root() {
    let dir = create_project(DEFAULT_CONFIG, &[("index.md", "# Hello")]);

    docanvil_cmd()
        .current_dir(dir.path())
        .args(["build", "--out", ".", "--clean", "--quiet"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "refusing to remove output directory",
        ));

    // Nothing was deleted.
    assert!(dir.path().join("docanvil.toml").exists());
    assert!(dir.path().join("docs/index.md").exists());
}

#[test]
fn test_cli_update_help_lists_flags() {
    docanvil_cmd()
        .args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--check"))
        .stdout(predicate::str::contains("--yes"))
        .stdout(predicate::str::contains("--version"));
}

#[test]
fn test_cli_update_rejects_invalid_version_without_network() {
    // Validation happens before any request, so this is offline-safe.
    docanvil_cmd()
        .args(["update", "--version", "latest"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("isn't a valid version"));
}

#[test]
fn test_cli_component_list_shows_builtins_and_custom() {
    let dir = tempfile::tempdir().unwrap();
    let comps = dir.path().join("theme/components");
    std::fs::create_dir_all(&comps).unwrap();
    std::fs::write(comps.join("card.html"), "<div></div>").unwrap();
    std::fs::write(comps.join("note.html"), "<aside></aside>").unwrap();

    let output = docanvil_cmd()
        .args(["component", "list", "--path"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tabs"), "{stdout}");
    assert!(stdout.contains("overridden"), "{stdout}");
    assert!(stdout.contains("card"), "{stdout}");
}

#[test]
fn test_cli_component_eject_writes_and_skips_existing() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("theme/components/tabs.html");

    let status = docanvil_cmd()
        .args(["component", "eject", "tabs", "--path"])
        .arg(dir.path())
        .output()
        .unwrap()
        .status;
    assert!(status.success());
    let ejected = std::fs::read_to_string(&target).unwrap();
    assert!(ejected.contains("tab-header"));

    std::fs::write(&target, "mine").unwrap();
    let output = docanvil_cmd()
        .args(["component", "eject", "tabs", "--path"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "mine");
    assert!(String::from_utf8_lossy(&output.stderr).contains("--force"));

    let status = docanvil_cmd()
        .args(["component", "eject", "tabs", "--force", "--path"])
        .arg(dir.path())
        .output()
        .unwrap()
        .status;
    assert!(status.success());
    assert!(
        std::fs::read_to_string(&target)
            .unwrap()
            .contains("tab-header")
    );
}

#[test]
fn test_cli_component_eject_all() {
    let dir = tempfile::tempdir().unwrap();
    let status = docanvil_cmd()
        .args(["component", "eject", "--all", "--path"])
        .arg(dir.path())
        .output()
        .unwrap()
        .status;
    assert!(status.success());
    for name in [
        "note",
        "warning",
        "lozenge",
        "mermaid",
        "tabs",
        "code-group",
        "hero",
        "buttons",
        "features",
        "feature",
    ] {
        assert!(
            dir.path()
                .join(format!("theme/components/{name}.html"))
                .exists(),
            "{name}"
        );
    }
}

#[test]
fn test_cli_component_eject_unknown_name_fails() {
    let dir = tempfile::tempdir().unwrap();
    let output = docanvil_cmd()
        .args(["component", "eject", "nte", "--path"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("nte") && stderr.contains("note"),
        "{stderr}"
    );
}

#[test]
fn test_cli_component_eject_requires_a_name() {
    let dir = tempfile::tempdir().unwrap();
    let output = docanvil_cmd()
        .args(["component", "eject", "--path"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn test_cli_doctor_reports_name_each_file_once_with_relative_path() {
    let dir = create_project(
        DEFAULT_CONFIG,
        &[(
            "index.md",
            "# Home\n\nSee [[nowhere]].\n\n:::include{file=\"_nope.md\"}\n",
        )],
    );

    for format in ["checkstyle", "junit"] {
        let output = docanvil_cmd()
            .current_dir(dir.path())
            .args(["doctor", "--path", ".", "--format", format])
            .output()
            .unwrap();
        let report = String::from_utf8(output.stdout).unwrap();
        let root = dir.path().canonicalize().unwrap();
        assert!(
            !report.contains(&*root.to_string_lossy())
                && !report.contains(&*dir.path().to_string_lossy()),
            "{format} report has absolute paths:\n{report}"
        );
        assert!(report.contains("broken-wiki-link") && report.contains("include-unresolved"));
        if format == "checkstyle" {
            assert_eq!(
                report.matches("<file name=\"docs/index.md\">").count(),
                1,
                "{report}"
            );
        }
    }
}
