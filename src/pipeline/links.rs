//! Link checks for generated API targets, shared by publishing and doctor.
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::pipeline::headings::{anchor_ids, decode_entities};
use crate::project::{PageInfo, PageInventory};

pub(crate) fn split_target(target: &str) -> (&str, Option<&str>) {
    match target.split_once('#') {
        Some((page, fragment)) => (page, Some(fragment)),
        None => (target, None),
    }
}

/// URI fragments are percent-decoded once; `+` remains a literal plus.
pub(crate) fn decode_fragment(fragment: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut input = fragment.bytes();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let high = (input.next()? as char).to_digit(16)?;
            let low = (input.next()? as char).to_digit(16)?;
            bytes.push((high * 16 + low) as u8);
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8(bytes).ok()
}

/// Authored-page fragments are outside this check's declared support boundary.
pub(crate) fn fragment_exists(
    inventory: &PageInventory,
    page: &PageInfo,
    fragment: Option<&str>,
) -> bool {
    let Some(fragment) = fragment.filter(|fragment| !fragment.is_empty()) else {
        return true;
    };
    let Some(generated) = inventory.generated_source(page) else {
        return true;
    };
    decode_fragment(fragment).is_some_and(|id| anchor_ids(&generated.html).contains(&id))
}

/// Inspect rendered anchors, so ordinary/reference-style Markdown links and
/// links in API descriptions use the same URL rules. External URLs are ignored.
pub(crate) fn generated_link_problems(
    html: &str,
    inventory: &PageInventory,
    source: &PageInfo,
    base_url: &str,
) -> Vec<String> {
    if inventory.generated.is_empty() {
        return Vec::new();
    }
    static LINKS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"<a\b[^>]*\bhref="([^"]*)""#).unwrap());
    LINKS
        .captures_iter(html)
        .filter_map(|capture| {
            let href = decode_entities(&capture[1]);
            let (path, fragment) = split_target(&href);
            let output = local_output_path(path, source, base_url)?;
            let page = inventory
                .pages
                .values()
                .find(|page| page.output_path == Path::new(&output));
            if let Some(page) = page {
                return (!fragment_exists(inventory, page, fragment))
                    .then(|| format!("Anchor not found in generated API page: {href}"));
            }
            // Missing routes are checked only inside a generated API namespace for
            // this version and locale, leaving other documentation/assets alone.
            let generated_namespace = inventory.pages.values().any(|page| {
                inventory.generated_source(page).is_some()
                    && page.slug.ends_with("/index")
                    && output.starts_with(&format!(
                        "{}/",
                        page.output_path
                            .parent()
                            .unwrap()
                            .display()
                            .to_string()
                            .replace('\\', "/")
                    ))
            });
            generated_namespace.then(|| format!("Generated API page not found: {href}"))
        })
        .collect()
}

/// Resolve browser-relative local URLs against the published page, not its spec file.
pub(crate) fn local_output_path(path: &str, source: &PageInfo, base_url: &str) -> Option<String> {
    let path = path.split('?').next().unwrap();
    if path.starts_with("//") || path.contains(':') {
        return None;
    }
    let output = if path.is_empty() {
        source.output_path.to_string_lossy().into_owned()
    } else if path.starts_with('/') {
        path.strip_prefix(base_url)?.to_owned()
    } else {
        let parent = source.output_path.parent().unwrap_or_else(|| Path::new(""));
        format!(
            "{}{path}",
            if parent.as_os_str().is_empty() {
                String::new()
            } else {
                format!("{}/", parent.display())
            }
        )
    };
    let output = output.replace('\\', "/");
    let mut normalized = Vec::new();
    for part in output.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                normalized.pop()?;
            }
            part => normalized.push(part),
        }
    }
    let mut output = normalized.join("/");
    if path.ends_with('/') {
        if !output.is_empty() {
            output.push('/');
        }
        output.push_str("index.html");
    }
    Some(output)
}
