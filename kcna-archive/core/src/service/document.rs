//! The reader's view of one document.

use uuid::Uuid;

use crate::{
    search::request::{StoredDocument, TranslationRef},
    search_text::{self, Clause},
    service::{
        archive::{Archive, ServiceError, segments},
        models::{CopyLink, DocumentResponse, TranslationLink},
    },
};

impl Archive {
    /// One document, with the matches of `query` marked in its title and body.
    pub fn document(&self, id: &str, query: &str) -> Result<DocumentResponse, ServiceError> {
        let Ok(id) = Uuid::parse_str(id) else {
            return Err(ServiceError::BadRequest);
        };
        let Some(stored) = self.engine().document(id)? else {
            return Err(ServiceError::NotFound);
        };
        let translations = self.engine().translations(&stored)?;
        let clauses = search_text::query_clauses(query);
        Ok(response(stored, translations, &clauses))
    }
}

fn response(
    stored: StoredDocument,
    translations: Vec<TranslationRef>,
    clauses: &[Clause],
) -> DocumentResponse {
    let paragraphs = stored
        .body
        .as_deref()
        .map(|body| {
            body.split("\n\n")
                .map(|paragraph| segments(paragraph, clauses))
                .filter(|paragraph| !paragraph.is_empty())
                .collect()
        })
        .unwrap_or_default();
    // The index never stores an entry without a source; the fallback keeps
    // this total without a panic path.
    let source = stored
        .sources
        .first()
        .copied()
        .unwrap_or(crate::types::Source::KcnaKp);
    DocumentResponse {
        id: stored.id.to_string(),
        title: segments(&stored.title, clauses),
        paragraphs,
        publication_date: stored.date.to_string(),
        publication_precision: stored.precision,
        language: stored.language,
        source,
        sources: stored.sources,
        canonical_url: stored.canonical_url,
        category: stored.category,
        source_record_id: stored.source_record_id,
        metadata: stored.metadata,
        translations: translations
            .into_iter()
            .map(|translation| TranslationLink {
                id: translation.id.to_string(),
                language: translation.language,
                title: translation.title,
            })
            .collect(),
        copies: stored
            .copies
            .into_iter()
            .map(|copy| CopyLink {
                id: copy.id.to_string(),
                source: copy.source,
                canonical_url: copy.url,
            })
            .collect(),
    }
}
