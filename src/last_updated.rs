//! "Last updated" dates for pages: from Git history, overridable in front matter.

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

/// A calendar date (UTC), shown as `YYYY-MM-DD`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Date {
    /// The UTC calendar date of a Unix timestamp (Howard Hinnant's civil-from-days).
    pub fn from_unix(secs: i64) -> Self {
        let days = secs.div_euclid(86_400);
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
        Self { year, month, day }
    }

    /// Parse a strict `YYYY-MM-DD` date, rejecting days that don't exist.
    pub fn parse(s: &str) -> Option<Self> {
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let digits = |range: std::ops::Range<usize>| -> Option<u32> {
            let part = &s[range];
            if !part.bytes().all(|c| c.is_ascii_digit()) {
                return None;
            }
            part.parse().ok()
        };
        let year = digits(0..4)? as i32;
        let month = digits(5..7)? as u8;
        let day = digits(8..10)? as u8;
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// A page's front matter `last_updated` value.
#[derive(Debug, PartialEq, Eq)]
pub enum Override {
    /// `"YYYY-MM-DD"`: use this date instead of the computed one.
    Date(Date),
    /// `false`: show no date on this page.
    Hide,
    /// Anything else. Ignored by the build; reported by `docanvil doctor`.
    Invalid,
}

/// Interpret a front matter `last_updated` value.
pub fn parse_override(value: &serde_json::Value) -> Override {
    match value {
        serde_json::Value::Bool(false) => Override::Hide,
        serde_json::Value::String(s) => Date::parse(s).map_or(Override::Invalid, Override::Date),
        _ => Override::Invalid,
    }
}

/// Where computed dates come from. Front matter overrides are applied by the caller,
/// so a source only answers for files.
pub trait DateSource {
    fn date_for(&mut self, path: &Path) -> Option<Date>;
}

/// The source for `source = "front-matter"`: never has a date of its own.
pub struct NoDates;

impl DateSource for NoDates {
    fn date_for(&mut self, _path: &Path) -> Option<Date> {
        None
    }
}

/// The newest date across a page's source file and the files it pulls in.
pub fn page_date(
    source: &mut dyn DateSource,
    page: &Path,
    deps: &BTreeSet<PathBuf>,
) -> Option<Date> {
    std::iter::once(page)
        .chain(deps.iter().map(PathBuf::as_path))
        .filter_map(|path| source.date_for(path))
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn d(s: &str) -> Date {
        Date::parse(s).unwrap()
    }

    #[test]
    fn from_unix_known_dates() {
        assert_eq!(Date::from_unix(0).to_string(), "1970-01-01");
        assert_eq!(Date::from_unix(-86_400).to_string(), "1969-12-31");
        assert_eq!(Date::from_unix(951_782_400).to_string(), "2000-02-29");
        assert_eq!(Date::from_unix(1_791_590_400).to_string(), "2026-10-10");
        // Late in the day stays on the same UTC date
        assert_eq!(
            Date::from_unix(1_791_590_400 + 86_399).to_string(),
            "2026-10-10"
        );
        assert_eq!(Date::from_unix(1_767_225_599).to_string(), "2025-12-31");
        assert_eq!(Date::from_unix(1_767_225_600).to_string(), "2026-01-01");
    }

    #[test]
    fn parse_accepts_valid_dates() {
        assert_eq!(d("2026-10-09").to_string(), "2026-10-09");
        assert_eq!(d("2024-02-29").to_string(), "2024-02-29");
        assert_eq!(d("2000-02-29").to_string(), "2000-02-29");
    }

    #[test]
    fn parse_rejects_invalid_dates() {
        for bad in [
            "2026-02-30",
            "2025-02-29",
            "1900-02-29",
            "2026-13-01",
            "2026-00-10",
            "2026-04-31",
            "2026-01-00",
            "26-1-1",
            "2026-1-01",
            "2026-01-01x",
            "2026/01/01",
            "",
            "abcd-ef-gh",
            " 2026-01-01",
        ] {
            assert!(Date::parse(bad).is_none(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn dates_order_chronologically() {
        assert!(d("2026-01-31") < d("2026-02-01"));
        assert!(d("2025-12-31") < d("2026-01-01"));
        assert_eq!(d("2026-03-04").max(d("2026-03-05")), d("2026-03-05"));
    }

    #[test]
    fn override_values() {
        assert_eq!(
            parse_override(&json!("2026-09-30")),
            Override::Date(d("2026-09-30"))
        );
        assert_eq!(parse_override(&json!(false)), Override::Hide);
        assert_eq!(parse_override(&json!(true)), Override::Invalid);
        assert_eq!(parse_override(&json!("2026-9-30")), Override::Invalid);
        assert_eq!(parse_override(&json!(20260930)), Override::Invalid);
        assert_eq!(parse_override(&json!(null)), Override::Invalid);
    }

    struct Fixed(HashMap<PathBuf, Date>);

    impl DateSource for Fixed {
        fn date_for(&mut self, path: &Path) -> Option<Date> {
            self.0.get(path).copied()
        }
    }

    #[test]
    fn page_date_takes_newest_of_page_and_deps() {
        let mut source = Fixed(HashMap::from([
            (PathBuf::from("/p/page.md"), d("2026-01-01")),
            (PathBuf::from("/p/_frag.md"), d("2026-03-01")),
            (PathBuf::from("/p/old.rs"), d("2025-06-01")),
        ]));
        let deps = BTreeSet::from([PathBuf::from("/p/_frag.md"), PathBuf::from("/p/old.rs")]);
        assert_eq!(
            page_date(&mut source, Path::new("/p/page.md"), &deps),
            Some(d("2026-03-01"))
        );
    }

    #[test]
    fn page_date_uses_deps_when_page_has_no_date() {
        let mut source = Fixed(HashMap::from([(
            PathBuf::from("/p/_frag.md"),
            d("2026-03-01"),
        )]));
        let deps = BTreeSet::from([PathBuf::from("/p/_frag.md")]);
        assert_eq!(
            page_date(&mut source, Path::new("/p/untracked.md"), &deps),
            Some(d("2026-03-01"))
        );
    }

    #[test]
    fn page_date_none_when_nothing_has_a_date() {
        let mut source = NoDates;
        assert_eq!(
            page_date(&mut source, Path::new("/p/page.md"), &BTreeSet::new()),
            None
        );
    }
}
