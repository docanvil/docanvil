//! Includes: `:::include{file="…"}` and code blocks filled from source files.

/// Parse a `lines="…"` spec for a file with `line_count` lines.
///
/// Grammar: `N`, `N-M`, `N-` (to the end), `-M` (from the start), joined by
/// commas; 1-based and inclusive. Ranges must go in order without overlapping
/// (adjacent is fine). A range past the end of the file is an error, so stale
/// line numbers are caught at build time rather than shipped.
pub fn parse_line_ranges(spec: &str, line_count: usize) -> Result<Vec<(usize, usize)>, String> {
    if spec.trim().is_empty() {
        return Err(
            "lines=\"\" is empty — give a line or range such as lines=\"5-12\", or leave lines out to show the whole file"
                .to_string(),
        );
    }

    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        let (start, end) = match part.split_once('-') {
            Some((a, b)) => {
                let (a, b) = (a.trim(), b.trim());
                if a.is_empty() && b.is_empty() {
                    return Err(not_a_range(part));
                }
                let start = if a.is_empty() {
                    1
                } else {
                    line_number(a, part)?
                };
                let end = if b.is_empty() {
                    None
                } else {
                    Some(line_number(b, part)?)
                };
                (start, end)
            }
            None => {
                let n = line_number(part, part)?;
                (n, Some(n))
            }
        };

        if let Some(end) = end
            && start > end
        {
            return Err(format!("lines range \"{part}\" ends before it starts"));
        }
        if start > line_count {
            return Err(format!(
                "lines range \"{part}\" starts past the end of the file, which has {}",
                count_lines(line_count)
            ));
        }
        let end = match end {
            Some(end) if end > line_count => {
                return Err(format!(
                    "lines range \"{part}\" runs past the end of the file, which has {}",
                    count_lines(line_count)
                ));
            }
            Some(end) => end,
            None => line_count,
        };

        if let Some(&(prev_start, prev_end)) = ranges.last() {
            if start < prev_start {
                return Err(format!(
                    "lines ranges must go in order — \"{part}\" comes before the range ahead of it"
                ));
            }
            if start <= prev_end {
                return Err(format!(
                    "lines range \"{part}\" overlaps the range before it"
                ));
            }
        }
        ranges.push((start, end));
    }
    Ok(ranges)
}

fn line_number(text: &str, part: &str) -> Result<usize, String> {
    match text.parse::<usize>() {
        Ok(0) => Err(format!(
            "lines range \"{part}\" uses line 0 — lines are numbered from 1"
        )),
        Ok(n) => Ok(n),
        Err(_) => Err(not_a_range(part)),
    }
}

fn not_a_range(part: &str) -> String {
    format!(
        "\"{part}\" isn't a line or range — use e.g. lines=\"7\", \"5-12\", \"5-\", \"-12\" or \"1-6,30-35\""
    )
}

fn count_lines(n: usize) -> String {
    if n == 1 {
        "1 line".to_string()
    } else {
        format!("{n} lines")
    }
}

/// The lines covered by `ranges` (as returned by [`parse_line_ranges`]), in order.
pub fn select_lines<'a>(lines: &[&'a str], ranges: &[(usize, usize)]) -> Vec<&'a str> {
    ranges
        .iter()
        .flat_map(|&(start, end)| lines[start - 1..end].iter().copied())
        .collect()
}

/// Remove the leading whitespace shared by every non-blank line, keeping
/// relative indentation. Tabs and spaces are compared literally (a tab never
/// matches spaces). Whitespace-only lines become empty.
pub fn dedent(lines: &[&str]) -> Vec<String> {
    let common = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| &line[..line.len() - line.trim_start().len()])
        .reduce(common_prefix)
        .unwrap_or("");
    lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                line[common.len()..].to_string()
            }
        })
        .collect()
}

fn common_prefix<'a>(a: &'a str, b: &str) -> &'a str {
    let len: usize = a
        .chars()
        .zip(b.chars())
        .take_while(|(x, y)| x == y)
        .map(|(x, _)| x.len_utf8())
        .sum();
    &a[..len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_line_and_closed_range() {
        assert_eq!(parse_line_ranges("7", 10), Ok(vec![(7, 7)]));
        assert_eq!(parse_line_ranges("5-12", 20), Ok(vec![(5, 12)]));
    }

    #[test]
    fn open_ended_ranges() {
        assert_eq!(parse_line_ranges("5-", 10), Ok(vec![(5, 10)]));
        assert_eq!(parse_line_ranges("-3", 10), Ok(vec![(1, 3)]));
        assert_eq!(parse_line_ranges("40-", 40), Ok(vec![(40, 40)]));
    }

    #[test]
    fn lists_whitespace_and_adjacent_ranges() {
        assert_eq!(
            parse_line_ranges(" 1 - 6 , 30-35 ", 40),
            Ok(vec![(1, 6), (30, 35)])
        );
        assert_eq!(parse_line_ranges("1-6,7-9", 40), Ok(vec![(1, 6), (7, 9)]));
    }

    #[test]
    fn rejects_bad_specs() {
        for spec in [
            "", " ", "0", "0-3", "7-5", "abc", "1-x", "-", "1--3", "1-6,",
        ] {
            assert!(
                parse_line_ranges(spec, 40).is_err(),
                "{spec:?} should be rejected"
            );
        }
    }

    #[test]
    fn rejects_descending_and_overlapping() {
        assert!(
            parse_line_ranges("30-35,1-6", 40)
                .unwrap_err()
                .contains("in order")
        );
        assert!(
            parse_line_ranges("1-6,5-9", 40)
                .unwrap_err()
                .contains("overlaps")
        );
        assert!(
            parse_line_ranges("1-6,6-9", 40)
                .unwrap_err()
                .contains("overlaps")
        );
    }

    #[test]
    fn rejects_ranges_past_the_end() {
        let err = parse_line_ranges("41", 40).unwrap_err();
        assert!(
            err.contains("past the end") && err.contains("40 lines"),
            "{err}"
        );
        assert!(
            parse_line_ranges("38-45", 40)
                .unwrap_err()
                .contains("past the end")
        );
        assert!(parse_line_ranges("41-", 40).is_err());
        assert!(parse_line_ranges("-41", 40).is_err());
        assert!(parse_line_ranges("1", 0).is_err());
    }

    #[test]
    fn selects_ranges_in_order() {
        let lines = ["a", "b", "c", "d", "e"];
        assert_eq!(select_lines(&lines, &[(1, 2), (4, 4)]), vec!["a", "b", "d"]);
    }

    #[test]
    fn dedent_removes_common_indent() {
        assert_eq!(
            dedent(&["    fn a() {", "        x", "    }"]),
            vec!["fn a() {", "    x", "}"]
        );
    }

    #[test]
    fn dedent_ignores_blank_lines() {
        assert_eq!(
            dedent(&["    a", "", "  ", "    b"]),
            vec!["a", "", "", "b"]
        );
    }

    #[test]
    fn dedent_keeps_relative_indent_across_ranges() {
        assert_eq!(
            dedent(&["        inner", "    outer"]),
            vec!["    inner", "outer"]
        );
    }

    #[test]
    fn dedent_compares_tabs_and_spaces_literally() {
        assert_eq!(dedent(&["\tx", "    y"]), vec!["\tx", "    y"]);
        assert_eq!(dedent(&["\t\tx", "\ty"]), vec!["\tx", "y"]);
        assert_eq!(dedent(&["a", " b"]), vec!["a", " b"]);
    }
}
