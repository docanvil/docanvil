//! Language-independent region markers. Comment syntax belongs to the author.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use regex::Regex;

static WRAPPER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{==.*?==\}").unwrap());
static MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\{==\s*(region|endregion)\s*=\s*"([^"\r\n]+)"\s*==\}$"#).unwrap()
});

/// Return original 1-based line ranges, excluding all markers within the region.
pub(super) fn select(lines: &[&str], wanted: &str) -> Result<Vec<(usize, usize)>, String> {
    if wanted.trim().is_empty() {
        return Err("region name cannot be empty".into());
    }
    let mut stack: Vec<(String, usize)> = Vec::new();
    let mut names = BTreeSet::new();
    let mut regions = BTreeMap::new();
    let mut markers = BTreeSet::new();
    for (index, line) in lines.iter().enumerate() {
        let line_number = index + 1;
        let wrappers: Vec<_> = WRAPPER.find_iter(line).collect();
        if wrappers.is_empty() {
            continue;
        }
        if wrappers.len() != 1 {
            return Err(format!("multiple region markers on line {line_number}"));
        }
        let caps = MARKER.captures(wrappers[0].as_str()).ok_or_else(|| {
            format!("invalid region marker on line {line_number}; use {{== region=\"name\" ==}} or {{== endregion=\"name\" ==}}")
        })?;
        let name = &caps[2];
        if name.trim().is_empty() {
            return Err(format!("empty region name on line {line_number}"));
        }
        markers.insert(line_number);
        if &caps[1] == "region" {
            if !names.insert(name.to_string()) {
                return Err(format!("duplicate region \"{name}\" on line {line_number}"));
            }
            stack.push((name.to_string(), line_number));
        } else {
            let Some((open, start)) = stack.pop() else {
                return Err(format!(
                    "endregion \"{name}\" on line {line_number} has no opening marker"
                ));
            };
            if open != name {
                return Err(format!(
                    "endregion \"{name}\" on line {line_number} closes region \"{open}\""
                ));
            }
            regions.insert(open, (start, line_number));
        }
    }
    if let Some((name, line)) = stack.last() {
        return Err(format!(
            "region \"{name}\" on line {line} has no closing marker"
        ));
    }
    let &(start, end) = regions
        .get(wanted)
        .ok_or_else(|| format!("can't find region \"{wanted}\""))?;
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for n in start + 1..end {
        if markers.contains(&n) {
            continue;
        }
        if let Some(last) = ranges.last_mut()
            && last.1 + 1 == n
        {
            last.1 = n;
        } else {
            ranges.push((n, n));
        }
    }
    if ranges.is_empty() {
        return Err(format!("region \"{wanted}\" contains no source lines"));
    }
    Ok(ranges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_any_comment_wrapper_and_removes_nested_markers() {
        let lines = [
            "// {== region=\"outer\" ==}",
            "one",
            "# {== region=\"inner\" ==}",
            "two",
            "<!-- {== endregion=\"inner\" ==} -->",
            "three",
            "/* {== endregion=\"outer\" ==} */",
        ];
        assert_eq!(
            select(&lines, "outer").unwrap(),
            vec![(2, 2), (4, 4), (6, 6)]
        );
        assert_eq!(select(&lines, "inner").unwrap(), vec![(4, 4)]);
    }

    #[test]
    fn diagnoses_invalid_boundaries() {
        for (lines, message) in [
            (vec!["{== region=\"x\" ==}"], "no closing"),
            (vec!["{== endregion=\"x\" ==}"], "no opening"),
            (
                vec!["{== region=\"x\" ==}", "{== endregion=\"y\" ==}"],
                "closes region",
            ),
            (
                vec!["{== region=\"x\" ==}", "{== region=\"x\" ==}"],
                "duplicate",
            ),
            (
                vec!["{== region=\"x\" ==}", "{== endregion=\"x\" ==}"],
                "no source lines",
            ),
            (vec!["{== region=x ==}"], "invalid"),
            (
                vec!["{== region=\"x\" ==} {== endregion=\"x\" ==}"],
                "multiple",
            ),
            (vec!["ordinary source"], "can't find"),
        ] {
            assert!(select(&lines, "x").unwrap_err().contains(message));
        }
        assert!(select(&[], "").unwrap_err().contains("empty"));
    }
}
