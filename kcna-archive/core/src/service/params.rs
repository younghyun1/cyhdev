//! Query-string parameters shared by the search endpoints, and their validation.

use serde::Deserialize;
use time::{Date, macros::format_description};

use crate::{
    search::request::{Bucket, Filters, Sort},
    search_text,
    service::archive::ServiceError,
    types::{Language, Source},
};

/// Longest accepted query text, in characters.
const MAX_QUERY_CHARS: usize = 200;
const MAX_CATEGORY_CHARS: usize = 200;
const MAX_PAGE_SIZE: usize = 100;
const DEFAULT_PAGE_SIZE: usize = 50;
/// Deepest result position a page request may reach. Offset paging costs grow
/// with depth, and narrowing the query is the way to see further results.
const MAX_RESULT_WINDOW: usize = 10_000;
/// A day histogram over more days than this would be thousands of bars.
const MAX_DAY_BUCKET_SPAN: i64 = 366;
/// Ranges up to this many days default to day buckets.
const AUTO_DAY_SPAN: i64 = 92;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct QueryParams {
    #[serde(default)]
    pub q: String,
    /// Comma-separated ISO 639-3 codes.
    pub lang: Option<String>,
    /// Comma-separated source identifiers.
    pub source: Option<String>,
    pub category: Option<String>,
    /// Inclusive bounds as `YYYY-MM-DD`.
    pub from: Option<String>,
    pub to: Option<String>,
    pub sort: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub bucket: Option<String>,
}

impl QueryParams {
    /// Validates the filter parameters into a typed request.
    pub fn filters(&self) -> Result<Filters, ServiceError> {
        if self.q.chars().count() > MAX_QUERY_CHARS {
            return Err(ServiceError::BadRequest);
        }
        let category = match self.category.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(name) if name.chars().count() <= MAX_CATEGORY_CHARS => Some(name.to_owned()),
            Some(_) => return Err(ServiceError::BadRequest),
        };
        let from = parse_date(self.from.as_deref())?;
        let to = parse_date(self.to.as_deref())?;
        if let (Some(from), Some(to)) = (from, to)
            && from > to
        {
            return Err(ServiceError::BadRequest);
        }
        Ok(Filters {
            clauses: search_text::query_clauses(&self.q),
            languages: parse_list(self.lang.as_deref(), Language::from_code)?,
            sources: parse_list(self.source.as_deref(), Source::from_code)?,
            category,
            from,
            to,
        })
    }

    /// True when the text holds characters but nothing searchable, such as
    /// punctuation alone. Such a query matches nothing instead of everything.
    pub fn is_unsearchable(&self, filters: &Filters) -> bool {
        !self.q.trim().is_empty() && filters.clauses.is_empty()
    }

    pub fn sort(&self) -> Result<Sort, ServiceError> {
        match self.sort.as_deref() {
            // The archive is read by date, so that is the order when none is asked for.
            None | Some("" | "newest") => Ok(Sort::Newest),
            Some("relevance") => Ok(Sort::Relevance),
            Some("oldest") => Ok(Sort::Oldest),
            Some(_) => Err(ServiceError::BadRequest),
        }
    }

    /// Returns the validated `(offset, limit)` of a result page.
    pub fn window(&self) -> Result<(usize, usize), ServiceError> {
        let offset = self.offset.unwrap_or(0);
        let limit = self.limit.unwrap_or(DEFAULT_PAGE_SIZE);
        if limit == 0 || limit > MAX_PAGE_SIZE || offset.saturating_add(limit) > MAX_RESULT_WINDOW {
            return Err(ServiceError::BadRequest);
        }
        Ok((offset, limit))
    }

    /// The bucket size of a trends request: years unless months are asked for.
    pub fn trend_bucket(&self) -> Result<Bucket, ServiceError> {
        match self.bucket.as_deref() {
            None | Some("" | "year") => Ok(Bucket::Year),
            Some("month") => Ok(Bucket::Month),
            Some(_) => Err(ServiceError::BadRequest),
        }
    }

    /// The requested bucket size, or one fitting the date range.
    pub fn bucket(&self, filters: &Filters) -> Result<Bucket, ServiceError> {
        let span = match (filters.from, filters.to) {
            (Some(from), Some(to)) => Some((to - from).whole_days()),
            _ => None,
        };
        match self.bucket.as_deref() {
            Some("year") => Ok(Bucket::Year),
            Some("month") => Ok(Bucket::Month),
            Some("day") if span.is_some_and(|days| days <= MAX_DAY_BUCKET_SPAN) => Ok(Bucket::Day),
            Some("day") => Err(ServiceError::BadRequest),
            None | Some("") => Ok(match span {
                Some(days) if days <= AUTO_DAY_SPAN => Bucket::Day,
                _ => Bucket::Month,
            }),
            Some(_) => Err(ServiceError::BadRequest),
        }
    }
}

