//! Build-time Lucide icons shared by built-in and custom component templates.

use std::collections::HashMap;
use std::sync::LazyLock;

pub const LICENSE: &str = include_str!("lucide/LICENSE.txt");

static ICONS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("lucide/icons.json"))
        .expect("bundled Lucide catalogue is valid JSON")
});

/// Render trusted bundled SVGs; escape legacy text/emoji before marking it safe.
/// An explicit `lucide:` prefix catches misspelled icon names at build time.
pub fn render(name: &str) -> tera::Result<String> {
    let key = name.strip_prefix("lucide:").unwrap_or(name);
    if let Some(svg) = ICONS.get(key) {
        return Ok(svg
            .replacen("<svg", "<svg aria-hidden=\"true\" focusable=\"false\"", 1)
            .replacen("class=\"lucide", "class=\"docanvil-icon lucide", 1));
    }
    if name.starts_with("lucide:") {
        return Err(tera::Error::msg(format!("Unknown Lucide icon: {key}")));
    }
    Ok(crate::util::html_escape(name).replace('\'', "&#39;"))
}

/// `{{ icon(name="lucide:zap") | safe }}` in any component template.
pub fn template_icon(args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
    let name = args
        .get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| tera::Error::msg("icon() requires a string name"))?;
    render(name).map(serde_json::Value::String)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_catalogue_renders_accessible_current_colour_svgs() {
        assert!(ICONS.len() > 2000);
        for name in ICONS.keys() {
            let svg = render(name).unwrap();
            assert!(svg.contains("class=\"docanvil-icon lucide"), "{name}");
            assert!(svg.contains("aria-hidden=\"true\""), "{name}");
            assert!(svg.contains("stroke=\"currentColor\""), "{name}");
        }
        assert_eq!(render("zap").unwrap(), render("lucide:zap").unwrap());
    }

    #[test]
    fn legacy_text_is_escaped_and_explicit_unknown_names_fail() {
        assert_eq!(render("⚡").unwrap(), "⚡");
        assert_eq!(
            render("<script>alert('x')</script>").unwrap(),
            "&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;"
        );
        assert!(render("lucide:not-a-real-icon").is_err());
        assert!(render("lucide:../../LICENSE").is_err());
        assert!(template_icon(&HashMap::new()).is_err());
    }
}
