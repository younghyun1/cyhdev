//! Converts corpus documents into index entries.

use tantivy::{TantivyDocument, Term};
use time::Date;
use uuid::Uuid;

use crate::{
    search::{
        fields::{Fields, date_value, language_code, precision_code, source_code},
        request::CopyRef,
    },
    types::{Language, PublicationPrecision, Source},
};

/// The indexed form of one article: its preferred copy plus what is needed to
/// show it, link its other language editions, and name every source that also
/// published it.
#[derive(Clone, Debug)]
pub struct IndexedDocument {
    pub id: Uuid,
    pub title: String,
    pub body: Option<String>,
    pub date: Date,
    pub precision: PublicationPrecision,
    pub language: Language,
    pub source: Source,
    pub canonical_url: String,
    pub category: Option<String>,
    pub source_record_id: Option<String>,
    pub metadata: serde_json::Value,
    pub translation_group: Option<Uuid>,
    /// Copies of this article held by other sources.
    pub copies: Vec<CopyRef>,
}

/// Converts a document into its indexed fields.
pub fn document(fields: Fields, source: &IndexedDocument) -> TantivyDocument {
    let mut document = TantivyDocument::default();
    document.add_bytes(fields.id, source.id.as_bytes());
    document.add_text(fields.title, &source.title);
    if let Some(body) = &source.body {
        document.add_text(fields.body, body);
    }
    document.add_i64(fields.date, date_value(source.date));
    document.add_u64(fields.precision, precision_code(source.precision));
    document.add_u64(fields.language, language_code(source.language));
    // The document's own source first; a source is listed once even when it
    // holds several copies.
    let mut sources = vec![source.source];
    for copy in &source.copies {
        if !sources.contains(&copy.source) {
            sources.push(copy.source);
        }
    }
    for value in sources {
        document.add_u64(fields.source, source_code(value));
    }
    document.add_text(fields.url, &source.canonical_url);
    if let Some(category) = &source.category {
        document.add_text(fields.category, category);
    }
    if let Some(record) = &source.source_record_id {
        document.add_text(fields.record, record);
    }
    if !source.metadata.is_null() {
        document.add_text(fields.metadata, source.metadata.to_string());
    }
    if let Some(group) = source.translation_group {
        document.add_bytes(fields.group, group.as_bytes());
    }
    if !source.copies.is_empty() {
        for copy in &source.copies {
            document.add_bytes(fields.alias, copy.id.as_bytes());
        }
        // Serializing plain structs of strings cannot fail; an empty list is
        // the harmless fallback.
        let copies = match serde_json::to_string(&source.copies) {
            Ok(copies) => copies,
            Err(_) => String::from("[]"),
        };
        document.add_text(fields.copies, copies);
    }
    document
}

/// Term identifying one document, for replacement and deletion.
pub fn id_term(fields: Fields, id: Uuid) -> Term {
    Term::from_field_bytes(fields.id, id.as_bytes())
}
