//! Term frequencies over time.

use std::collections::BTreeMap;

use time::Date;

use crate::{
    search::{
        engine::SearchEngine,
        request::{Bucket, Filters},
    },
    search_text,
    service::{
        archive::ServiceError,
        models::{TrendSeries, TrendsResponse},
        params::QueryParams,
    },
};

/// Terms compared in one trends request.
pub const MAX_TREND_TERMS: usize = 4;
const MAX_TERM_CHARS: usize = 100;

/// Counts documents matching each term per time bucket, with the totals
/// needed to express them as a share.
pub fn trends(
    engine: &SearchEngine,
    params: &QueryParams,
    terms: Vec<String>,
) -> Result<TrendsResponse, ServiceError> {
    let terms = terms
        .into_iter()
        .map(|term| term.trim().to_owned())
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();
    if terms.is_empty()
        || terms.len() > MAX_TREND_TERMS
        || terms
            .iter()
            .any(|term| term.chars().count() > MAX_TERM_CHARS)
    {
        return Err(ServiceError::BadRequest);
    }
    let mut base = params.filters()?;
    base.clauses.clear();
    let bucket = params.trend_bucket()?;
    let range = |date: Date| {
        base.from
            .is_none_or(|from| date >= bucket_start(bucket, from))
            && base.to.is_none_or(|to| date <= to)
    };
    let totals = engine
        .histogram(&base, bucket)?
        .into_iter()
        .filter(|(start, _)| range(*start))
        .collect::<BTreeMap<_, _>>();
    let starts = dense(
        bucket,
        totals.keys().next().copied(),
        totals.keys().next_back().copied(),
    );
    let mut series = Vec::with_capacity(terms.len());
    for term in terms {
        let filters = Filters {
            clauses: search_text::query_clauses(&term),
            ..base.clone()
        };
        // A term with nothing searchable matches nothing, not everything.
        let counts = if filters.clauses.is_empty() {
            BTreeMap::new()
        } else {
            engine.histogram(&filters, bucket)?.into_iter().collect()
        };
        series.push(TrendSeries {
            counts: starts
                .iter()
                .map(|start| counts.get(start).copied().unwrap_or(0))
                .collect(),
            term,
        });
    }
    Ok(TrendsResponse {
        bucket: bucket.name(),
        totals: starts
            .iter()
            .map(|start| totals.get(start).copied().unwrap_or(0))
            .collect(),
        starts: starts.iter().map(Date::to_string).collect(),
        series,
    })
}

fn bucket_start(bucket: Bucket, date: Date) -> Date {
    bucket.start(bucket.key(date)).unwrap_or(date)
}

/// Every bucket start from `first` to `last`, so charts have no gaps.
fn dense(bucket: Bucket, first: Option<Date>, last: Option<Date>) -> Vec<Date> {
    let (Some(first), Some(last)) = (first, last) else {
        return Vec::new();
    };
    (bucket.key(first)..=bucket.key(last))
        .filter_map(|key| bucket.start(key))
        .collect()
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    #[test]
    fn dense_buckets_fill_gaps() {
        let months = dense(
            Bucket::Month,
            Some(date!(2025 - 11 - 01)),
            Some(date!(2026 - 02 - 01)),
        );
        assert_eq!(
            months,
            [
                date!(2025 - 11 - 01),
                date!(2025 - 12 - 01),
                date!(2026 - 01 - 01),
                date!(2026 - 02 - 01)
            ]
        );
        assert!(dense(Bucket::Year, None, None).is_empty());
        assert_eq!(
            bucket_start(Bucket::Year, date!(2026 - 11 - 21)),
            date!(2026 - 01 - 01)
        );
    }
}
