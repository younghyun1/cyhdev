//! Validated document and retrieval values at the database boundary.

use nutype::nutype;
use serde_json::Value;
use sha2::{Digest, Sha256};
use time::{Date, OffsetDateTime};
use url::Url;

use crate::database::types::{Language, PublicationPrecision, Source};

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 8192, predicate = is_http_url),
    derive(Clone, Debug, AsRef, Display)
)]
pub struct CanonicalUrl(String);

#[nutype(sanitize(trim), validate(not_empty), derive(Clone, Debug, AsRef))]
pub struct DocumentTitle(String);

#[nutype(sanitize(trim), validate(not_empty), derive(Clone, Debug, AsRef))]
pub struct DocumentBody(String);

#[nutype(
    validate(greater_or_equal = 0),
    derive(Clone, Copy, Debug, Deref, Display)
)]
pub struct ElementIndex(i16);

#[nutype(
    validate(predicate = |content| !content.is_empty()),
    derive(Clone, Debug, AsRef)
)]
pub struct RawContent(Vec<u8>);

/// A complete normalized record plus its immutable source snapshot.
#[derive(Clone, Debug)]
pub struct DocumentBundle {
    pub source: Source,
    pub source_record_id: Option<String>,
    pub canonical_url: CanonicalUrl,
    pub element_index: ElementIndex,
    pub language: Language,
    pub translation_group_key: Option<String>,
    pub title: DocumentTitle,
    pub body: Option<DocumentBody>,
    pub published_at: OffsetDateTime,
    pub publication_precision: PublicationPrecision,
    pub publication_date: Date,
    pub published_at_raw: Option<String>,
    pub category: Option<String>,
    pub content_sha256: [u8; 32],
    pub copyright_status: Option<String>,
    pub source_metadata: Value,
    pub retrieved_at: OffsetDateTime,
    pub requested_url: CanonicalUrl,
    pub final_url: Option<CanonicalUrl>,
    pub http_status: Option<i16>,
    pub media_type: Option<String>,
    pub charset: Option<String>,
    pub raw_content: RawContent,
    pub raw_sha256: [u8; 32],
    pub response_headers: Value,
    pub manifest_metadata: Value,
}

/// Version of the stored document format. Bumping it changes every
/// fingerprint, so the next import rewrites every row; do that whenever
/// extraction or the stored fields change meaning.
const IMPORT_FORMAT_VERSION: u8 = 2;

impl DocumentBundle {
    /// Digest of everything stored for this document and its retrieval.
    ///
    /// A re-import compares it with the stored value and skips the row when
    /// nothing changed, instead of rewriting the whole corpus every run.
    pub fn fingerprint(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        // Length-prefixed so adjacent fields cannot run into each other.
        let mut field = |bytes: &[u8]| {
            digest.update((bytes.len() as u64).to_be_bytes());
            digest.update(bytes);
        };
        let text = |value: &Option<String>| value.as_deref().unwrap_or("\u{0}").as_bytes().to_vec();
        field(&[IMPORT_FORMAT_VERSION]);
        field(self.source.code().as_bytes());
        field(self.language.code().as_bytes());
        field(self.canonical_url.as_ref().as_bytes());
        field(&self.element_index.to_be_bytes());
        field(&text(&self.source_record_id));
        field(&text(&self.translation_group_key));
        field(&text(&self.category));
        field(&text(&self.published_at_raw));
        field(&text(&self.copyright_status));
        field(&self.content_sha256);
        field(&self.published_at.unix_timestamp_nanos().to_be_bytes());
        field(&[match self.publication_precision {
            PublicationPrecision::Second => 0,
            PublicationPrecision::Minute => 1,
            PublicationPrecision::Day => 2,
            PublicationPrecision::InferredDay => 3,
        }]);
        field(self.source_metadata.to_string().as_bytes());
        field(&self.raw_sha256);
        field(&self.retrieved_at.unix_timestamp_nanos().to_be_bytes());
        field(self.requested_url.as_ref().as_bytes());
        digest.finalize().into()
    }
}

fn is_http_url(value: &str) -> bool {
    match Url::parse(value) {
        Ok(url) => matches!(url.scheme(), "http" | "https") && url.host_str().is_some(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_url_rejects_non_http_and_whitespace() {
        assert!(CanonicalUrl::try_new("file:///tmp/a").is_err());
        assert!(CanonicalUrl::try_new("not a URL").is_err());
        assert!(CanonicalUrl::try_new(" https://kcna.kp/kp ").is_ok());
    }

    #[test]
    fn raw_content_rejects_empty_responses() {
        assert!(RawContent::try_new(Vec::new()).is_err());
        assert!(RawContent::try_new(vec![1]).is_ok());
    }
}
