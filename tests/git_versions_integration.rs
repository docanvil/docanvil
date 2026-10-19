use std::fs;
use std::path::Path;
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

const CONFIG: &str = r#"
[project]
name = "Combined docs"
[build]
base_url = "/docs/"
site_url = "https://example.com/docs/"
[version]
current = "release"
enabled = ["release", "master"]
[version.git]
refs = { release = "refs/tags/v1.0.0" }
working_tree = "master"
[edit]
repo = "https://github.com/example/repo"
branch = "master"
[last_updated]
enabled = true
[llms]
enabled = true
full = true
[syntax]
enabled = false
"#;

fn fixture(locales: bool) -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let root = repo.path();
    git(root, &["init", "-b", "master"]);
    git(root, &["config", "user.email", "test@example.com"]);
    git(root, &["config", "user.name", "Test"]);
    // Tags predate the new version configuration and use a different content path.
    write(
        root,
        "site/docanvil.toml",
        "[project]\nname = 'Release'\ncontent_dir = 'pages'\n",
    );
    let suffixes = if locales {
        vec![".en", ".fr"]
    } else {
        vec![""]
    };
    for suffix in &suffixes {
        write(
            root,
            &format!("site/pages/index{suffix}.md"),
            "# Released home\n\n:::include{file=\"/fragment.md\"}\n\n```rust file=\"../../example.rs\"\n```\n\n![Diagram](diagram.svg)\n\n[[old-page]]\n",
        );
        write(
            root,
            &format!("site/pages/old-page{suffix}.md"),
            "# Old page\n",
        );
    }
    write(root, "site/fragment.md", "Released fragment\n");
    write(root, "example.rs", "// Released example\n");
    write(root, "site/assets/diagram.svg", "released image");
    write(
        root,
        "site/nav.toml",
        "[[nav]]\npage = 'index'\nlabel = 'Release navigation'\n[[nav]]\npage = 'old-page'\n",
    );
    git(root, &["add", "."]);
    let output = Command::new("git")
        .current_dir(root)
        .env("GIT_AUTHOR_DATE", "2020-01-02T12:00:00Z")
        .env("GIT_COMMITTER_DATE", "2020-01-02T12:00:00Z")
        .args(["commit", "-m", "Release"])
        .output()
        .unwrap();
    assert!(output.status.success());
    git(root, &["tag", "-a", "v1.0.0", "-m", "Release"]);
    fs::remove_dir_all(root.join("site/pages")).unwrap();
    let locale_config = if locales {
        "\n[locale]\ndefault = 'en'\nenabled = ['en', 'fr']\n"
    } else {
        ""
    };
    write(
        root,
        "site/docanvil.toml",
        &format!("{CONFIG}{locale_config}"),
    );
    for suffix in &suffixes {
        write(
            root,
            &format!("site/docs/index{suffix}.md"),
            "# Master home\n\n:::include{file=\"/fragment.md\"}\n\n![Diagram](diagram.svg)\n\n[[new-page]]\n",
        );
        write(
            root,
            &format!("site/docs/new-page{suffix}.md"),
            "# New page\n",
        );
    }
    write(root, "site/fragment.md", "Master fragment\n");
    write(root, "example.rs", "// Master example\n");
    write(root, "site/assets/diagram.svg", "master image");
    write(
        root,
        "site/nav.toml",
        "[[nav]]\npage = 'index'\nlabel = 'Master navigation'\n[[nav]]\npage = 'new-page'\n",
    );
    repo
}

