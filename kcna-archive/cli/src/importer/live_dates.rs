//! Publication dates for live articles, read from stored list snapshots.

use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
};

use scraper::{Html, Selector};
use time::{Date, Month};
use tracing::warn;

use crate::{
    error::{Error, Result, io_error},
    html,
};

const MAX_LIVE_ARTICLES: usize = 250_000;

#[derive(Clone, Debug)]
pub struct LiveInfo {
    pub date: Date,
    pub raw_date: String,
    /// Category names as the site titles them, for example `국내소식`.
    pub categories: BTreeSet<String>,
    /// The opaque category identifiers used in list URLs.
    pub category_ids: BTreeSet<String>,
}

pub type Dates = HashMap<String, LiveInfo>;

/// Temporary download files and other strays are not articles.
pub fn html_files(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "html")
        })
        .collect()
}

/// Reads every stored list snapshot of an edition into an article date map.
/// Blocking: parses a few thousand pages.
pub fn load_dates(root: &Path) -> Result<Dates> {
    let mut result = HashMap::with_capacity(10_000);
    for category_dir in sorted_entries(root, true)? {
        let category = category_dir
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| Error::InvalidRecord("category path is not UTF-8".to_owned()))?
            .to_owned();
        for path in html_files(sorted_entries(&category_dir, false)?) {
            let bytes = std::fs::read(&path).map_err(io_error(&path))?;
            parse_list_page(&html::decode(&bytes), &category, &mut result)?;
            if result.len() > MAX_LIVE_ARTICLES {
                return Err(Error::CrawlLimit {
                    limit: MAX_LIVE_ARTICLES,
                });
            }
        }
    }
    Ok(result)
}

fn sorted_entries(root: &Path, directories: bool) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(root).map_err(io_error(root))? {
        let entry = entry.map_err(io_error(root))?;
        let file_type = entry.file_type().map_err(io_error(entry.path()))?;
        if file_type.is_dir() == directories {
            paths.push(entry.path());
        }
    }
    paths.sort_unstable();
    Ok(paths)
}

fn parse_list_page(document: &str, category: &str, result: &mut Dates) -> Result<()> {
    let html = Html::parse_document(document);
    let row = Selector::parse("h5").map_err(|error| Error::Selector(error.to_string()))?;
    let anchor = Selector::parse("a[href]").map_err(|error| Error::Selector(error.to_string()))?;
    let span = Selector::parse("span").map_err(|error| Error::Selector(error.to_string()))?;
    let name = category_name(&html)?;
    for node in html.select(&row) {
        let Some(href) = node
            .select(&anchor)
            .find_map(|link| link.value().attr("href"))
        else {
            continue;
        };
        if !href.contains("/article/detail/") {
            continue;
        }
        let Some(article_id) = href.rsplit('/').next().filter(|value| !value.is_empty()) else {
            continue;
        };
        let Some(raw_date) = node
            .select(&span)
            .map(|value| value.text().collect::<String>())
            .find(|value| value.contains("20"))
        else {
            continue;
        };
        // One unreadable row must not abort an edition's import.
        let date = match parse_date(&raw_date) {
            Ok(date) => date,
            Err(error) => {
                warn!(article_id, error = %error, "Skipping list row with unreadable date");
                continue;
            }
        };
        let entry = result
            .entry(article_id.to_owned())
            .or_insert_with(|| LiveInfo {
                date,
                raw_date: raw_date.clone(),
                categories: BTreeSet::new(),
                category_ids: BTreeSet::new(),
            });
        if entry.date != date {
            // Snapshots are read in a stable order, so the first date wins
            // deterministically.
            warn!(article_id, kept = %entry.date, ignored = %date, "Conflicting list dates");
        }
        entry.category_ids.insert(category.to_owned());
        if let Some(name) = &name {
            entry.categories.insert(name.clone());
        }
    }
    Ok(())
}

