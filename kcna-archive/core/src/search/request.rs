//! Typed search requests and results, independent of HTTP and of Tantivy.

use time::Date;
use uuid::Uuid;

use crate::{
    search_text::Clause,
    types::{Language, PublicationPrecision, Source},
};

/// What to match. Every part narrows the result; an empty part matches all.
#[derive(Clone, Debug, Default)]
pub struct Filters {
    /// Analyzed query text; empty browses.
    pub clauses: Vec<Clause>,
    pub languages: Vec<Language>,
    pub sources: Vec<Source>,
    pub category: Option<String>,
    /// Inclusive publication date bounds.
    pub from: Option<Date>,
    pub to: Option<Date>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Sort {
    /// Best match first; newest first when there is no query text.
    #[default]
    Relevance,
    Newest,
    Oldest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Bucket {
    Year,
    Month,
    Day,
}

#[derive(Clone, Debug)]
pub struct Hit {
    pub id: Uuid,
    pub title: String,
    pub body: Option<String>,
    pub date: Date,
    pub language: Language,
    /// Every source that published the article, the preferred copy's first.
    pub sources: Vec<Source>,
    pub category: Option<String>,
}

/// One document with everything the reader shows.
#[derive(Clone, Debug)]
pub struct StoredDocument {
    pub id: Uuid,
    pub title: String,
    pub body: Option<String>,
    pub date: Date,
    pub precision: PublicationPrecision,
    pub language: Language,
    /// Every source that published the article, the preferred copy's first.
    pub sources: Vec<Source>,
    pub canonical_url: String,
    pub category: Option<String>,
    pub source_record_id: Option<String>,
    pub metadata: serde_json::Value,
    pub translation_group: Option<Uuid>,
    /// The same article as published by other sources.
    pub copies: Vec<CopyRef>,
}

/// A copy of an article held by another source. Its text is not indexed on
/// its own; the preferred document stands for it.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CopyRef {
    pub id: Uuid,
    pub source: Source,
    pub url: String,
}

/// The same article in another language.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslationRef {
    pub id: Uuid,
    pub language: Language,
    pub title: String,
}

#[derive(Clone, Debug, Default)]
pub struct Page {
    pub total: usize,
    pub hits: Vec<Hit>,
}

/// Counts per filter value. Each group ignores its own filter, so the other
/// values of a selected filter stay visible with the counts they would give.
#[derive(Clone, Debug, Default)]
pub struct Facets {
    pub languages: Vec<(Language, u64)>,
    pub sources: Vec<(Source, u64)>,
    pub categories: Vec<(String, u64)>,
}

/// Matching documents per time bucket, oldest first, empty buckets omitted.
pub type Histogram = Vec<(Date, u64)>;

impl Bucket {
    /// Sortable key of the bucket containing `date`.
    pub fn key(self, date: Date) -> i64 {
        match self {
            Self::Year => i64::from(date.year()),
            Self::Month => i64::from(date.year()) * 12 + i64::from(u8::from(date.month())) - 1,
            Self::Day => i64::from(date.to_julian_day()),
        }
    }

    /// First day of the bucket with this key.
    pub fn start(self, key: i64) -> Option<Date> {
        match self {
            Self::Year => i32::try_from(key)
                .ok()
                .and_then(|year| Date::from_calendar_date(year, time::Month::January, 1).ok()),
            Self::Month => {
                let year = i32::try_from(key.div_euclid(12)).ok()?;
                let month = u8::try_from(key.rem_euclid(12) + 1).ok()?;
                Date::from_calendar_date(year, time::Month::try_from(month).ok()?, 1).ok()
            }
            Self::Day => i32::try_from(key)
                .ok()
                .and_then(|day| Date::from_julian_day(day).ok()),
        }
    }
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    #[test]
    fn bucket_keys_round_trip_to_their_first_day() {
        let day = date!(2026 - 11 - 21);
        assert_eq!(
            Bucket::Year.start(Bucket::Year.key(day)),
            Some(date!(2026 - 01 - 01))
        );
        assert_eq!(
            Bucket::Month.start(Bucket::Month.key(day)),
            Some(date!(2026 - 11 - 01))
        );
        assert_eq!(Bucket::Day.start(Bucket::Day.key(day)), Some(day));
        assert_eq!(
            Bucket::Month.key(date!(2027 - 01 - 05)) - Bucket::Month.key(date!(2026 - 12 - 31)),
            1
        );
    }
}