#[test]
fn builds_tag_and_dirty_working_tree_with_independent_sources() {
    for locales in [false, true] {
        let repo = fixture(locales);
        let site = repo.path().join("site");
        let out = site.join("dist");
        let before = git(
            repo.path(),
            &["status", "--porcelain", "--", ":(exclude)site/dist"],
        );
        docanvil::cli::build::run(&site, Some(&out), false, true, true, false).unwrap();
        let locale = if locales { "en/" } else { "" };
        let release = fs::read_to_string(out.join(format!("release/{locale}index.html"))).unwrap();
        let master = fs::read_to_string(out.join(format!("master/{locale}index.html"))).unwrap();
        assert!(release.contains("Released home"));
        assert!(release.contains("Released fragment"));
        assert!(release.contains("Released example"));
        assert!(release.contains("Release navigation"));
        assert!(!release.contains("Master fragment"));
        assert!(!release.contains("Master navigation"));
        assert!(release.contains("/docs/release/assets/diagram.svg"));
        assert!(release.contains(&format!("/docs/release/{locale}old-page.html")));
        assert!(release.contains("2020-01-02"));
        let commit = git(repo.path(), &["rev-parse", "HEAD"]);
        assert!(
            release.contains(&format!("/edit/{commit}/site/pages/index")),
            "Edit link HTML: {}",
            release
                .lines()
                .filter(|line| line.contains("edit") || line.contains(&commit))
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert!(master.contains("Master home"));
        assert!(master.contains("documentation for unreleased changes"));
        assert!(!release.contains("documentation for unreleased changes"));
        let alias = fs::read_to_string(out.join(format!("latest/{locale}old-page.html"))).unwrap();
        assert!(alias.contains(&format!("/docs/release/{locale}old-page.html")));
        assert!(!out.join(format!("latest/{locale}new-page.html")).exists());
        if locales {
            assert!(out.join("latest/index.html").is_file());
            assert!(master.contains("/docs/master/fr/index.html"));
        }
        assert!(master.contains("Master fragment"));
        assert!(master.contains("Master navigation"));
        assert!(master.contains("/docs/master/assets/diagram.svg"));
        assert!(master.contains(&format!("/docs/master/{locale}new-page.html")));
        assert!(master.contains("/edit/master/site/docs/index"));
        assert_eq!(
            fs::read_to_string(out.join("release/assets/diagram.svg")).unwrap(),
            "released image"
        );
        assert_eq!(
            fs::read_to_string(out.join("master/assets/diagram.svg")).unwrap(),
            "master image"
        );
        let search =
            fs::read_to_string(out.join(format!("release/{locale}search-index.json"))).unwrap();
        assert!(search.contains("Released fragment"));
        assert!(!search.contains("Master fragment"));
        let llms = fs::read_to_string(out.join(format!("release/{locale}llms-full.txt"))).unwrap();
        assert!(llms.contains("https://example.com/docs/release/assets/diagram.svg"));
        assert!(
            fs::read_to_string(out.join("index.html"))
                .unwrap()
                .contains(&format!("/docs/release/{locale}index.html"))
        );
        assert_eq!(
            git(
                repo.path(),
                &["status", "--porcelain", "--", ":(exclude)site/dist"]
            ),
            before
        );
        let config = docanvil::config::Config::load(&site).unwrap();
        assert!(docanvil::doctor::checks::version::check_version(&site, &config).is_empty());
    }
}

#[test]
fn missing_tag_preserves_output_and_has_no_directory_fix() {
    let repo = fixture(false);
    let site = repo.path().join("site");
    write(
        repo.path(),
        "site/docanvil.toml",
        &CONFIG.replace("refs/tags/v1.0.0", "refs/tags/missing"),
    );
    write(repo.path(), "site/dist/index.html", "previous site");
    let error =
        docanvil::cli::build::run(&site, Some(&site.join("dist")), false, true, false, false)
            .unwrap_err();
    assert!(error.to_string().contains("refs/tags/missing"));
    assert_eq!(
        fs::read_to_string(site.join("dist/index.html")).unwrap(),
        "previous site"
    );
    let config = docanvil::config::Config::load(&site).unwrap();
    let diagnostics = docanvil::doctor::checks::version::check_version(&site, &config);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].check, "version-source-invalid");
    assert!(diagnostics[0].fix.is_none());
}

