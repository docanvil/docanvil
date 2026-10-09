//! Redirects: small HTML pages at old URLs that send readers on to where a page
//! lives now.
//!
//! Declared in front matter (`"redirect_from": ["old/path"]` on the page that
//! moved) and in `[redirects]` in `docanvil.toml`. [`plan`] turns both into one
//! flat list of stub pages, plus the problems the build warns about and
//! `docanvil doctor` reports.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{Error, Result};
use crate::pipeline::frontmatter::FrontMatter;
use crate::project::{PageInfo, PageInventory};
use crate::util::html_escape;

/// One side of a redirect, as written.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Spec {
    /// `help/faq`: a page slug, expanded for every version and language.
    Slug(String),
    /// `/old/page.html`: a path from the site root (stored without the leading
    /// `/`), never expanded.
    Literal(String),
    /// `https://…`: another site. Only valid as a target.
    External(String),
}

/// Read one side of a redirect. `target` allows external URLs.
fn parse_spec(raw: &str, target: bool) -> std::result::Result<Spec, String> {
    let s = raw.trim().replace('\\', "/");
    if s.is_empty() {
        return Err("a redirect path is empty".to_string());
    }
    if s.contains("://") {
        return if target {
            Ok(Spec::External(s))
        } else {
            Err(format!(
                "\"{raw}\" is a URL, but an old path must be a page on this site"
            ))
        };
    }
    let bytes = s.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return Err(format!(
            "\"{raw}\" is a filesystem path. Use a page slug like \"guides/setup\" or a site path like \"/old.html\""
        ));
    }
    if s.contains(['?', '#']) {
        return Err(format!("\"{raw}\" can't contain ? or #"));
    }
    if s.split('/').any(|segment| segment == "..") {
        return Err(format!("\"{raw}\" can't contain .."));
    }

    if let Some(rest) = s.strip_prefix('/') {
        let rest = rest.trim_start_matches('/');
        let path = if rest.is_empty() || rest.ends_with('/') {
            format!("{rest}index.html")
        } else if rest.ends_with(".html") {
            rest.to_string()
        } else if rest
            .rsplit('/')
            .next()
            .is_some_and(|last| last.contains('.'))
        {
            return Err(format!(
                "\"{raw}\" isn't a page. A site path must end in .html or /"
            ));
        } else {
            format!("{rest}.html")
        };
        return Ok(Spec::Literal(path));
    }

    let slug = s.trim_end_matches('/');
    let slug = slug.strip_suffix(".html").unwrap_or(slug);
    if slug.is_empty() {
        return Err(format!("\"{raw}\" isn't a page slug"));
    }
    Ok(Spec::Slug(slug.to_string()))
}

/// A page that sends the reader to `href`: meta refresh for no-JS, a script that
/// keeps the old URL's `?query` and `#anchor`, `noindex` + canonical so search
/// engines move their ranking to the target, and a plain link as a last resort.
pub fn stub_html(href: &str, canonical: &str, lang: Option<&str>) -> String {
    let href_attr = html_escape(href);
    let canonical_attr = html_escape(canonical);
    // A JSON string is a valid JS string literal; escaping `<` stops a target
    // from closing the script element.
    let href_js = serde_json::to_string(href)
        .unwrap_or_default()
        .replace('<', "\\u003c");
    let lang_attr = lang
        .map(|l| format!(" lang=\"{}\"", html_escape(l)))
        .unwrap_or_default();
    format!(
        "<!DOCTYPE html>\n\
         <html{lang_attr}>\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <title>Redirecting…</title>\n\
         <meta name=\"robots\" content=\"noindex\">\n\
         <link rel=\"canonical\" href=\"{canonical_attr}\">\n\
         <meta http-equiv=\"refresh\" content=\"0; url={href_attr}\">\n\
         <script>\n\
         var u = new URL({href_js}, location.href);\n\
         if (!u.search) u.search = location.search;\n\
         if (!u.hash) u.hash = location.hash;\n\
         location.replace(u.href);\n\
         </script>\n\
         </head>\n\
         <body>\n\
         <p>Redirecting to <a href=\"{href_attr}\">{href_attr}</a>…</p>\n\
         </body>\n\
         </html>\n"
    )
}

pub const CHECK_TARGET_MISSING: &str = "redirect-target-missing";
pub const CHECK_SHADOWED: &str = "redirect-shadowed";
pub const CHECK_CONFLICT: &str = "redirect-conflict";
pub const CHECK_LOOP: &str = "redirect-loop";
pub const CHECK_INVALID: &str = "redirect-invalid";

