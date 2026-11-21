//! Decodes stored index entries into typed results.

use tantivy::{
    TantivyDocument,
    schema::{Field, Value as _},
};
use uuid::Uuid;

use crate::{
    error::{Error, Result},
    search::{
        fields::{
            Fields, date_from_value, language_from_code, precision_from_code, source_from_code,
        },
        request::{CopyRef, Hit, StoredDocument},
    },
    types::Source,
};

fn missing(field: &str) -> Error {
    Error::Search(format!("indexed document lacks {field}"))
}

fn text(document: &TantivyDocument, field: Field) -> Option<String> {
    document
        .get_first(field)
        .and_then(|value| value.as_str())
        .map(str::to_owned)
}

fn uuid(document: &TantivyDocument, field: Field) -> Option<Uuid> {
    document
        .get_first(field)
        .and_then(|value| value.as_bytes())
        .and_then(|bytes| Uuid::from_slice(bytes).ok())
}

fn sources(document: &TantivyDocument, fields: &Fields) -> Result<Vec<Source>> {
    let sources = document
        .get_all(fields.source)
        .filter_map(|value| value.as_u64())
        .filter_map(source_from_code)
        .collect::<Vec<_>>();
    if sources.is_empty() {
        return Err(missing("source"));
    }
    Ok(sources)
}

/// The fields a result row shows.
pub fn hit(document: &TantivyDocument, fields: &Fields) -> Result<Hit> {
    Ok(Hit {
        id: uuid(document, fields.id).ok_or_else(|| missing("id"))?,
        title: text(document, fields.title).ok_or_else(|| missing("title"))?,
        body: text(document, fields.body),
        date: document
            .get_first(fields.date)
            .and_then(|value| value.as_i64())
            .and_then(date_from_value)
            .ok_or_else(|| missing("date"))?,
        language: document
            .get_first(fields.language)
            .and_then(|value| value.as_u64())
            .and_then(language_from_code)
            .ok_or_else(|| missing("language"))?,
        sources: sources(document, fields)?,
        category: text(document, fields.category),
    })
}

/// Everything the reader shows for one document.
pub fn stored_document(document: &TantivyDocument, fields: &Fields) -> Result<StoredDocument> {
    let hit = hit(document, fields)?;
    let metadata = match text(document, fields.metadata) {
        Some(json) => serde_json::from_str(&json)
            .map_err(|error| Error::Search(format!("stored metadata is not JSON: {error}")))?,
        None => serde_json::Value::Null,
    };
    let copies: Vec<CopyRef> = match text(document, fields.copies) {
        Some(json) => serde_json::from_str(&json)
            .map_err(|error| Error::Search(format!("stored copies are not JSON: {error}")))?,
        None => Vec::new(),
    };
    Ok(StoredDocument {
        id: hit.id,
        title: hit.title,
        body: hit.body,
        date: hit.date,
        precision: document
            .get_first(fields.precision)
            .and_then(|value| value.as_u64())
            .and_then(precision_from_code)
            .ok_or_else(|| missing("precision"))?,
        language: hit.language,
        sources: hit.sources,
        canonical_url: text(document, fields.url).ok_or_else(|| missing("url"))?,
        category: hit.category,
        source_record_id: text(document, fields.record),
        metadata,
        translation_group: uuid(document, fields.group),
        copies,
    })
}