#[test]
fn serve_does_not_watch_or_open_snapshot_files() {
    let repo = fixture(false);
    let site = repo.path().join("site");
    let out = repo.path().join("preview");
    let (editor, _) = docanvil::edit::Editor::resolve(Some("vscode"));
    let dependencies =
        docanvil::cli::build::run_with_options(&site, &out, true, editor.as_ref()).unwrap();
    assert!(
        dependencies
            .iter()
            .all(|path| path.starts_with(repo.path().canonicalize().unwrap()))
    );
    assert!(
        dependencies
            .iter()
            .any(|path| path.ends_with("fragment.md"))
    );
    let release = fs::read_to_string(out.join("release/index.html")).unwrap();
    assert!(release.contains("/release/assets/diagram.svg"));
    assert!(!release.contains("vscode://"));
    let master = fs::read_to_string(out.join("master/index.html")).unwrap();
    assert!(master.contains("vscode://"));
}

#[test]
fn current_selection_moves_aliases_and_preserves_old_locale_urls() {
    let repo = fixture(true);
    let site = repo.path().join("site");
    let config = fs::read_to_string(site.join("docanvil.toml"))
        .unwrap()
        .replace("current = \"release\"", "current = \"master\"");
    write(
        repo.path(),
        "site/docanvil.toml",
        &format!("{config}\n[redirects]\nunprefixed = true\n"),
    );
    let out = site.join("dist");
    docanvil::cli::build::run(&site, Some(&out), false, true, true, false).unwrap();
    for locale in ["en", "fr"] {
        for path in [
            format!("latest/{locale}/new-page.html"),
            format!("{locale}/new-page.html"),
        ] {
            assert!(
                fs::read_to_string(out.join(path))
                    .unwrap()
                    .contains(&format!("/docs/master/{locale}/new-page.html"))
            );
        }
        assert!(!out.join(format!("latest/{locale}/old-page.html")).exists());
    }
    assert!(
        fs::read_to_string(out.join("index.html"))
            .unwrap()
            .contains("/docs/master/en/index.html")
    );
}

#[test]
fn latest_alias_conflicts_are_reported_by_build_and_doctor() {
    let repo = fixture(false);
    write(
        repo.path(),
        "site/static/latest/old-page.html",
        "existing static file",
    );
    let site = repo.path().join("site");
    let error =
        docanvil::cli::build::run(&site, Some(&site.join("dist")), false, true, true, false)
            .unwrap_err();
    assert!(matches!(error, docanvil::error::Error::StrictWarnings(_)));
    let config = docanvil::config::Config::load(&site).unwrap();
    let diagnostics = docanvil::doctor::checks::redirects::check_redirects(&site, &config);
    assert!(diagnostics.iter().any(|d| d.check == "redirect-shadowed"));
}

