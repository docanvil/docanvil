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
pub fn render(name: &str) -> tera::TeraResult<String> {
    let key = name.strip_prefix("lucide:").unwrap_or(name);
    if let Some(svg) = ICONS.get(key) {
        return Ok(svg
            .replacen("<svg", "<svg aria-hidden=\"true\" focusable=\"false\"", 1)
            .replacen("class=\"lucide", "class=\"docanvil-icon lucide", 1));
    }
    if name.starts_with("lucide:") {
        return Err(tera::Error::message(format!("Unknown Lucide icon: {key}")));
    }
    Ok(crate::util::html_escape(name).replace('\'', "&#39;"))
}

/// `{{ icon(name="lucide:zap") }}` returns trusted SVG or escaped legacy text.
pub fn template_icon(args: tera::Kwargs, _: &tera::State) -> tera::TeraResult<tera::Value> {
    let html = render(args.must_get::<&str>("name")?)?;
    Ok(tera::Value::safe_string(&html))
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
    fn template_icons_are_safe_but_legacy_text_is_escaped_once() {
        let mut tera = tera::Tera::default();
        tera.register_function("icon", template_icon);
        tera.add_raw_template("test.html", "{{ icon(name=name) }}")
            .unwrap();
        let mut context = tera::Context::new();
        for name in [
            "lucide:zap",
            "⚡",
            "<script>alert('x')</script>",
            "A & B / C",
        ] {
            context.insert("name", name);
            assert_eq!(
                tera.render("test.html", &context).unwrap(),
                render(name).unwrap()
            );
        }
        // Existing custom templates with an explicit safe filter still work.
        tera.add_raw_template("legacy.html", "{{ icon(name=name) | safe }}")
            .unwrap();
        assert_eq!(
            tera.render("legacy.html", &context).unwrap(),
            "A &amp; B / C"
        );
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
        let mut tera = tera::Tera::default();
        tera.register_function("icon", template_icon);
        tera.add_raw_template("test.html", "{{ icon() }}").unwrap();
        assert!(tera.render("test.html", &tera::Context::new()).is_err());
    }
}
