CREATE TYPE kcna_source AS ENUM (
    'kcna_kp',
    'kcna_co_jp',
    'stiles_archive'
);

CREATE TYPE kcna_language AS ENUM (
    'kor',
    'eng',
    'zho',
    'rus',
    'spa',
    'jpn'
);

CREATE TYPE kcna_publication_precision AS ENUM (
    'second',
    'minute',
    'day',
    'inferred_day'
);

CREATE FUNCTION kcna_uuidv7_at(source_timestamp timestamptz)
RETURNS uuid
LANGUAGE sql
VOLATILE
PARALLEL SAFE
STRICT
RETURN uuidv7(source_timestamp - clock_timestamp());

CREATE FUNCTION kcna_documents_assign_uuidv7()
RETURNS trigger
LANGUAGE plpgsql
VOLATILE
PARALLEL UNSAFE
AS $$
BEGIN
    IF NEW.document_publication_precision IN ('day', 'inferred_day') THEN
        NEW.document_published_at :=
            (NEW.document_publication_date + TIME '12:00:00')
            AT TIME ZONE 'Asia/Pyongyang';
    END IF;
    IF NEW.document_id IS NULL THEN
        NEW.document_id := kcna_uuidv7_at(NEW.document_published_at);
    END IF;
    RETURN NEW;
END;
$$;

CREATE FUNCTION kcna_retrievals_assign_uuidv7()
RETURNS trigger
LANGUAGE plpgsql
VOLATILE
PARALLEL UNSAFE
AS $$
BEGIN
    IF NEW.retrieval_id IS NULL THEN
        NEW.retrieval_id := kcna_uuidv7_at(NEW.retrieval_retrieved_at);
    END IF;
    RETURN NEW;
END;
$$;

CREATE TABLE kcna_documents (
    document_id uuid PRIMARY KEY,
    document_source kcna_source NOT NULL,
    document_source_record_id text,
    document_canonical_url text NOT NULL,
    document_element_index smallint NOT NULL DEFAULT 0,
    document_language kcna_language NOT NULL,
    document_translation_group_key text,
    document_title text NOT NULL,
    document_body text,
    document_published_at timestamptz NOT NULL,
    document_publication_precision kcna_publication_precision NOT NULL,
    document_publication_date date NOT NULL,
    document_published_at_raw text,
    document_category text,
    document_content_sha256 bytea NOT NULL,
    document_copyright_status text,
    document_source_metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    document_search tsvector GENERATED ALWAYS AS (
        to_tsvector('simple', coalesce(document_title, '') || ' ' || coalesce(document_body, ''))
    ) STORED,
    document_first_retrieved_at timestamptz NOT NULL,
    document_last_retrieved_at timestamptz NOT NULL,
    document_created_at timestamptz NOT NULL DEFAULT transaction_timestamp(),
    document_updated_at timestamptz NOT NULL DEFAULT transaction_timestamp(),
    CONSTRAINT kcna_documents_natural_key_uniq UNIQUE (
        document_source,
        document_canonical_url,
        document_element_index,
        document_language
    ),
    CONSTRAINT kcna_documents_document_id_version_chk CHECK (
        uuid_extract_version(document_id) = 7
    ),
    CONSTRAINT kcna_documents_document_id_timestamp_chk CHECK (
        abs(extract(epoch FROM uuid_extract_timestamp(document_id) - document_published_at)) < 0.025
    ),
    CONSTRAINT kcna_documents_element_index_chk CHECK (
        document_element_index >= 0
    ),
    CONSTRAINT kcna_documents_canonical_url_chk CHECK (
        document_canonical_url ~ '^https?://[^[:space:]]+$'
    ),
    CONSTRAINT kcna_documents_title_chk CHECK (
        length(btrim(document_title)) > 0
    ),
    CONSTRAINT kcna_documents_body_chk CHECK (
        document_body IS NULL OR length(btrim(document_body)) > 0
    ),
    CONSTRAINT kcna_documents_publication_date_chk CHECK (
        document_publication_date BETWEEN DATE '1996-01-01' AND DATE '2100-12-31'
    ),
    CONSTRAINT kcna_documents_publication_date_matches_timestamp_chk CHECK (
        document_publication_date = (document_published_at AT TIME ZONE 'Asia/Pyongyang')::date
    ),
    CONSTRAINT kcna_documents_day_precision_anchor_chk CHECK (
        document_publication_precision NOT IN ('day', 'inferred_day')
        OR document_published_at = (document_publication_date + TIME '12:00:00') AT TIME ZONE 'Asia/Pyongyang'
    ),
    CONSTRAINT kcna_documents_content_sha256_chk CHECK (
        octet_length(document_content_sha256) = 32
    ),
    CONSTRAINT kcna_documents_retrieval_order_chk CHECK (
        document_last_retrieved_at >= document_first_retrieved_at
    )
);

CREATE INDEX kcna_documents_published_at_idx
    ON kcna_documents (document_published_at, document_id);
