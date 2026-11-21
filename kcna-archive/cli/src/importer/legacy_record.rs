//! Record assembly for the historical importer: one stored document per article.

use std::path::Path;

use serde::Deserialize;
use serde_json::Value;
use time::{
    Date,
    macros::{format_description, offset, time},
};
use tracing::warn;

use crate::{
    database::{
        domain::{DocumentBody, DocumentBundle, DocumentTitle, ElementIndex, RawContent},
        types::{Language, PublicationPrecision, Source},
    },
    error::{Error, Result},
    importer::{
        common::{content_sha256, validated_url},
        extract::ArticleText,
        legacy_page::{Page, load_page},
        provenance::ProvenanceIndex,
    },
};

/// Title stored for a record whose page yields none.
pub const UNAVAILABLE_TITLE: &str = "[Title unavailable in source]";

#[derive(Debug, Deserialize)]
pub struct SeedRecord {
    pub id: String,
    pub url: String,
    language: String,
    copyright_status: Option<String>,
    metadata: SeedMetadata,
}

#[derive(Debug, Deserialize)]
struct SeedMetadata {
    date: String,
    #[serde(default)]
    elem_idx: usize,
}

/// A seed record with its complete source JSON.
pub struct PendingRecord {
    pub record: SeedRecord,
    pub raw: Value,
}

/// What is known about one article apart from the page text itself.
pub struct Descriptor {
    pub source_record_id: Option<String>,
    pub language: Language,
    pub element_index: usize,
    pub publication_date: Date,
    pub publication_precision: PublicationPrecision,
    pub published_at_raw: Option<String>,
    pub copyright_status: Option<String>,
    /// Source metadata stored with the document.
    pub metadata: Value,
}

/// Builds every record of one page. Returns the bundles and the failure count.
///
/// Runs on the blocking pool: decoding, HTML parsing, hashing, and search
/// analysis are CPU-bound, and the file reads here are synchronous.
pub fn build_group(
    root: &Path,
    provenance: &ProvenanceIndex,
    records: Vec<PendingRecord>,
) -> (Vec<DocumentBundle>, usize) {
    let Some(first) = records.first() else {
        return (Vec::new(), 0);
    };
    let page = match load_page(root, provenance, &first.record.url) {
        Ok(page) => page,
        Err(error) => {
            warn!(
                url = first.record.url,
                records = records.len(),
                error = %error,
                "Historical page skipped"
            );
            return (Vec::new(), records.len());
        }
    };
    let mut bundles = Vec::with_capacity(records.len());
    let mut failed = 0_usize;
    for pending in records {
        let built = descriptor(&pending.record, pending.raw)
            .and_then(|descriptor| build_bundle(&page, descriptor));
        match built {
            Ok(bundle) => bundles.push(bundle),
            Err(error) => {
                failed += 1;
                warn!(record_id = pending.record.id, error = %error, "Historical record skipped");
            }
        }
    }
    (bundles, failed)
}

fn descriptor(record: &SeedRecord, raw_metadata: Value) -> Result<Descriptor> {
    Ok(Descriptor {
        source_record_id: Some(record.id.clone()),
        language: language(&record.language)?,
        element_index: record.metadata.elem_idx,
        publication_date: Date::parse(
            &record.metadata.date,
            format_description!("[year]-[month]-[day]"),
        )?,
        publication_precision: PublicationPrecision::Day,
        published_at_raw: Some(record.metadata.date.clone()),
        copyright_status: record.copyright_status.clone(),
        metadata: raw_metadata,
    })
}

/// Assembles one stored document from a parsed page and its descriptor.
pub fn build_bundle(page: &Page, descriptor: Descriptor) -> Result<DocumentBundle> {
    let mut article = page
        .articles
        .get(descriptor.element_index)
        .map_or_else(ArticleText::default, Clone::clone);
    // The database re-anchors day-precision records to noon in Asia/Pyongyang.
    let published_at = descriptor
        .publication_date
        .with_time(time!(12:00))
        .assume_offset(offset!(+9));
    let element_index = i16::try_from(descriptor.element_index)
        .map_err(|_| Error::InvalidRecord("element index exceeds smallint".to_owned()))?;
    let mut source_metadata = descriptor.metadata;
    let title_unavailable = article.title.is_empty();
    let body_unavailable = article.body.is_empty();
    if title_unavailable {
        article.title = UNAVAILABLE_TITLE.to_owned();
    }
    if title_unavailable || body_unavailable {
        annotate_unavailable_content(&mut source_metadata, title_unavailable, body_unavailable);
    }
    let body = (!body_unavailable).then_some(article.body.as_str());
    Ok(DocumentBundle {
        source: Source::KcnaCoJp,
        source_record_id: descriptor.source_record_id,
        canonical_url: validated_url(page.url.as_str())?,
        element_index: ElementIndex::try_new(element_index)
            .map_err(|error| Error::InvalidRecord(error.to_string()))?,
        language: descriptor.language,
        translation_group_key: None,
        content_sha256: content_sha256(&article.title, Some(&article.body)),
        title: DocumentTitle::try_new(article.title.clone())
            .map_err(|error| Error::InvalidRecord(error.to_string()))?,
        body: match body {
            Some(body) => Some(
                DocumentBody::try_new(body)
                    .map_err(|error| Error::InvalidRecord(error.to_string()))?,
            ),
            None => None,
        },
        published_at,
        publication_precision: descriptor.publication_precision,
        publication_date: descriptor.publication_date,
        published_at_raw: descriptor.published_at_raw,
        category: None,
        copyright_status: descriptor.copyright_status,
        source_metadata,
        retrieved_at: page.retrieval.retrieved_at,
        requested_url: validated_url(&page.retrieval.url)?,
        final_url: None,
        http_status: Some(200),
        media_type: Some("text/html".to_owned()),
        charset: Some(page.charset.clone()),
        raw_content: RawContent::try_new(page.bytes.clone())
            .map_err(|error| Error::InvalidRecord(error.to_string()))?,
        raw_sha256: page.raw_sha256,
        response_headers: serde_json::json!({}),
        manifest_metadata: page.retrieval.raw.clone(),
    })
}

/// Merges importer notes into the `_kcna_archive` object of the metadata.
fn annotate_unavailable_content(metadata: &mut Value, title: bool, body: bool) {
    let Some(object) = metadata.as_object_mut() else {
        return;
    };
    let notes = object
        .entry("_kcna_archive")
        .or_insert_with(|| serde_json::json!({}));
    if let Some(notes) = notes.as_object_mut() {
        notes.insert("title_unavailable_in_source".to_owned(), title.into());
        notes.insert("body_unavailable_in_source".to_owned(), body.into());
    }
}

fn language(value: &str) -> Result<Language> {
    match value {
        "North Korean" => Ok(Language::Korean),
        "English" => Ok(Language::English),
        "Spanish" => Ok(Language::Spanish),
        other => Err(Error::InvalidRecord(format!(
            "unsupported language {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_seed_language_names() {
        assert!(matches!(language("North Korean"), Ok(Language::Korean)));
        assert!(language("Klingon").is_err());
    }
}
