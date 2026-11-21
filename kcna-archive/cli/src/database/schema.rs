//! Hand-maintained Diesel schema kept in lockstep with embedded migrations.

pub mod sql_types {
    pub use kcna_archive_core::types::sql_types::{
        KcnaLanguage, KcnaPublicationPrecision, KcnaSource,
    };

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "kcna_translation_link_method"))]
    pub struct KcnaTranslationLinkMethod;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::{KcnaLanguage, KcnaPublicationPrecision, KcnaSource};

    kcna_documents (document_id) {
        document_id -> Uuid,
        document_source -> KcnaSource,
        document_source_record_id -> Nullable<Text>,
        document_canonical_url -> Text,
        document_element_index -> Int2,
        document_language -> KcnaLanguage,
        document_title -> Text,
        document_body -> Nullable<Text>,
        document_published_at -> Timestamptz,
        document_publication_precision -> KcnaPublicationPrecision,
        document_publication_date -> Date,
        document_published_at_raw -> Nullable<Text>,
        document_content_sha256 -> Bytea,
        document_copyright_status -> Nullable<Text>,
        document_source_metadata -> Jsonb,
        document_first_retrieved_at -> Timestamptz,
        document_last_retrieved_at -> Timestamptz,
        document_created_at -> Timestamptz,
        document_updated_at -> Timestamptz,
        document_translation_group_id -> Nullable<Uuid>,
        document_category_id -> Nullable<Uuid>,
        document_canonical_document_id -> Nullable<Uuid>,
        document_import_fingerprint -> Bytea,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::KcnaTranslationLinkMethod;

    kcna_translation_groups (translation_group_id) {
        translation_group_id -> Uuid,
        translation_group_source_key -> Text,
        translation_group_link_method -> KcnaTranslationLinkMethod,
        translation_group_created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::{KcnaLanguage, KcnaSource};

    kcna_categories (category_id) {
        category_id -> Uuid,
        category_source -> KcnaSource,
        category_language -> KcnaLanguage,
        category_name -> Text,
        category_created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    kcna_retrievals (retrieval_id) {
        retrieval_id -> Uuid,
        retrieval_retrieved_at -> Timestamptz,
        retrieval_requested_url -> Text,
        retrieval_final_url -> Nullable<Text>,
        retrieval_http_status -> Nullable<Int2>,
        retrieval_media_type -> Nullable<Text>,
        retrieval_charset -> Nullable<Text>,
        retrieval_raw_content -> Bytea,
        retrieval_raw_sha256 -> Bytea,
        retrieval_response_headers -> Jsonb,
        retrieval_manifest_metadata -> Jsonb,
        retrieval_created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    kcna_document_retrievals (document_retrieval_document_id, document_retrieval_retrieval_id) {
        document_retrieval_document_id -> Uuid,
        document_retrieval_retrieval_id -> Uuid,
        document_retrieval_created_at -> Timestamptz,
    }
}

diesel::joinable!(kcna_document_retrievals -> kcna_documents (document_retrieval_document_id));
diesel::joinable!(kcna_documents -> kcna_translation_groups (document_translation_group_id));
diesel::joinable!(kcna_documents -> kcna_categories (document_category_id));
diesel::joinable!(kcna_document_retrievals -> kcna_retrievals (document_retrieval_retrieval_id));
diesel::allow_tables_to_appear_in_same_query!(
    kcna_documents,
    kcna_retrievals,
    kcna_document_retrievals,
    kcna_translation_groups,
    kcna_categories,
);