/// Reads the category name from a list page title such as `조선중앙통신 | 국내소식`.
fn category_name(html: &Html) -> Result<Option<String>> {
    let title = Selector::parse("title").map_err(|error| Error::Selector(error.to_string()))?;
    Ok(html
        .select(&title)
        .next()
        .map(|node| node.text().collect::<String>())
        .and_then(|text| {
            text.rsplit_once('|')
                .map(|(_, name)| name.split_whitespace().collect::<Vec<_>>().join(" "))
        })
        .filter(|name| !name.is_empty()))
}

fn parse_date(value: &str) -> Result<Date> {
    let trimmed = value.trim().trim_start_matches('[').trim_end_matches(']');
    let parts = trimmed.trim_end_matches('.').split('.').collect::<Vec<_>>();
    let [year, month, day] = parts.as_slice() else {
        return Err(Error::InvalidRecord(format!("invalid live date {value}")));
    };
    let year = year
        .trim()
        .parse::<i32>()
        .map_err(|error| Error::InvalidRecord(error.to_string()))?;
    let month = month
        .trim()
        .parse::<u8>()
        .map_err(|error| Error::InvalidRecord(error.to_string()))?;
    let day = day
        .trim()
        .parse::<u8>()
        .map_err(|error| Error::InvalidRecord(error.to_string()))?;
    let month = Month::try_from(month).map_err(|error| Error::InvalidRecord(error.to_string()))?;
    Date::from_calendar_date(year, month, day)
        .map_err(|error| Error::InvalidRecord(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROW: &str = r#"<h5 class="block"><a href="/kp/article/detail/abc123">제목</a><span>[2026.9.2.]</span></h5>"#;

    #[test]
    fn parses_kcna_list_date() -> Result<()> {
        let date = parse_date("[2026.8.31.]")?;
        assert_eq!(
            date,
            Date::from_calendar_date(2026, Month::August, 31)
                .map_err(|error| Error::InvalidRecord(error.to_string()))?
        );
        assert!(parse_date("[2026.13.1.]").is_err());
        assert!(parse_date("2026").is_err());
        Ok(())
    }

    #[test]
    fn reads_dates_and_categories_from_list_rows() -> Result<()> {
        let mut dates = Dates::new();
        parse_list_page(ROW, "first", &mut dates)?;
        parse_list_page(ROW, "second", &mut dates)?;
        let info = dates.get("abc123");
        assert!(info.is_some_and(|info| {
            u8::from(info.date.month()) == 9
                && info.date.day() == 2
                && info.category_ids.len() == 2
                && info.categories.is_empty()
        }));
        let titled = format!(
            "<html><head><title>조선중앙통신 | 국내소식</title></head><body>{ROW}</body></html>"
        );
        parse_list_page(&titled, "third", &mut dates)?;
        assert!(dates.get("abc123").is_some_and(|info| {
            info.categories.iter().eq(["국내소식"]) && info.category_ids.len() == 3
        }));
        Ok(())
    }

    #[test]
    fn keeps_the_first_date_and_skips_unreadable_rows() -> Result<()> {
        let mut dates = Dates::new();
        parse_list_page(ROW, "first", &mut dates)?;
        parse_list_page(&ROW.replace("2026.9.2.", "2026.9.3."), "first", &mut dates)?;
        parse_list_page(
            &ROW.replace("abc123", "def456")
                .replace("2026.9.2.", "2026.99.2."),
            "first",
            &mut dates,
        )?;
        assert!(dates.get("abc123").is_some_and(|info| info.date.day() == 2));
        assert!(!dates.contains_key("def456"));
        Ok(())
    }

    #[test]
    fn ignores_non_article_files() {
        let files = html_files(vec![
            PathBuf::from("a/abc.html"),
            PathBuf::from("a/abc.html.part"),
            PathBuf::from("a/notes.txt"),
        ]);
        assert_eq!(files, [PathBuf::from("a/abc.html")]);
    }
}
