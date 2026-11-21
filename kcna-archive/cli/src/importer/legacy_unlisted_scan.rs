//! Finds stored historical article pages that the public inventory does not list.

use std::{
    collections::HashSet,
    io::{BufRead, BufReader},
    path::Path,
    sync::LazyLock,
};

use regex::Regex;
use serde::Deserialize;
use time::{Date, Month};

use crate::{
    database::types::Language,
    error::{Error, Result, io_error},
};

/// Bound on directory entries visited under the article directories.
const MAX_ENTRIES: usize = 2_000_000;

// Korean: calendar/2025/06/06-20/2025-0620-001.html
static KOREAN_ARTICLE: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r"(?i)^calendar/\d{4}/\d{2}/\d{2}-\d{2}/(\d{4})-(\d{2})(\d{2})-\d{3}\.html?$").ok()
});
// English and Spanish: item/2025/202510/news31/20251031-12ee.html
static TRANSLATED_ARTICLE: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r"(?i)^item/\d{4}/\d{6}/news\d{2}/(\d{4})(\d{2})(\d{2})-\d{2}(ee|ss)\.html?$").ok()
});

#[derive(Deserialize)]
struct SeedUrl {
    url: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Unlisted {
    /// Path below the mirror root, which is also the URL path.
    pub relative: String,
    pub date: Date,
    pub language: Language,
}

/// Walks the article directories and returns article pages the inventory lacks,
/// in path order.
pub fn find_unlisted(root: &Path, seed_index: &Path) -> Result<Vec<Unlisted>> {
    let listed = listed_paths(seed_index)?;
    let mut unlisted = Vec::new();
    let mut visited = 0_usize;
    let mut directories = vec![root.join("calendar"), root.join("item")];
    while let Some(directory) = directories.pop() {
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            // A corpus collected only from the inventory may lack a directory.
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => continue,
            Err(source) => return Err(io_error(&directory)(source)),
        };
        for entry in entries {
            let entry = entry.map_err(io_error(&directory))?;
            visited += 1;
            if visited > MAX_ENTRIES {
                return Err(Error::CrawlLimit { limit: MAX_ENTRIES });
            }
            let path = entry.path();
            let metadata = entry.metadata().map_err(io_error(&path))?;
            if metadata.is_dir() {
                directories.push(path);
                continue;
            }
            // The mirror answers a few article URLs with an empty body. Those
            // are stored as evidence but hold nothing to import.
            if metadata.len() == 0 {
                continue;
            }
            let Some(relative) = relative_path(root, &path) else {
                continue;
            };
            if listed.contains(&relative) {
                continue;
            }
            if let Some(article) = classify(&relative)? {
                unlisted.push(article);
            }
        }
    }
    unlisted.sort_unstable_by(|left, right| left.relative.cmp(&right.relative));
    Ok(unlisted)
}

fn relative_path(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()
        .and_then(Path::to_str)
        .map(str::to_owned)
}

/// URL paths named by the inventory. A missing inventory lists nothing.
fn listed_paths(seed_index: &Path) -> Result<HashSet<String>> {
    let file = match std::fs::File::open(seed_index) {
        Ok(file) => file,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Ok(HashSet::new());
        }
        Err(source) => return Err(io_error(seed_index)(source)),
    };
    let mut listed = HashSet::with_capacity(300_000);
    // Streamed line by line: the inventory is a few hundred megabytes.
    for line in BufReader::new(file).lines() {
        let line = line.map_err(io_error(seed_index))?;
        // Malformed lines are reported by the listed importer; here they only
        // mean the page stays a candidate.
        if let Ok(record) = serde_json::from_str::<SeedUrl>(&line)
            && let Ok(url) = url::Url::parse(&record.url)
        {
            listed.insert(url.path().trim_start_matches('/').to_owned());
        }
    }
    Ok(listed)
}

/// Reads the date and language from an article path; `None` for index pages.
fn classify(relative: &str) -> Result<Option<Unlisted>> {
    let expression = |cell: &'static LazyLock<Option<Regex>>| {
        cell.as_ref()
            .ok_or_else(|| Error::Selector("built-in expression is invalid".to_owned()))
    };
    let (captures, language) =
        if let Some(captures) = expression(&KOREAN_ARTICLE)?.captures(relative) {
            (captures, Language::Korean)
        } else if let Some(captures) = expression(&TRANSLATED_ARTICLE)?.captures(relative) {
            let language = match captures
                .get(4)
                .map(|suffix| suffix.as_str().to_ascii_lowercase())
            {
                Some(suffix) if suffix == "ss" => Language::Spanish,
                _ => Language::English,
            };
            (captures, language)
        } else {
            return Ok(None);
        };
    let number = |index: usize| {
        captures
            .get(index)
            .and_then(|value| value.as_str().parse::<u16>().ok())
    };
    let date = match (number(1), number(2), number(3)) {
        (Some(year), Some(month), Some(day)) => u8::try_from(month)
            .ok()
            .and_then(|month| Month::try_from(month).ok())
            .zip(u8::try_from(day).ok())
            .and_then(|(month, day)| Date::from_calendar_date(i32::from(year), month, day).ok()),
        _ => None,
    };
    // A file name that looks like an article but carries an impossible date is
    // left out instead of being stored under a guessed day.
    Ok(date.map(|date| Unlisted {
        relative: relative.to_owned(),
        date,
        language,
    }))
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    #[test]
    fn reads_date_and_language_from_article_paths() -> Result<()> {
        let korean = classify("calendar/2025/06/06-20/2025-0620-001.html")?;
        assert_eq!(
            korean.map(|page| (page.date, page.language)),
            Some((date!(2025 - 06 - 20), Language::Korean))
        );
        let english = classify("item/2025/202510/news31/20251031-12ee.html")?;
        assert_eq!(
            english.map(|page| (page.date, page.language)),
            Some((date!(2025 - 10 - 31), Language::English))
        );
        let spanish = classify("item/2025/202510/news31/20251031-03SS.HTML")?;
        assert_eq!(spanish.map(|page| page.language), Some(Language::Spanish));
        Ok(())
    }

    #[test]
    fn ignores_index_pages_and_impossible_dates() -> Result<()> {
        for path in [
            "calendar/2025/06/06-20/2025-06-20.html",
            "calendar/2025/calendar-2025.html",
            "item/2025/202501/2025-01-01ee.html",
            "calendar/2025/02/02-30/2025-0230-001.html",
            "item2/2002/200211/news11/15.htm",
        ] {
            assert_eq!(classify(path)?, None);
        }
        Ok(())
    }

    #[test]
    fn a_missing_inventory_lists_nothing() -> Result<()> {
        assert!(listed_paths(Path::new("/nonexistent/kcna_jp.jsonl"))?.is_empty());
        let found = find_unlisted(
            Path::new("/nonexistent/root"),
            Path::new("/nonexistent/seed"),
        )?;
        assert!(found.is_empty());
        Ok(())
    }
}
