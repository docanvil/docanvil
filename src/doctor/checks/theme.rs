use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::components::ComponentRegistry;
use crate::config::Config;
use crate::doctor::{Diagnostic, Fix, Severity};
use crate::error::Error;
use crate::pipeline::directives::INCLUDE_DIRECTIVE;
use crate::render::templates::{find_include_cycle, include_cycle_message};

/// Check theme: custom CSS existence, layout template validity.
pub fn check_theme(project_root: &Path, config: &Config) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Check custom CSS file
    if let Some(ref css_path) = config.theme.custom_css {
        let full_path = project_root.join(css_path);
        if !full_path.exists() {
            diags.push(Diagnostic {
                check: "custom-css-not-found",
                category: "theme",
                severity: Severity::Warning,
                message: format!("Custom CSS file not found: {css_path}"),
                file: Some(full_path.clone()),
                line: None,
                fix: Some(Fix::CreateFile {
                    path: full_path,
                    content: String::new(),
                }),
            });
        }
    }

    // Check user layout template for Tera errors
    let user_layout_path = project_root.join("theme/templates/layout.html");
    if user_layout_path.exists() {
        let template_content = match std::fs::read_to_string(&user_layout_path) {
            Ok(content) => content,
            Err(e) => {
                diags.push(Diagnostic {
                    check: "layout-read-error",
                    category: "theme",
                    severity: Severity::Error,
                    message: format!("Cannot read layout template: {e}"),
                    file: Some(user_layout_path),
                    line: None,
                    fix: None,
                });
                return diags;
            }
        };

        let mut tera = tera::Tera::default();
        let error = match tera.add_raw_template("layout.html", &template_content) {
            Err(e) => Some(e.to_string()),
            Ok(()) => {
                find_include_cycle(&tera).map(|cycle| include_cycle_message(&cycle, "templates"))
            }
        };
        if let Some(e) = error {
            diags.push(Diagnostic {
                check: "layout-tera-error",
                category: "theme",
                severity: Severity::Error,
                message: format!("Layout template has Tera errors: {e}"),
                file: Some(user_layout_path),
                line: None,
                fix: None,
            });
        }
    }

    diags.extend(check_component_templates(project_root));

    diags
}

static COMPONENT_NAME_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\w[\w-]*$").unwrap());
static BARE_BODY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{-?\s*body\s*-?\}\}").unwrap());