/// One version's pages (or the whole site's, without versioning), as the build
/// sees them: front matter slugs applied, drafts moved to `inventory.drafts`.
pub struct PageSet<'a> {
    pub version: Option<&'a str>,
    pub inventory: &'a PageInventory,
    /// Front matter keyed like `inventory.pages`.
    pub front_matters: &'a HashMap<String, FrontMatter>,
}

/// Where a redirect goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A page this build writes: its output path (`v2/en/guide.html`).
    Page(String),
    /// Any other path on this site (`archive/legacy.html`).
    Path(String),
    /// Another site.
    External(String),
}

/// Where a redirect was declared. The variant order is the precedence: front
/// matter beats the `[redirects]` table, which beats `unprefixed`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Origin {
    FrontMatter(PathBuf),
    Table(String),
    Unprefixed,
}

/// One stub page to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    /// Output path of the stub, `/`-separated (`v2/en/setup.html`).
    pub from: String,
    pub to: Target,
    /// Language of the page it points to, for `<html lang>`.
    pub lang: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RedirectProblem {
    pub check: &'static str,
    pub origin: Origin,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct RedirectPlan {
    /// Sorted by `from`.
    pub redirects: Vec<Redirect>,
    pub problems: Vec<RedirectProblem>,
}

/// A possible stub, before precedence, shadowing and chains are worked out.
struct Candidate {
    from: String,
    to: Target,
    lang: Option<String>,
    origin: Origin,
    /// A `Path` target made from a slug that isn't a page here: dropped unless
    /// it leads to a page through another redirect.
    must_resolve: bool,
}

enum Found<'a> {
    Page(&'a PageInfo),
    Draft,
    Missing,
}

/// Work out every stub this build writes, from front matter `redirect_from`,
/// the `[redirects]` table and `unprefixed`.
pub fn plan(config: &Config, sets: &[PageSet]) -> RedirectPlan {
    let mut problems = Vec::new();
    let mut candidates = Vec::new();
    front_matter_candidates(sets, &mut candidates, &mut problems);
    let drafted = table_candidates(config, sets, &mut candidates, &mut problems);
    unprefixed_candidates(config, sets, &mut candidates);
    resolve(config, sets, candidates, &drafted, problems)
}

/// `v2/en/`, `v2/`, `en/` or empty.
fn prefix(version: Option<&str>, locale: Option<&str>) -> String {
    let mut prefix = String::new();
    for part in [version, locale].into_iter().flatten() {
        prefix.push_str(part);
        prefix.push('/');
    }
    prefix
}

fn output_path(page: &PageInfo) -> String {
    page.output_path.to_string_lossy().replace('\\', "/")
}

/// The language a site-level redirect is written in: the default, when the site
/// is translated.
fn default_lang(config: &Config) -> Option<&str> {
    if !config.is_i18n_enabled() {
        return None;
    }
    config
        .default_locale()
        .or(config.locale.enabled.first().map(String::as_str))
}

/// Every language a slug is expanded into (`[None]` without i18n).
fn languages(config: &Config) -> Vec<Option<&str>> {
    if config.is_i18n_enabled() {
        config
            .locale
            .enabled
            .iter()
            .map(|l| Some(l.as_str()))
            .collect()
    } else {
        vec![None]
    }
}

/// The current version's pages (or the only set, without versioning).
fn current_set<'s, 'a>(config: &Config, sets: &'s [PageSet<'a>]) -> Option<&'s PageSet<'a>> {
    match config.current_version() {
        Some(current) if config.is_versioning_enabled() => {
            sets.iter().find(|set| set.version == Some(current))
        }
        _ => sets.first(),
    }
}

fn find<'a>(inventory: &'a PageInventory, slug: &str, locale: Option<&str>) -> Found<'a> {
    let page = match locale {
        Some(l) => inventory.resolve_link_in_locale(slug, l),
        None => inventory.resolve_link(slug),
    };
    match page {
        Some(page) => Found::Page(page),
        None if inventory.resolve_draft(slug, locale).is_some() => Found::Draft,
        None => Found::Missing,
    }
}

fn invalid(origin: Origin, why: String) -> RedirectProblem {
    RedirectProblem {
        check: CHECK_INVALID,
        origin,
        message: format!("Invalid redirect: {why}"),
    }
}

