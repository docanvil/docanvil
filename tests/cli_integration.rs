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
