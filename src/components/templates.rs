//! Embedded default templates for the builtin components.

use rust_embed::Embed;

#[derive(Embed)]
#[folder = "src/theme/default/components/"]
struct BuiltinTemplates;

/// Names of all builtin components (template file stems), sorted.
pub fn builtin_names() -> Vec<String> {
    let mut names: Vec<String> = BuiltinTemplates::iter()
        .filter_map(|file| file.strip_suffix(".html").map(str::to_string))
        .collect();
    names.sort();
    names
}

/// The embedded template source for a builtin component.
pub fn builtin_source(name: &str) -> Option<String> {
    BuiltinTemplates::get(&format!("{name}.html"))
        .map(|file| String::from_utf8_lossy(&file.data).into_owned())
}
