//! Reads single documents and their other language editions from the index.

use tantivy::{
    TantivyDocument, Term,
    collector::TopDocs,
    query::TermQuery,
    schema::{Field, IndexRecordOption},
};
use uuid::Uuid;

use crate::{
    error::{Result, search_error},
    search::{
        engine::SearchEngine,
        request::{StoredDocument, TranslationRef},
        stored,
    },
};

/// A translation group is one article in at most six languages; the bound
/// guards against a malformed group.
const MAX_GROUP_MEMBERS: usize = 64;

impl SearchEngine {
    /// The document with this identifier. The identifier of a copy held by
    /// another source resolves to the preferred document that stands for it.
    pub fn document(&self, id: Uuid) -> Result<Option<StoredDocument>> {
        let fields = self.fields();
        for field in [fields.id, fields.alias] {
            let found = self.by_key(field, id, 1)?;
            if let Some(document) = found.first() {
                return stored::stored_document(document, fields).map(Some);
            }
        }
        Ok(None)
    }

    /// The other language editions of a document, one per language, ordered
    /// by language code.
    pub fn translations(&self, document: &StoredDocument) -> Result<Vec<TranslationRef>> {
        let Some(group) = document.translation_group else {
            return Ok(Vec::new());
        };
        let fields = self.fields();
        let mut translations: Vec<TranslationRef> = Vec::new();
        for member in self.by_key(fields.group, group, MAX_GROUP_MEMBERS)? {
            let hit = stored::hit(&member, fields)?;
            if hit.language == document.language
                || translations
                    .iter()
                    .any(|existing| existing.language == hit.language)
            {
                continue;
            }
            translations.push(TranslationRef {
                id: hit.id,
                language: hit.language,
                title: hit.title,
            });
        }
        translations.sort_unstable_by_key(|translation| translation.language.code());
        Ok(translations)
    }

    /// Stored documents whose `field` holds exactly these UUID bytes.
    fn by_key(&self, field: Field, key: Uuid, limit: usize) -> Result<Vec<TantivyDocument>> {
        let searcher = self.searcher();
        let query = TermQuery::new(
            Term::from_field_bytes(field, key.as_bytes()),
            IndexRecordOption::Basic,
        );
        let found = searcher
            .search(&query, &TopDocs::with_limit(limit).order_by_score())
            .map_err(search_error)?;
        found
            .into_iter()
            .map(|(_, address)| {
                searcher
                    .doc::<TantivyDocument>(address)
                    .map_err(search_error)
            })
            .collect()
    }
}
