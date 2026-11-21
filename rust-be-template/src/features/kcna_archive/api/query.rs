//! Query-string contracts of the archive endpoints.

use kcna_archive_core::service::params::QueryParams;
use serde_derive::Deserialize;
use utoipa::IntoParams;

/// Filters and paging shared by the search, histogram, trends, and export
/// endpoints. Unused parameters are ignored; the archive validates the rest and
/// answers 400 for malformed values.
#[derive(Clone, Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ArchiveQuery {
    /// Search text, at most 200 characters.
    pub q: Option<String>,
    /// Comma-separated ISO 639-3 codes: kor, eng, zho, rus, spa, jpn.
    pub lang: Option<String>,
    /// Comma-separated sources: kcna_kp, kcna_co_jp, stiles_archive.
    pub source: Option<String>,
    pub category: Option<String>,
    /// Inclusive lower bound, `YYYY-MM-DD`.
    pub from: Option<String>,
    /// Inclusive upper bound, `YYYY-MM-DD`.
    pub to: Option<String>,
    /// `newest` (default), `oldest`, or `relevance`. Search and export only.
    pub sort: Option<String>,
    /// Result offset; offset plus limit may not exceed 10000. Search only.
    pub offset: Option<usize>,
    /// Page size, 1 to 100 (default 50). Search only.
    pub limit: Option<usize>,
    /// `year`, `month`, or `day` for the histogram; `year` or `month` for trends.
    pub bucket: Option<String>,
}

impl From<ArchiveQuery> for QueryParams {
    fn from(query: ArchiveQuery) -> Self {
        Self {
            q: query.q.unwrap_or_default(),
            lang: query.lang,
            source: query.source,
            category: query.category,
            from: query.from,
            to: query.to,
            sort: query.sort,
            offset: query.offset,
            limit: query.limit,
            bucket: query.bucket,
        }
    }
}

/// The repeated `term` parameter of the trends endpoint.
///
/// Documentation only: a typed extractor cannot read a repeated key, so the
/// handler collects the pairs itself.
#[derive(IntoParams)]
#[into_params(parameter_in = Query)]
#[allow(
    dead_code,
    reason = "describes the repeated term parameter for OpenAPI only"
)]
pub struct TrendTermsQuery {
    /// One to four terms to compare, each at most 100 characters; repeat the key.
    #[param(style = Form, explode, max_items = 4)]
    pub term: Vec<String>,
}

/// The highlight query of the document endpoint.
#[derive(Clone, Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DocumentQuery {
    /// Search text whose matches are marked in the title and body.
    pub q: Option<String>,
}

/// The format parameter of the export endpoint.
#[derive(Clone, Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ExportQuery {
    /// `csv` (default) or `jsonl`.
    pub format: Option<String>,
}

/// Collects the repeated `term` values, reading one past the allowed number so
/// an oversized request is rejected by the archive instead of truncated.
pub fn trend_terms(pairs: Vec<(String, String)>, max_terms: usize) -> Vec<String> {
    pairs
        .into_iter()
        .filter(|(key, _)| key == "term")
        .map(|(_, value)| value)
        .take(max_terms.saturating_add(1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(key: &str, value: &str) -> (String, String) {
        (key.to_owned(), value.to_owned())
    }

    #[test]
    fn terms_are_collected_in_order_and_capped_one_past_the_limit() {
        let pairs = vec![
            pair("q", "ignored"),
            pair("term", "a"),
            pair("term", "b"),
            pair("bucket", "year"),
            pair("term", "c"),
            pair("term", "d"),
            pair("term", "e"),
            pair("term", "f"),
        ];
        assert_eq!(trend_terms(pairs, 4), ["a", "b", "c", "d", "e"]);
        assert!(trend_terms(vec![pair("q", "x")], 4).is_empty());
    }

    #[test]
    fn a_query_converts_to_core_parameters() {
        let params = QueryParams::from(ArchiveQuery {
            q: Some("peace".to_owned()),
            lang: Some("kor,eng".to_owned()),
            limit: Some(20),
            ..ArchiveQuery::default()
        });
        assert_eq!(params.q, "peace");
        assert_eq!(params.lang.as_deref(), Some("kor,eng"));
        assert_eq!(params.limit, Some(20));
        assert!(QueryParams::from(ArchiveQuery::default()).q.is_empty());
    }
}