CREATE INDEX kcna_documents_language_published_at_idx
    ON kcna_documents (document_language, document_published_at, document_id);
CREATE INDEX kcna_documents_source_published_at_idx
    ON kcna_documents (document_source, document_published_at, document_id);
CREATE UNIQUE INDEX kcna_documents_source_record_uniq
    ON kcna_documents (document_source, document_source_record_id, document_language)
    WHERE document_source_record_id IS NOT NULL;
CREATE INDEX kcna_documents_translation_group_idx
    ON kcna_documents (document_translation_group_key)
    WHERE document_translation_group_key IS NOT NULL;
CREATE INDEX kcna_documents_metadata_gin_idx
    ON kcna_documents USING gin (document_source_metadata jsonb_path_ops);
CREATE INDEX kcna_documents_search_gin_idx
    ON kcna_documents USING gin (document_search);
CREATE INDEX kcna_documents_content_sha256_idx
    ON kcna_documents (document_content_sha256);
CREATE INDEX kcna_documents_canonical_url_idx
    ON kcna_documents (document_canonical_url);

CREATE TRIGGER kcna_documents_assign_uuidv7_trg
    BEFORE INSERT ON kcna_documents
    FOR EACH ROW
    EXECUTE FUNCTION kcna_documents_assign_uuidv7();

CREATE TABLE kcna_retrievals (
    retrieval_id uuid PRIMARY KEY,
    retrieval_retrieved_at timestamptz NOT NULL,
    retrieval_requested_url text NOT NULL,
    retrieval_final_url text,
    retrieval_http_status smallint,
    retrieval_media_type text,
    retrieval_charset text,
    retrieval_raw_content bytea NOT NULL,
    retrieval_raw_sha256 bytea NOT NULL,
    retrieval_response_headers jsonb NOT NULL DEFAULT '{}'::jsonb,
    retrieval_manifest_metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    retrieval_created_at timestamptz NOT NULL DEFAULT transaction_timestamp(),
    CONSTRAINT kcna_retrievals_url_body_uniq UNIQUE (
        retrieval_requested_url,
        retrieval_raw_sha256
    ),
    CONSTRAINT kcna_retrievals_retrieval_id_version_chk CHECK (
        uuid_extract_version(retrieval_id) = 7
    ),
    CONSTRAINT kcna_retrievals_retrieval_id_timestamp_chk CHECK (
        abs(extract(epoch FROM uuid_extract_timestamp(retrieval_id) - retrieval_retrieved_at)) < 0.025
    ),
    CONSTRAINT kcna_retrievals_requested_url_chk CHECK (
        retrieval_requested_url ~ '^https?://[^[:space:]]+$'
    ),
    CONSTRAINT kcna_retrievals_final_url_chk CHECK (
        retrieval_final_url IS NULL OR retrieval_final_url ~ '^https?://[^[:space:]]+$'
    ),
    CONSTRAINT kcna_retrievals_http_status_chk CHECK (
        retrieval_http_status IS NULL OR retrieval_http_status BETWEEN 100 AND 599
    ),
    CONSTRAINT kcna_retrievals_raw_content_chk CHECK (
        octet_length(retrieval_raw_content) > 0
    ),
    CONSTRAINT kcna_retrievals_raw_sha256_chk CHECK (
        octet_length(retrieval_raw_sha256) = 32
    )
);

CREATE INDEX kcna_retrievals_retrieved_at_idx
    ON kcna_retrievals (retrieval_retrieved_at, retrieval_id);
CREATE INDEX kcna_retrievals_requested_url_idx
    ON kcna_retrievals (retrieval_requested_url, retrieval_retrieved_at DESC);
CREATE INDEX kcna_retrievals_manifest_gin_idx
    ON kcna_retrievals USING gin (retrieval_manifest_metadata jsonb_path_ops);

CREATE TRIGGER kcna_retrievals_assign_uuidv7_trg
    BEFORE INSERT ON kcna_retrievals
    FOR EACH ROW
    EXECUTE FUNCTION kcna_retrievals_assign_uuidv7();

CREATE TABLE kcna_document_retrievals (
    document_retrieval_document_id uuid NOT NULL,
    document_retrieval_retrieval_id uuid NOT NULL,
    document_retrieval_created_at timestamptz NOT NULL DEFAULT transaction_timestamp(),
    CONSTRAINT kcna_document_retrievals_pk PRIMARY KEY (
        document_retrieval_document_id,
        document_retrieval_retrieval_id
    ),
    CONSTRAINT kcna_document_retrievals_document_fk FOREIGN KEY (
        document_retrieval_document_id
    ) REFERENCES kcna_documents (document_id) ON DELETE CASCADE,
    CONSTRAINT kcna_document_retrievals_retrieval_fk FOREIGN KEY (
        document_retrieval_retrieval_id
    ) REFERENCES kcna_retrievals (retrieval_id) ON DELETE CASCADE
);

CREATE INDEX kcna_document_retrievals_retrieval_id_idx
    ON kcna_document_retrievals (document_retrieval_retrieval_id);