fn parse_date(value: Option<&str>) -> Result<Option<Date>, ServiceError> {
    match value.map(str::trim) {
        None | Some("") => Ok(None),
        Some(text) => Date::parse(text, format_description!("[year]-[month]-[day]"))
            .ok()
            .filter(|date| (1990..=2100).contains(&date.year()))
            .map(Some)
            .ok_or(ServiceError::BadRequest),
    }
}

fn parse_list<T>(
    value: Option<&str>,
    parse: impl Fn(&str) -> Option<T>,
) -> Result<Vec<T>, ServiceError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| parse(part).ok_or(ServiceError::BadRequest))
        .collect()
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    fn params(pairs: &[(&str, &str)]) -> QueryParams {
        let mut params = QueryParams::default();
        for (key, value) in pairs {
            let text = (*value).to_owned();
            match *key {
                "q" => params.q = text,
                "lang" => params.lang = Some(text),
                "source" => params.source = Some(text),
                "category" => params.category = Some(text),
                "from" => params.from = Some(text),
                "to" => params.to = Some(text),
                "sort" => params.sort = Some(text),
                "bucket" => params.bucket = Some(text),
                _ => {}
            }
        }
        params
    }

    #[test]
    fn parses_filters_into_typed_values() {
        let filters = params(&[
            ("q", "김정은 peace"),
            ("lang", "kor, eng"),
            ("source", "kcna_kp"),
            ("category", " 국내소식 "),
            ("from", "2026-09-01"),
            ("to", "2026-11-21"),
        ])
        .filters();
        assert!(filters.is_ok_and(|filters| {
            filters.clauses.len() == 2
                && filters.languages == [Language::Korean, Language::English]
                && filters.sources == [Source::KcnaKp]
                && filters.category.as_deref() == Some("국내소식")
                && filters.from == Some(date!(2026 - 09 - 01))
                && filters.to == Some(date!(2026 - 11 - 21))
        }));
    }

    #[test]
    fn sorts_newest_first_unless_asked_otherwise() {
        assert!(matches!(params(&[]).sort(), Ok(Sort::Newest)));
        assert!(matches!(params(&[("q", "평화")]).sort(), Ok(Sort::Newest)));
        assert!(matches!(
            params(&[("sort", "relevance")]).sort(),
            Ok(Sort::Relevance)
        ));
        assert!(matches!(
            params(&[("sort", "oldest")]).sort(),
            Ok(Sort::Oldest)
        ));
    }

    #[test]
    fn rejects_malformed_filters() {
        for bad in [
            ("lang", "kor,xx"),
            ("source", "elsewhere"),
            ("from", "2026-9-1"),
            ("from", "1800-01-01"),
            ("sort", "random"),
        ] {
            let request = params(&[bad]);
            assert!(
                request.filters().is_err() || request.sort().is_err(),
                "{bad:?}"
            );
        }
        assert!(
            params(&[("from", "2026-11-02"), ("to", "2026-11-01")])
                .filters()
                .is_err()
        );
        assert!(
            params(&[("q", &"가".repeat(MAX_QUERY_CHARS + 1))])
                .filters()
                .is_err()
        );
    }

    #[test]
    fn detects_text_with_nothing_searchable() {
        let punctuation = params(&[("q", " !?, ")]);
        assert!(
            punctuation
                .filters()
                .is_ok_and(|filters| punctuation.is_unsearchable(&filters))
        );
        let empty = params(&[]);
        assert!(
            empty
                .filters()
                .is_ok_and(|filters| !empty.is_unsearchable(&filters))
        );
    }

    #[test]
    fn bounds_the_result_window() {
        let mut request = QueryParams::default();
        assert_eq!(request.window().ok(), Some((0, DEFAULT_PAGE_SIZE)));
        request.limit = Some(MAX_PAGE_SIZE + 1);
        assert!(request.window().is_err());
        request.limit = Some(0);
        assert!(request.window().is_err());
        request.limit = Some(50);
        request.offset = Some(MAX_RESULT_WINDOW);
        assert!(request.window().is_err());
        request.offset = Some(MAX_RESULT_WINDOW - 50);
        assert!(request.window().is_ok());
    }

    #[test]
    fn chooses_a_bucket_that_fits_the_range() {
        let range = |from: &str, to: &str| params(&[("from", from), ("to", to)]);
        let bucket = |request: &QueryParams| {
            request
                .filters()
                .and_then(|filters| request.bucket(&filters))
                .ok()
        };
        assert_eq!(bucket(&params(&[])), Some(Bucket::Month));
        assert_eq!(
            bucket(&range("2026-09-01", "2026-11-21")),
            Some(Bucket::Day)
        );
        assert_eq!(
            bucket(&range("2020-01-01", "2026-11-21")),
            Some(Bucket::Month)
        );
        let mut wide = range("2020-01-01", "2026-11-21");
        wide.bucket = Some("day".to_owned());
        assert_eq!(bucket(&wide), None);
        wide.bucket = Some("year".to_owned());
        assert_eq!(bucket(&wide), Some(Bucket::Year));
    }
}
