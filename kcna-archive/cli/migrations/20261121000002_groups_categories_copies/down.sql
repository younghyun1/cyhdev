DROP INDEX kcna_documents_canonical_document_id_idx;
DROP INDEX kcna_documents_category_id_idx;
DROP INDEX kcna_documents_translation_group_id_idx;

ALTER TABLE kcna_documents
    DROP CONSTRAINT kcna_documents_import_fingerprint_chk,
    DROP CONSTRAINT kcna_documents_canonical_document_chk,
    DROP CONSTRAINT kcna_documents_canonical_document_fk,
    DROP CONSTRAINT kcna_documents_category_fk,
    DROP CONSTRAINT kcna_documents_translation_group_fk,
    DROP COLUMN document_import_fingerprint,
    DROP COLUMN document_canonical_document_id,
    DROP COLUMN document_category_id,
    DROP COLUMN document_translation_group_id,
    ADD COLUMN document_category text,
    ADD COLUMN document_translation_group_key text;

CREATE INDEX kcna_documents_translation_group_idx
    ON kcna_documents (document_translation_group_key)
    WHERE document_translation_group_key IS NOT NULL;

DROP TABLE kcna_categories;
DROP TABLE kcna_translation_groups;
DROP TYPE kcna_translation_link_method;
