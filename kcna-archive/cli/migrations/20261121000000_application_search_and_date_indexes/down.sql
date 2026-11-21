CREATE INDEX kcna_documents_language_published_at_idx
    ON kcna_documents (document_language, document_published_at, document_id);
DROP INDEX kcna_documents_language_publication_date_idx;
DROP INDEX kcna_documents_publication_date_idx;

CREATE FUNCTION kcna_websearch_query(search_text text)
RETURNS tsquery
LANGUAGE sql
STABLE
PARALLEL SAFE
STRICT
RETURN websearch_to_tsquery('simple'::regconfig, search_text);

DROP INDEX kcna_documents_search_gin_idx;
ALTER TABLE kcna_documents DROP COLUMN document_search;
ALTER TABLE kcna_documents
    ADD COLUMN document_search tsvector GENERATED ALWAYS AS (
        to_tsvector('simple', coalesce(document_title, '') || ' ' || coalesce(document_body, ''))
    ) STORED;
CREATE INDEX kcna_documents_search_gin_idx
    ON kcna_documents USING gin (document_search);
