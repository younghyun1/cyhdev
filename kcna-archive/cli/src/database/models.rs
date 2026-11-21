//! Diesel rows used for bounded bulk upserts.

use diesel::Insertable;
use serde_json::Value;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use crate::database::{
    domain::DocumentBundle,
    schema::{
        kcna_categories, kcna_document_retrievals, kcna_documents, kcna_retrievals,
        kcna_translation_groups,
    },
    types::{Language, PublicationPrecision, Source, TranslationLinkMethod},
};

#[derive(Debug, Insertable)]
#[diesel(table_name = kcna_documents, check_for_backend(Pg))]
pub struct NewDocument {
    pub document_source: Source,
    pub document_source_record_id: Option<String>,
    pub document_canonical_url: String,
    pub document_element_index: i16,
    pub document_language: Language,
    pub document_translation_group_id: Option<Uuid>,
    pub document_title: String,
    pub document_body: Option<String>,
    pub document_published_at: OffsetDateTime,
    pub document_publication_precision: PublicationPrecision,
    pub document_publication_date: Date,
    pub document_published_at_raw: Option<String>,
    pub document_category_id: Option<Uuid>,
    pub document_content_sha256: Vec<u8>,
    pub document_copyright_status: Option<String>,
    pub document_source_metadata: Value,
    pub document_first_retrieved_at: OffsetDateTime,
    pub document_last_retrieved_at: OffsetDateTime,
    pub document_import_fingerprint: Vec<u8>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = kcna_retrievals, check_for_backend(Pg))]
pub struct NewRetrieval {
    pub retrieval_retrieved_at: OffsetDateTime,
    pub retrieval_requested_url: String,
    pub retrieval_final_url: Option<String>,
    pub retrieval_http_status: Option<i16>,
    pub retrieval_media_type: Option<String>,
    pub retrieval_charset: Option<String>,
    pub retrieval_raw_content: Vec<u8>,
    pub retrieval_raw_sha256: Vec<u8>,
    pub retrieval_response_headers: Value,
    pub retrieval_manifest_metadata: Value,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = kcna_document_retrievals, check_for_backend(Pg))]
pub struct NewDocumentRetrieval {
    pub document_retrieval_document_id: Uuid,
    pub document_retrieval_retrieval_id: Uuid,
}

impl NewDocument {
    /// Builds the row for a bundle whose group and category were resolved to
    /// their identifiers.
    pub fn from_bundle(
        bundle: &DocumentBundle,
        fingerprint: [u8; 32],
        translation_group_id: Option<Uuid>,
        category_id: Option<Uuid>,
    ) -> Self {
        Self {
            document_source: bundle.source,
            document_source_record_id: bundle.source_record_id.clone(),
            document_canonical_url: bundle.canonical_url.as_ref().to_owned(),
            document_element_index: *bundle.element_index,
            document_language: bundle.language,
            document_translation_group_id: translation_group_id,
            document_title: bundle.title.as_ref().to_owned(),
            document_body: bundle.body.as_ref().map(|body| body.as_ref().to_owned()),
            document_published_at: bundle.published_at,
            document_publication_precision: bundle.publication_precision,
            document_publication_date: bundle.publication_date,
            document_published_at_raw: bundle.published_at_raw.clone(),
            document_category_id: category_id,
            document_content_sha256: bundle.content_sha256.to_vec(),
            document_copyright_status: bundle.copyright_status.clone(),
            document_source_metadata: bundle.source_metadata.clone(),
            document_first_retrieved_at: bundle.retrieved_at,
            document_last_retrieved_at: bundle.retrieved_at,
            document_import_fingerprint: fingerprint.to_vec(),
        }
    }
}

/// A translation group to create when its source key is new.
#[derive(Debug, Insertable)]
#[diesel(table_name = kcna_translation_groups, check_for_backend(Pg))]
pub struct NewTranslationGroup {
    pub translation_group_source_key: String,
    pub translation_group_link_method: TranslationLinkMethod,
}

/// A category to create when its name is new for the source and language.
#[derive(Debug, Insertable)]
#[diesel(table_name = kcna_categories, check_for_backend(Pg))]
pub struct NewCategory {
    pub category_source: Source,
    pub category_language: Language,
    pub category_name: String,
}

impl NewRetrieval {
    pub fn from_bundle(bundle: &DocumentBundle) -> Self {
        Self {
            retrieval_retrieved_at: bundle.retrieved_at,
            retrieval_requested_url: bundle.requested_url.as_ref().to_owned(),
            retrieval_final_url: bundle.final_url.as_ref().map(|url| url.as_ref().to_owned()),
            retrieval_http_status: bundle.http_status,
            retrieval_media_type: bundle.media_type.clone(),
            retrieval_charset: bundle.charset.clone(),
            retrieval_raw_content: bundle.raw_content.as_ref().to_vec(),
            retrieval_raw_sha256: bundle.raw_sha256.to_vec(),
            retrieval_response_headers: bundle.response_headers.clone(),
            retrieval_manifest_metadata: bundle.manifest_metadata.clone(),
        }
    }
}
