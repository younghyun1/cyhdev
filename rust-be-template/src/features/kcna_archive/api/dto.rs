//! Response contracts shared by the archive endpoints.
//!
//! These types are the browser contract. They mirror the core crate's result
//! models field for field but are distinct types, converted in `convert.rs`.

use serde_derive::Serialize;
use utoipa::ToSchema;

/// ISO 639-3 code of an article's language.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveLanguage {
    Kor,
    Eng,
    Zho,
    Rus,
    Spa,
    Jpn,
}

/// Publisher of an article copy.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveSource {
    KcnaKp,
    KcnaCoJp,
    StilesArchive,
}

/// How exactly a publication time is known.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArchivePrecision {
    Second,
    Minute,
    Day,
    InferredDay,
}

/// Size of a time bucket.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveBucket {
    Year,
    Month,
    Day,
}

/// A run of text, flagged when it matched the query.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveTextSegment {
    pub text: String,
    pub hit: bool,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLanguageFacet {
    pub value: ArchiveLanguage,
    pub count: u64,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSourceFacet {
    pub value: ArchiveSource,
    pub count: u64,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveCategoryFacet {
    pub value: String,
    pub count: u64,
}

/// Match counts per facet value under the current filters.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveFacets {
    pub languages: Vec<ArchiveLanguageFacet>,
    pub sources: Vec<ArchiveSourceFacet>,
    pub categories: Vec<ArchiveCategoryFacet>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSearchItem {
    pub id: String,
    pub title: Vec<ArchiveTextSegment>,
    pub snippet: Vec<ArchiveTextSegment>,
    /// `YYYY-MM-DD`.
    pub publication_date: String,
    pub language: ArchiveLanguage,
    /// Every source holding the article, the shown copy's first.
    pub sources: Vec<ArchiveSource>,
    pub category: Option<String>,
    pub has_full_text: bool,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSearchResponse {
    /// Matches across all pages.
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub items: Vec<ArchiveSearchItem>,
    pub facets: ArchiveFacets,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveHistogramBucket {
    /// First day of the bucket, `YYYY-MM-DD`.
    pub start: String,
    pub count: u64,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveHistogramResponse {
    pub bucket: ArchiveBucket,
    pub buckets: Vec<ArchiveHistogramBucket>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveTrendSeries {
    pub term: String,
    /// Documents per bucket that match the term, parallel to `starts`.
    pub counts: Vec<u64>,
}

/// Term frequencies over time, as parallel arrays over `starts`.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveTrendsResponse {
    pub bucket: ArchiveBucket,
    /// First day of each bucket, oldest first, with no gaps.
    pub starts: Vec<String>,
    /// Documents per bucket under the filters, regardless of text.
    pub totals: Vec<u64>,
    pub series: Vec<ArchiveTrendSeries>,
}