/// `redirect_from` as a list of strings (a single string is fine too).
fn redirect_from_entries(value: &serde_json::Value) -> Option<Vec<&str>> {
    match value {
        serde_json::Value::String(s) => Some(vec![s.as_str()]),
        serde_json::Value::Array(items) => items.iter().map(|v| v.as_str()).collect(),
        _ => None,
    }
}

fn front_matter_candidates(
    sets: &[PageSet],
    out: &mut Vec<Candidate>,
    problems: &mut Vec<RedirectProblem>,
) {
    for set in sets {
        // A redirect on any translation applies to every translation of the page.
        let mut translations: HashMap<&str, Vec<&PageInfo>> = HashMap::new();
        for key in &set.inventory.ordered {
            let page = &set.inventory.pages[key];
            translations
                .entry(page.slug.as_str())
                .or_default()
                .push(page);
        }

        for key in &set.inventory.ordered {
            let page = &set.inventory.pages[key];
            let Some(value) = set
                .front_matters
                .get(key)
                .and_then(|fm| fm.redirect_from.as_ref())
            else {
                continue;
            };
            let origin = Origin::FrontMatter(page.source_path.clone());
            let Some(entries) = redirect_from_entries(value) else {
                problems.push(invalid(
                    origin,
                    format!(
                        "front matter \"redirect_from\" must be a list of old paths, like [\"old-page\"] (found {value})"
                    ),
                ));
                continue;
            };
            for entry in entries {
                match parse_spec(entry, false) {
                    Ok(Spec::Slug(slug)) => {
                        for variant in &translations[page.slug.as_str()] {
                            out.push(Candidate {
                                from: format!(
                                    "{}{slug}.html",
                                    prefix(variant.version.as_deref(), variant.locale.as_deref())
                                ),
                                to: Target::Page(output_path(variant)),
                                lang: variant.locale.clone(),
                                origin: origin.clone(),
                                must_resolve: false,
                            });
                        }
                    }
                    Ok(Spec::Literal(path)) => out.push(Candidate {
                        from: path,
                        to: Target::Page(output_path(page)),
                        lang: page.locale.clone(),
                        origin: origin.clone(),
                        must_resolve: false,
                    }),
                    // parse_spec only returns External for targets.
                    Ok(Spec::External(_)) => {}
                    Err(why) => problems.push(invalid(origin.clone(), why)),
                }
            }
        }
    }
}

/// Candidates from `[redirects]`. Returns the keys whose target is a draft left
/// out of this build, which are skipped without a warning.
fn table_candidates(
    config: &Config,
    sets: &[PageSet],
    out: &mut Vec<Candidate>,
    problems: &mut Vec<RedirectProblem>,
) -> HashSet<String> {
    let mut drafted = HashSet::new();
    let current = current_set(config, sets);
    let languages = languages(config);

    for (key, value) in &config.redirects.paths {
        let origin = Origin::Table(key.clone());
        let (from, to) = match (parse_spec(key, false), parse_spec(value, true)) {
            (Ok(from), Ok(to)) => (from, to),
            (Err(why), _) | (_, Err(why)) => {
                problems.push(invalid(origin, why));
                continue;
            }
        };

        // A slug is written in every version and language; a literal path once,
        // resolving its target in the current version and default language.
        let contexts: Vec<(&PageSet, Option<&str>)> = match &from {
            Spec::Slug(_) => sets
                .iter()
                .flat_map(|set| languages.iter().map(move |lang| (set, *lang)))
                .collect(),
            _ => current
                .map(|set| (set, default_lang(config)))
                .into_iter()
                .collect(),
        };

        for (set, lang) in contexts {
            let from_path = match &from {
                Spec::Slug(slug) => format!("{}{slug}.html", prefix(set.version, lang)),
                Spec::Literal(path) => path.clone(),
                // parse_spec only returns External for targets.
                Spec::External(_) => continue,
            };
            let (to, lang, must_resolve) = match &to {
                Spec::External(url) => (Target::External(url.clone()), lang, false),
                Spec::Literal(path) => (Target::Path(path.clone()), lang, false),
                Spec::Slug(slug) => match find(set.inventory, slug, lang) {
                    Found::Page(page) => (
                        Target::Page(output_path(page)),
                        page.locale.as_deref(),
                        false,
                    ),
                    Found::Draft => {
                        drafted.insert(key.clone());
                        continue;
                    }
                    Found::Missing => (
                        Target::Path(format!("{}{slug}.html", prefix(set.version, lang))),
                        lang,
                        true,
                    ),
                },
            };
            out.push(Candidate {
                from: from_path,
                to,
                lang: lang.map(String::from),
                origin: origin.clone(),
                must_resolve,
            });
        }
    }
    drafted
}