/// Check `theme/components/*.html`: parses, usable names, `body` printed with `| safe`.
fn check_component_templates(project_root: &Path) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let dir = ComponentRegistry::user_template_dir(project_root);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return diags;
    };
    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "html"))
        .collect();
    paths.sort();

    // Get the Tera verdict the same way the build does: all of
    // `theme/components/*.html` loaded together, so a valid cross-file
    // `{% extends %}` / `{% import %}` isn't falsely flagged, and a real error
    // matches what `docanvil build` would fail on.
    let tera_error = match ComponentRegistry::load(project_root) {
        Ok(_) => None,
        Err(Error::ComponentTemplate { path, message }) => Some((path, message)),
        Err(e) => Some((dir.clone(), e.to_string())),
    };
    if let Some((path, message)) = &tera_error {
        diags.push(Diagnostic {
            check: "component-tera-error",
            category: "theme",
            severity: Severity::Error,
            message: format!("Component template has Tera errors: {message}"),
            file: Some(path.clone()),
            line: None,
            fix: None,
        });
    }

    for path in paths {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        if !COMPONENT_NAME_RE.is_match(&stem) {
            diags.push(Diagnostic {
                check: "component-invalid-name",
                category: "theme",
                severity: Severity::Warning,
                message: format!(
                    "Component template '{stem}.html' can't be used: names may only contain letters, numbers, '_' and '-'"
                ),
                file: Some(path),
                line: None,
                fix: None,
            });
            continue;
        }

        if stem == INCLUDE_DIRECTIVE {
            diags.push(Diagnostic {
                check: "component-reserved-name",
                category: "theme",
                severity: Severity::Warning,
                message: "'include.html' is never used: :::include is reserved for including Markdown files. Rename the template to use it as a component".to_string(),
                file: Some(path),
                line: None,
                fix: None,
            });
            continue;
        }

        // Already reported above as the one template that failed to load —
        // its content can't be trusted enough to also check for `{{ body }}`.
        if tera_error
            .as_ref()
            .is_some_and(|(failed, _)| *failed == path)
        {
            continue;
        }

        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };

        if BARE_BODY_RE.is_match(&source) {
            diags.push(Diagnostic {
                check: "component-body-unescaped",
                category: "theme",
                severity: Severity::Warning,
                message: format!(
                    "'{stem}.html' prints {{{{ body }}}} without | safe, so the body will show as escaped HTML — use {{{{ body | safe }}}}"
                ),
                file: Some(path),
                line: None,
                fix: None,
            });
        }
    }
    diags
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component_project(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let comps = dir.path().join("theme/components");
        std::fs::create_dir_all(&comps).unwrap();
        for (name, content) in files {
            std::fs::write(comps.join(name), content).unwrap();
        }
        dir
    }

    fn checks_for(dir: &tempfile::TempDir) -> Vec<&'static str> {
        check_theme(dir.path(), &Config::default())
            .into_iter()
            .map(|d| d.check)
            .collect()
    }

    #[test]
    fn valid_component_templates_pass() {
        let dir = component_project(&[("card.html", "<div>{{ body | safe }}</div>")]);
        assert!(checks_for(&dir).is_empty());
    }

    #[test]
    fn component_include_cycle_is_reported() {
        let dir = component_project(&[("loop.html", "{% include \"loop.html\" %}")]);
        assert_eq!(checks_for(&dir), vec!["component-tera-error"]);
    }

    #[test]
    fn layout_include_cycle_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let templates = dir.path().join("theme/templates");
        std::fs::create_dir_all(&templates).unwrap();
        std::fs::write(
            templates.join("layout.html"),
            "{% include \"layout.html\" %}",
        )
        .unwrap();
        assert!(checks_for(&dir).contains(&"layout-tera-error"));
    }

    #[test]
    fn component_syntax_error_is_reported() {
        let dir = component_project(&[("card.html", "{% if %}")]);
        assert_eq!(checks_for(&dir), vec!["component-tera-error"]);
    }

    #[test]
    fn component_invalid_name_is_reported() {
        let dir = component_project(&[("My Card.html", "<div></div>")]);
        assert_eq!(checks_for(&dir), vec!["component-invalid-name"]);
    }

    #[test]
    fn component_body_without_safe_is_reported() {
        let dir = component_project(&[("card.html", "<div>{{ body }}</div>")]);
        assert_eq!(checks_for(&dir), vec!["component-body-unescaped"]);
        let dir = component_project(&[("card.html", "<div>{{- body -}}</div>")]);
        assert_eq!(checks_for(&dir), vec!["component-body-unescaped"]);
    }

    #[test]
    fn body_raw_and_safe_body_are_fine() {
        let dir = component_project(&[("card.html", "{{ body_raw }}{{ body | safe }}")]);
        assert!(checks_for(&dir).is_empty());
    }

    #[test]
    fn cross_file_import_with_later_sorting_macros_file_passes() {
        // "badge.html" sorts before "zz-macros.html" alphabetically; doctor must
        // agree with the build that loads every template together.
        let dir = component_project(&[
            (
                "badge.html",
                "{% import \"zz-macros.html\" as m %}{{ m::shout(text=attrs.text) }}",
            ),
            (
                "zz-macros.html",
                "{% macro shout(text) %}{{ text | upper }}{% endmacro shout %}",
            ),
        ]);
        assert!(checks_for(&dir).is_empty());
    }

    #[test]
    fn extends_a_builtin_template_passes() {
        let dir = component_project(&[("alert.html", "{% extends \"note.html\" %}")]);
        assert!(checks_for(&dir).is_empty());
    }

    #[test]
    fn include_template_is_reserved() {
        let dir = component_project(&[("include.html", "<div>{{ body | safe }}</div>")]);
        assert_eq!(checks_for(&dir), vec!["component-reserved-name"]);
    }
}
