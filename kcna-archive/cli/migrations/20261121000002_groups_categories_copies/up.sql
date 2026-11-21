-- Translations, categories, and duplicate copies become relations with foreign
-- keys instead of free text. The new columns start empty and are filled by the
-- next `database import`; the empty fingerprint marks every existing row as
-- changed so that import rewrites it once.

CREATE TYPE kcna_translation_link_method AS ENUM (
    'source_identifier',
    'inferred'
);

-- One article across its language editions.
CREATE TABLE kcna_translation_groups (
    translation_group_id uuid PRIMARY KEY DEFAULT uuidv7(),
    translation_group_source_key text NOT NULL,
    translation_group_link_method kcna_translation_link_method NOT NULL,
    translation_group_created_at timestamptz NOT NULL DEFAULT transaction_timestamp(),
    CONSTRAINT kcna_translation_groups_source_key_uniq UNIQUE (translation_group_source_key),
    CONSTRAINT kcna_translation_groups_source_key_chk CHECK (
        length(btrim(translation_group_source_key)) > 0
    )
);

-- Category names as each source and language edition titles them.
CREATE TABLE kcna_categories (
    category_id uuid PRIMARY KEY DEFAULT uuidv7(),
    category_source kcna_source NOT NULL,
    category_language kcna_language NOT NULL,
    category_name text NOT NULL,
    category_created_at timestamptz NOT NULL DEFAULT transaction_timestamp(),
    CONSTRAINT kcna_categories_natural_key_uniq UNIQUE (
        category_source,
        category_language,
        category_name
    ),
    CONSTRAINT kcna_categories_name_chk CHECK (length(btrim(category_name)) > 0)
);

DROP INDEX kcna_documents_translation_group_idx;

ALTER TABLE kcna_documents
    DROP COLUMN document_translation_group_key,
    DROP COLUMN document_category,
    ADD COLUMN document_translation_group_id uuid,
    ADD COLUMN document_category_id uuid,
    -- The preferred copy of the same article from another source; NULL when
    -- this row is itself the preferred copy.
    ADD COLUMN document_canonical_document_id uuid,
    -- Digest of everything the importer stores for the row. An unchanged
    -- digest lets a re-import skip the row.
    ADD COLUMN document_import_fingerprint bytea NOT NULL DEFAULT '\x',
    ADD CONSTRAINT kcna_documents_translation_group_fk FOREIGN KEY (document_translation_group_id)
        REFERENCES kcna_translation_groups (translation_group_id) ON DELETE SET NULL,
    ADD CONSTRAINT kcna_documents_category_fk FOREIGN KEY (document_category_id)
        REFERENCES kcna_categories (category_id) ON DELETE RESTRICT,
    ADD CONSTRAINT kcna_documents_canonical_document_fk FOREIGN KEY (document_canonical_document_id)
        REFERENCES kcna_documents (document_id) ON DELETE SET NULL,
    ADD CONSTRAINT kcna_documents_canonical_document_chk CHECK (
        document_canonical_document_id <> document_id
    ),
    ADD CONSTRAINT kcna_documents_import_fingerprint_chk CHECK (
        octet_length(document_import_fingerprint) IN (0, 32)
    );

CREATE INDEX kcna_documents_translation_group_id_idx
    ON kcna_documents (document_translation_group_id)
    WHERE document_translation_group_id IS NOT NULL;
CREATE INDEX kcna_documents_category_id_idx
    ON kcna_documents (document_category_id)
    WHERE document_category_id IS NOT NULL;
CREATE INDEX kcna_documents_canonical_document_id_idx
    ON kcna_documents (document_canonical_document_id)
    WHERE document_canonical_document_id IS NOT NULL;