/// `page.html` → the current version and default language, for sites whose URLs
/// moved when versions or languages were switched on.
fn unprefixed_candidates(config: &Config, sets: &[PageSet], out: &mut Vec<Candidate>) {
    if !config.redirects.unprefixed || !(config.is_i18n_enabled() || config.is_versioning_enabled())
    {
        return;
    }
    let Some(set) = current_set(config, sets) else {
        return;
    };
    let lang = default_lang(config);
    for key in &set.inventory.ordered {
        let page = &set.inventory.pages[key];
        if page.locale.as_deref() != lang {
            continue;
        }
        let from = format!("{}.html", page.slug);
        // The site root already redirects to the current version and language.
        if from == "index.html" {
            continue;
        }
        out.push(Candidate {
            from,
            to: Target::Page(output_path(page)),
            lang: page.locale.clone(),
            origin: Origin::Unprefixed,
            must_resolve: false,
        });
    }
}

fn describe(origin: &Origin) -> String {
    match origin {
        Origin::FrontMatter(path) => format!("the front matter of {}", path.display()),
        Origin::Table(key) => format!("[redirects] (\"{key}\")"),
        Origin::Unprefixed => "[redirects] unprefixed".to_string(),
    }
}

/// Drop stubs that would replace a page, pick one stub per path, follow chains
/// and report what went wrong.
fn resolve(
    config: &Config,
    sets: &[PageSet],
    candidates: Vec<Candidate>,
    drafted: &HashSet<String>,
    mut problems: Vec<RedirectProblem>,
) -> RedirectPlan {
    let pages: HashSet<String> = sets
        .iter()
        .flat_map(|set| set.inventory.pages.values())
        .map(output_path)
        .collect();
    // Paths a stub must never replace: every page (drafts too, so publishing one
    // doesn't change what its URL does) and the files the build writes itself.
    let mut taken: HashSet<String> = sets
        .iter()
        .flat_map(|set| set.inventory.drafts.values())
        .map(output_path)
        .chain(pages.iter().cloned())
        .collect();
    taken.insert("404.html".to_string());
    if config.is_i18n_enabled() || config.is_versioning_enabled() {
        taken.insert("index.html".to_string());
    }

    let mut by_from: BTreeMap<String, Vec<Candidate>> = BTreeMap::new();
    for candidate in candidates {
        if taken.contains(&candidate.from) {
            if candidate.origin != Origin::Unprefixed {
                problems.push(RedirectProblem {
                    check: CHECK_SHADOWED,
                    origin: candidate.origin.clone(),
                    message: format!(
                        "Redirect from {} is never used: a page already lives there",
                        candidate.from
                    ),
                });
            }
            continue;
        }
        by_from
            .entry(candidate.from.clone())
            .or_default()
            .push(candidate);
    }

    let mut winners: BTreeMap<String, Candidate> = BTreeMap::new();
    for (from, mut group) in by_from {
        // Stable sort, so equal origins keep their declaration order.
        group.sort_by(|a, b| a.origin.cmp(&b.origin));
        let best = group.remove(0);
        for other in &group {
            if other.origin != Origin::Unprefixed && other.to != best.to {
                problems.push(RedirectProblem {
                    check: CHECK_CONFLICT,
                    origin: other.origin.clone(),
                    message: format!(
                        "Redirect from {from} is ignored: {} already sends it somewhere else",
                        describe(&best.origin)
                    ),
                });
            }
        }
        winners.insert(from, best);
    }

    // Follow chains, so every stub points straight at where the reader ends up.
    let mut redirects = Vec::new();
    let mut tables_tried: BTreeSet<&str> = BTreeSet::new();
    let mut tables_resolved: HashSet<&str> = HashSet::new();
    for (from, candidate) in &winners {
        if let Origin::Table(key) = &candidate.origin {
            tables_tried.insert(key.as_str());
        }
        let mut to = candidate.to.clone();
        let mut lang = candidate.lang.clone();
        let mut must_resolve = candidate.must_resolve;
        let mut seen: HashSet<String> = HashSet::from([from.clone()]);
        let mut looped = false;
        while let Target::Path(path) = &to {
            if pages.contains(path) {
                to = Target::Page(path.clone());
                break;
            }
            let Some(next) = winners.get(path) else {
                break;
            };
            if !seen.insert(path.clone()) {
                looped = true;
                break;
            }
            to = next.to.clone();
            lang = next.lang.clone();
            must_resolve = next.must_resolve;
        }

        if looped {
            problems.push(RedirectProblem {
                check: CHECK_LOOP,
                origin: candidate.origin.clone(),
                message: format!("Redirect from {from} goes round in a loop, so it's skipped"),
            });
            continue;
        }
        if must_resolve && matches!(to, Target::Path(_)) {
            continue;
        }
        if let Origin::Table(key) = &candidate.origin {
            tables_resolved.insert(key.as_str());
        }
        redirects.push(Redirect {
            from: from.clone(),
            to,
            lang,
        });
    }

    for key in tables_tried {
        if !tables_resolved.contains(key) && !drafted.contains(key) {
            let looped = problems
                .iter()
                .any(|p| p.check == CHECK_LOOP && p.origin == Origin::Table(key.to_string()));
            if looped {
                continue;
            }
            let value = &config.redirects.paths[key];
            problems.push(RedirectProblem {
                check: CHECK_TARGET_MISSING,
                origin: Origin::Table(key.to_string()),
                message: format!(
                    "Redirect \"{key}\" points at \"{value}\", which isn't a page in any version or language"
                ),
            });
        }
    }

    let mut unique = HashSet::new();
    problems.retain(|p| unique.insert(p.clone()));
    RedirectPlan {
        redirects,
        problems,
    }
}

