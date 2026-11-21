//! Response contract of the single-document endpoint.

use serde_derive::Serialize;
use utoipa::ToSchema;

use super::dto::{ArchiveLanguage, ArchivePrecision, ArchiveSource, ArchiveTextSegment};

/// The same article in another language.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveTranslationLink {
    pub id: String,
    pub language: ArchiveLanguage,
    pub title: String,
}

/// The same article as published by another source.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveCopyLink {
    pub id: String,
    pub source: ArchiveSource,
    pub canonical_url: String,
}

/// One document with the query's matches marked in its title and body.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDocumentResponse {
    pub id: String,
    pub title: Vec<ArchiveTextSegment>,
    /// Body paragraphs; empty when the record has no body.
    pub paragraphs: Vec<Vec<ArchiveTextSegment>>,
    /// `YYYY-MM-DD`.
    pub publication_date: String,
    pub publication_precision: ArchivePrecision,
    pub language: ArchiveLanguage,
    /// The source of the shown copy.
    pub source: ArchiveSource,
    /// Every source holding the article, the shown copy's first.
    pub sources: Vec<ArchiveSource>,
    pub canonical_url: String,
    pub category: Option<String>,
    pub source_record_id: Option<String>,
    /// Source-specific record metadata; `null` when none was stored.
    #[schema(value_type = serde_json::Value)]
    pub metadata: serde_json::Value,
    pub translations: Vec<ArchiveTranslationLink>,
    pub copies: Vec<ArchiveCopyLink>,
}