#[test]
fn historical_dates_exclude_commits_after_the_tag() {
    let repo = fixture(false);
    git(repo.path(), &["add", "."]);
    let output = Command::new("git")
        .current_dir(repo.path())
        .env("GIT_AUTHOR_DATE", "2024-05-06T12:00:00Z")
        .env("GIT_COMMITTER_DATE", "2024-05-06T12:00:00Z")
        .args(["commit", "-m", "Update docs"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let site = repo.path().join("site");
    let out = site.join("dist");
    docanvil::cli::build::run(&site, Some(&out), false, true, true, false).unwrap();
    let release = fs::read_to_string(out.join("release/index.html")).unwrap();
    let master = fs::read_to_string(out.join("master/index.html")).unwrap();
    assert!(release.contains("2020-01-02"));
    assert!(!release.contains("2024-05-06"));
    assert!(master.contains("2024-05-06"));
}

#[test]
fn tag_only_site_does_not_require_working_content_directory() {
    let repo = fixture(false);
    let site = repo.path().join("site");
    let config = CONFIG
        .replace(
            "enabled = [\"release\", \"master\"]",
            "enabled = [\"release\"]",
        )
        .replace("working_tree = \"master\"\n", "");
    write(repo.path(), "site/docanvil.toml", &config);
    fs::remove_dir_all(site.join("docs")).unwrap();
    let out = site.join("dist");
    docanvil::cli::build::run(&site, Some(&out), false, true, true, false).unwrap();
    assert!(out.join("release/index.html").is_file());
    assert!(out.join("latest/index.html").is_file());
    assert!(docanvil::doctor::checks::project::check_project(&site).is_empty());
}

#[test]
fn tag_patterns_build_every_matching_release_and_optional_prereleases() {
    let repo = fixture(false);
    for tag in ["v1.9.0", "v1.10.0", "v1.11.0-beta.1", "v2.0.0"] {
        git(repo.path(), &["tag", tag]);
    }
    let site = repo.path().join("site");
    let config = CONFIG
        .replace("current = \"release\"", "current = \"v1.10.0\"")
        .replace(
            "enabled = [\"release\", \"master\"]",
            "enabled = [\"v1.0.0\", \"master\"]",
        )
        .replace(
            "refs = { release = \"refs/tags/v1.0.0\" }",
            "tag_patterns = [\"v1.*\", \"v1.9.*\"]",
        );
    write(repo.path(), "site/docanvil.toml", &config);
    let resolved = docanvil::config::Config::load(&site).unwrap();
    assert_eq!(
        resolved.version.enabled,
        ["v1.0.0", "v1.9.0", "v1.10.0", "master"]
    );
    let out = site.join("dist");
    docanvil::cli::build::run(&site, Some(&out), false, true, true, false).unwrap();
    for version in ["v1.0.0", "v1.9.0", "v1.10.0"] {
        let page = fs::read_to_string(out.join(format!("{version}/index.html"))).unwrap();
        assert!(page.contains("Released fragment"));
        assert!(out.join(format!("{version}/search-index.json")).is_file());
    }
    assert!(!out.join("v2.0.0").exists());
    assert!(!out.join("v1.11.0-beta.1").exists());
    assert!(
        fs::read_to_string(out.join("latest/index.html"))
            .unwrap()
            .contains("/docs/v1.10.0/index.html")
    );
    assert!(docanvil::doctor::checks::version::check_version(&site, &resolved).is_empty());
    let config = config.replace(
        "working_tree = \"master\"",
        "working_tree = \"master\"\ninclude_prereleases = true",
    );
    write(repo.path(), "site/docanvil.toml", &config);
    docanvil::cli::build::run(&site, Some(&out), false, true, true, false).unwrap();
    assert!(out.join("v1.11.0-beta.1/index.html").is_file());
    // Discovering a newer prerelease never changes the explicitly chosen current.
    assert!(
        fs::read_to_string(out.join("latest/index.html"))
            .unwrap()
            .contains("/docs/v1.10.0/index.html")
    );
}

#[test]
fn unmatched_tag_pattern_fails_before_replacing_output() {
    let repo = fixture(false);
    let site = repo.path().join("site");
    let config = CONFIG
        .replace(
            "enabled = [\"release\", \"master\"]",
            "enabled = [\"master\"]",
        )
        .replace(
            "refs = { release = \"refs/tags/v1.0.0\" }",
            "tag_patterns = [\"v99.*\"]",
        );
    write(repo.path(), "site/docanvil.toml", &config);
    write(repo.path(), "site/dist/index.html", "previous site");
    let error =
        docanvil::cli::build::run(&site, Some(&site.join("dist")), false, true, true, false)
            .unwrap_err();
    assert!(error.to_string().contains("v99.*"));
    assert!(error.to_string().contains("no eligible local tags"));
    assert_eq!(
        fs::read_to_string(site.join("dist/index.html")).unwrap(),
        "previous site"
    );
}
