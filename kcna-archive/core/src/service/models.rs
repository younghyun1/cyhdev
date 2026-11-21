//! Results of the read-only archive service.
//!
//! These are the service's own values. A host that exposes them over HTTP
//! maps them to its transport types at its boundary.

use serde::Serialize;
use serde_json::Value;

use crate::{
    search::{highlight::Segment, request::Bucket},
    types::{Language, PublicationPrecision, Source},
};

/// A run of text, flagged when it matched the query.
#[derive(Clone, Debug, Serialize)]
pub struct TextSegment {
    pub text: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub hit: bool,
}

impl From<Segment> for TextSegment {
    fn from(segment: Segment) -> Self {
        Self {
            text: segment.text,
            hit: segment.hit,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchItem {
    pub id: String,
    pub title: Vec<TextSegment>,
    pub snippet: Vec<TextSegment>,
    pub publication_date: String,
    pub language: Language,
    /// Every source holding the article, the shown copy's first.
    pub sources: Vec<Source>,
    pub category: Option<String>,
    pub has_full_text: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct FacetValue<T> {
    pub value: T,
    pub count: u64,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct FacetGroups {
    pub languages: Vec<FacetValue<Language>>,
    pub sources: Vec<FacetValue<Source>>,
    pub categories: Vec<FacetValue<String>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchResponse {
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub items: Vec<SearchItem>,
    pub facets: FacetGroups,
}

#[derive(Clone, Debug, Serialize)]
pub struct HistogramBucket {
    /// First day of the bucket, `YYYY-MM-DD`.
    pub start: String,
    pub count: u64,
}

impl Bucket {
    /// Name of the bucket size in requests and responses.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Year => "year",
            Self::Month => "month",
            Self::Day => "day",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct HistogramResponse {
    pub bucket: &'static str,
    pub buckets: Vec<HistogramBucket>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentResponse {
    pub id: String,
    pub title: Vec<TextSegment>,
    /// Body paragraphs; empty when the record has no body.
    pub paragraphs: Vec<Vec<TextSegment>>,
    pub publication_date: String,
    pub publication_precision: PublicationPrecision,
    pub language: Language,
    /// The source of the shown copy.
    pub source: Source,
    /// Every source holding the article, the shown copy's first.
    pub sources: Vec<Source>,
    pub canonical_url: String,
    pub category: Option<String>,
    pub source_record_id: Option<String>,
    pub metadata: Value,
    /// The same article in other languages.
    pub translations: Vec<TranslationLink>,
    /// The same article as published by other sources.
    pub copies: Vec<CopyLink>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TranslationLink {
    pub id: String,
    pub language: Language,
    pub title: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyLink {
    pub id: String,
    pub source: Source,
    pub canonical_url: String,
}

/// Term frequencies over time, as parallel arrays over `starts`.
#[derive(Clone, Debug, Serialize)]
pub struct TrendsResponse {
    pub bucket: &'static str,
    /// First day of each bucket, oldest first, with no gaps.
    pub starts: Vec<String>,
    /// Documents per bucket under the filters, regardless of text.
    pub totals: Vec<u64>,
    pub series: Vec<TrendSeries>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrendSeries {
    pub term: String,
    /// Documents per bucket that match the term.
    pub counts: Vec<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    /// Documents stored in the collector's database when the snapshot was written.
    pub documents: u64,
    /// Of those, duplicates folded into another source's copy.
    pub copies: u64,
    /// Articles in the search index.
    pub indexed: u64,
    pub translation_groups: u64,
    pub oldest_date: Option<String>,
    pub newest_date: Option<String>,
    /// RFC 3339 time of the newest stored retrieval.
    pub last_retrieved_at: Option<String>,
    /// RFC 3339 time the snapshot was written.
    pub snapshot_built_at: String,
    /// Bytes of index held in memory; absent when the index is read from disk.
    pub memory_bytes: Option<u64>,
    pub languages: Vec<FacetValue<Language>>,
    pub sources: Vec<FacetValue<Source>>,
}