/// Write each stub into `output_dir`. Links use `base_url`; the canonical link
/// uses `site_url` when it's set.
pub fn write_stubs(
    output_dir: &Path,
    redirects: &[Redirect],
    base_url: &str,
    site_url: Option<&str>,
) -> Result<()> {
    for redirect in redirects {
        let (href, canonical) = match &redirect.to {
            Target::Page(path) | Target::Path(path) => (
                format!("{base_url}{path}"),
                site_url.map(|site| format!("{site}{path}")),
            ),
            Target::External(url) => (url.clone(), None),
        };
        let html = stub_html(
            &href,
            canonical.as_deref().unwrap_or(&href),
            redirect.lang.as_deref(),
        );
        let path = output_dir.join(&redirect.from);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::General(format!("{}: {e}", parent.display())))?;
        }
        std::fs::write(&path, html)
            .map_err(|e| Error::General(format!("{}: {e}", path.display())))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn slugs_ignore_trailing_slash_and_html() {
        for raw in ["setup", "setup/", "setup.html", " setup "] {
            assert_eq!(
                parse_spec(raw, false),
                Ok(Spec::Slug("setup".into())),
                "{raw}"
            );
        }
        assert_eq!(
            parse_spec("guides\\setup", false),
            Ok(Spec::Slug("guides/setup".into()))
        );
    }

    #[test]
    fn leading_slash_is_a_literal_site_path() {
        assert_eq!(
            parse_spec("/old.html", false),
            Ok(Spec::Literal("old.html".into()))
        );
        assert_eq!(
            parse_spec("/old/page", false),
            Ok(Spec::Literal("old/page.html".into()))
        );
        assert_eq!(
            parse_spec("/old/", false),
            Ok(Spec::Literal("old/index.html".into()))
        );
        assert_eq!(
            parse_spec("/", false),
            Ok(Spec::Literal("index.html".into()))
        );
    }

    #[test]
    fn urls_are_only_allowed_as_targets() {
        assert_eq!(
            parse_spec("https://blog.example.com", true),
            Ok(Spec::External("https://blog.example.com".into()))
        );
        assert!(parse_spec("https://blog.example.com", false).is_err());
    }

    #[test]
    fn bad_paths_are_rejected() {
        for raw in [
            "",
            "  ",
            "../secret",
            "a/../b",
            "C:/docs/x.md",
            "old#part",
            "page?x=1",
            "/report.pdf",
            ".html",
        ] {
            assert!(parse_spec(raw, true).is_err(), "{raw:?} should be invalid");
        }
    }

    #[test]
    fn stub_redirects_three_ways() {
        let html = stub_html(
            "/docs/guides/install.html",
            "https://x.dev/docs/guides/install.html",
            Some("fr"),
        );
        assert!(html.contains("<html lang=\"fr\">"));
        assert!(html.contains(
            "<meta http-equiv=\"refresh\" content=\"0; url=/docs/guides/install.html\">"
        ));
        assert!(
            html.contains(
                "<link rel=\"canonical\" href=\"https://x.dev/docs/guides/install.html\">"
            )
        );
        assert!(html.contains("<meta name=\"robots\" content=\"noindex\">"));
        assert!(html.contains("location.replace"));
        assert!(html.contains("<a href=\"/docs/guides/install.html\">"));
    }

    #[test]
    fn stub_keeps_query_and_hash() {
        // The script copies the old URL's ?query and #anchor onto the target unless
        // the target sets its own.
        let html = stub_html("/new.html", "/new.html", None);
        assert!(html.contains("if (!u.search) u.search = location.search;"));
        assert!(html.contains("if (!u.hash) u.hash = location.hash;"));
        assert!(html.contains("<html>"));
    }

    #[test]
    fn stub_escapes_its_target() {
        let html = stub_html("/a\"b<c>.html", "/a\"b<c>.html", None);
        assert!(html.contains("url=/a&quot;b&lt;c&gt;.html"));
        // A target can't close the script element early.
        assert!(!html.contains("</script></script>"));
        assert!(html.contains("\\u003c"));
    }

    /// A scanned site: `(version, inventory, front matter)` per version, built the
    /// way the build does it (slugs applied, drafts left out unless `drafts`).
    struct Site {
        _dir: tempfile::TempDir,
        config: Config,
        versions: Vec<(Option<String>, PageInventory, HashMap<String, FrontMatter>)>,
    }

    impl Site {
        fn new(config: &str, files: &[(&str, &str)]) -> Self {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join("docanvil.toml"), config).unwrap();
            for (path, content) in files {
                let p = dir.path().join("docs").join(path);
                fs::create_dir_all(p.parent().unwrap()).unwrap();
                fs::write(p, content).unwrap();
            }
            let config = Config::load(dir.path()).unwrap();
            let versions = crate::cli::build::scan_site(dir.path(), &config).unwrap();
            Site {
                _dir: dir,
                config,
                versions,
            }
        }

        fn plan(&self) -> RedirectPlan {
            let sets: Vec<PageSet> = self
                .versions
                .iter()
                .map(|(version, inventory, front_matters)| PageSet {
                    version: version.as_deref(),
                    inventory,
                    front_matters,
                })
                .collect();
            plan(&self.config, &sets)
        }
    }

    const PLAIN: &str = "[project]\nname = \"T\"\n";
    const I18N: &str =
        "[project]\nname = \"T\"\n\n[locale]\ndefault = \"en\"\nenabled = [\"en\", \"fr\"]\n";
    const VERSIONED: &str = "[project]\nname = \"T\"\n\n[version]\nenabled = [\"v1\", \"v2\"]\n";

    fn moved(from: &[&str]) -> String {
        format!(
            "---\n{{\"redirect_from\": {}}}\n---\n# Install",
            serde_json::to_string(from).unwrap()
        )
    }

    fn pairs(plan: &RedirectPlan) -> Vec<(&str, Target)> {
        plan.redirects
            .iter()
            .map(|r| (r.from.as_str(), r.to.clone()))
            .collect()
    }

    fn page(path: &str) -> Target {
        Target::Page(path.to_string())
    }

    fn checks(plan: &RedirectPlan) -> Vec<&'static str> {
        plan.problems.iter().map(|p| p.check).collect()
    }

    #[test]
    fn front_matter_redirects_to_the_page() {
        let site = Site::new(
            PLAIN,
            &[
                ("index.md", "# Home"),
                ("guides/install.md", &moved(&["setup", "/old/install.html"])),
            ],
        );
        let plan = site.plan();
        assert_eq!(
            pairs(&plan),
            vec![
                ("old/install.html", page("guides/install.html")),
                ("setup.html", page("guides/install.html")),
            ]
        );
        assert!(plan.problems.is_empty(), "{:?}", plan.problems);
    }

    #[test]
    fn front_matter_applies_to_every_translation() {
        let site = Site::new(
            I18N,
            &[
                ("guides/install.en.md", &moved(&["setup"])),
                ("guides/install.fr.md", "# Installer"),
            ],
        );
        let plan = site.plan();
        assert_eq!(
            pairs(&plan),
            vec![
                ("en/setup.html", page("en/guides/install.html")),
                ("fr/setup.html", page("fr/guides/install.html")),
            ]
        );
        let fr = plan
            .redirects
            .iter()
            .find(|r| r.from == "fr/setup.html")
            .unwrap();
        assert_eq!(fr.lang.as_deref(), Some("fr"));
    }

    #[test]
    fn table_slugs_expand_per_version_where_the_target_exists() {
        let config = format!("{VERSIONED}\n[redirects]\n\"old-faq\" = \"help/faq\"\n");
        let site = Site::new(
            &config,
            &[
                ("v1/index.md", "# One"),
                ("v2/index.md", "# Two"),
                ("v2/help/faq.md", "# FAQ"),
            ],
        );
        let plan = site.plan();
        assert_eq!(
            pairs(&plan),
            vec![("v2/old-faq.html", page("v2/help/faq.html"))]
        );
        assert!(plan.problems.is_empty(), "{:?}", plan.problems);
    }

    #[test]
    fn table_literal_and_external_targets() {
        let config = format!(
            "{PLAIN}\n[redirects]\n\"/blog/\" = \"https://blog.example.com\"\n\"/start.html\" = \"guide\"\n\"legacy\" = \"/archive/legacy.html\"\n"
        );
        let site = Site::new(&config, &[("index.md", "# Home"), ("guide.md", "# Guide")]);
        let plan = site.plan();
        assert_eq!(
            pairs(&plan),
            vec![
                (
                    "blog/index.html",
                    Target::External("https://blog.example.com".into())
                ),
                ("legacy.html", Target::Path("archive/legacy.html".into())),
                ("start.html", page("guide.html")),
            ]
        );
    }

    #[test]
    fn unprefixed_points_old_urls_at_the_current_version() {
        let config = format!("{VERSIONED}\n[redirects]\nunprefixed = true\n");
        let site = Site::new(
            &config,
            &[
                ("v1/index.md", "# One"),
                ("v1/old.md", "# Old"),
                ("v2/index.md", "# Two"),
                ("v2/guide.md", "# Guide"),
            ],
        );
        // index.html is the root redirect already; v1-only pages get nothing.
        assert_eq!(
            pairs(&site.plan()),
            vec![("guide.html", page("v2/guide.html"))]
        );
    }

    #[test]
    fn unprefixed_does_nothing_on_a_plain_site() {
        let config = format!("{PLAIN}\n[redirects]\nunprefixed = true\n");
        let site = Site::new(&config, &[("guide.md", "# Guide")]);
        assert!(site.plan().redirects.is_empty());
    }

    #[test]
    fn chains_are_flattened() {
        let config = format!("{PLAIN}\n[redirects]\n\"a\" = \"/b.html\"\n\"b\" = \"guide\"\n");
        let site = Site::new(&config, &[("guide.md", "# Guide")]);
        assert_eq!(
            pairs(&site.plan()),
            vec![
                ("a.html", page("guide.html")),
                ("b.html", page("guide.html"))
            ]
        );
    }

    #[test]
    fn a_table_slug_can_point_at_another_redirect() {
        let config = format!("{PLAIN}\n[redirects]\n\"oldest\" = \"setup\"\n");
        let site = Site::new(&config, &[("guides/install.md", &moved(&["setup"]))]);
        let plan = site.plan();
        assert_eq!(
            pairs(&plan),
            vec![
                ("oldest.html", page("guides/install.html")),
                ("setup.html", page("guides/install.html")),
            ]
        );
        assert!(plan.problems.is_empty(), "{:?}", plan.problems);
    }

    #[test]
    fn loops_are_reported_and_skipped() {
        let config = format!("{PLAIN}\n[redirects]\n\"a\" = \"/b.html\"\n\"b\" = \"/a.html\"\n");
        let site = Site::new(&config, &[("index.md", "# Home")]);
        let plan = site.plan();
        assert!(plan.redirects.is_empty());
        assert_eq!(checks(&plan), vec![CHECK_LOOP, CHECK_LOOP]);
    }

    #[test]
    fn missing_target_is_reported() {
        let config = format!("{PLAIN}\n[redirects]\n\"old\" = \"nowhere\"\n");
        let site = Site::new(&config, &[("index.md", "# Home")]);
        let plan = site.plan();
        assert!(plan.redirects.is_empty());
        assert_eq!(checks(&plan), vec![CHECK_TARGET_MISSING]);
        assert_eq!(plan.problems[0].origin, Origin::Table("old".into()));
        assert!(plan.problems[0].message.contains("nowhere"));
    }

    #[test]
    fn real_page_is_never_replaced() {
        let site = Site::new(
            PLAIN,
            &[
                ("setup.md", "# Setup"),
                ("guides/install.md", &moved(&["setup"])),
            ],
        );
        let plan = site.plan();
        assert!(plan.redirects.is_empty());
        assert_eq!(checks(&plan), vec![CHECK_SHADOWED]);
    }

    #[test]
    fn front_matter_beats_the_table() {
        let config = format!("{PLAIN}\n[redirects]\n\"setup\" = \"guide\"\n");
        let site = Site::new(
            &config,
            &[
                ("guide.md", "# Guide"),
                ("guides/install.md", &moved(&["setup"])),
            ],
        );
        let plan = site.plan();
        assert_eq!(
            pairs(&plan),
            vec![("setup.html", page("guides/install.html"))]
        );
        assert_eq!(checks(&plan), vec![CHECK_CONFLICT]);
        assert_eq!(plan.problems[0].origin, Origin::Table("setup".into()));
    }

    #[test]
    fn the_table_overrides_unprefixed_quietly() {
        let config = format!(
            "{I18N}\n[redirects]\nunprefixed = true\n\"/guide.html\" = \"https://example.com\"\n"
        );
        let site = Site::new(&config, &[("guide.en.md", "# Guide")]);
        let plan = site.plan();
        assert_eq!(
            pairs(&plan),
            vec![("guide.html", Target::External("https://example.com".into()))]
        );
        assert!(plan.problems.is_empty(), "{:?}", plan.problems);
    }

    #[test]
    fn draft_target_is_skipped_quietly() {
        let config = format!("{PLAIN}\n[redirects]\n\"old\" = \"wip\"\n");
        let site = Site::new(
            &config,
            &[
                ("index.md", "# Home"),
                (
                    "wip.md",
                    "---\n{\"draft\": true, \"redirect_from\": [\"older\"]}\n---\n# WIP",
                ),
            ],
        );
        let plan = site.plan();
        assert!(plan.redirects.is_empty(), "{:?}", plan.redirects);
        assert!(plan.problems.is_empty(), "{:?}", plan.problems);
    }

    #[test]
    fn invalid_entries_are_reported() {
        let config = format!("{PLAIN}\n[redirects]\n\"https://x.dev/a\" = \"guide\"\n");
        let site = Site::new(
            &config,
            &[
                ("guide.md", "# Guide"),
                ("a.md", "---\n{\"redirect_from\": 5}\n---\n# A"),
                ("b.md", "---\n{\"redirect_from\": [\"../up\"]}\n---\n# B"),
            ],
        );
        let plan = site.plan();
        assert!(plan.redirects.is_empty());
        assert_eq!(
            checks(&plan),
            vec![CHECK_INVALID, CHECK_INVALID, CHECK_INVALID]
        );
    }

    #[test]
    fn a_single_string_redirect_from_is_accepted() {
        let site = Site::new(
            PLAIN,
            &[(
                "guide.md",
                "---\n{\"redirect_from\": \"start\"}\n---\n# Guide",
            )],
        );
        assert_eq!(
            pairs(&site.plan()),
            vec![("start.html", page("guide.html"))]
        );
    }

    #[test]
    fn write_stubs_uses_base_and_site_url() {
        let dir = tempfile::tempdir().unwrap();
        let redirects = vec![
            Redirect {
                from: "old/setup.html".into(),
                to: page("guides/install.html"),
                lang: None,
            },
            Redirect {
                from: "blog.html".into(),
                to: Target::External("https://b.dev".into()),
                lang: None,
            },
        ];
        write_stubs(
            dir.path(),
            &redirects,
            "/docs/",
            Some("https://x.dev/docs/"),
        )
        .unwrap();

        let stub = fs::read_to_string(dir.path().join("old/setup.html")).unwrap();
        assert!(stub.contains("url=/docs/guides/install.html"));
        assert!(stub.contains("href=\"https://x.dev/docs/guides/install.html\""));
        let blog = fs::read_to_string(dir.path().join("blog.html")).unwrap();
        assert!(blog.contains("url=https://b.dev"));
    }
}
