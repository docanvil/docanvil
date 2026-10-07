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
        .map(|file| normalize_line_endings(&String::from_utf8_lossy(&file.data)))
}

/// Convert CRLF line endings to LF. Templates checked out on Windows (and the
/// release binary built there embeds them as-is) or edited in a Windows editor
/// would otherwise leave stray `\r`s in component output.
pub fn normalize_line_endings(source: &str) -> String {
    source.replace("\r\n", "\n")
}
