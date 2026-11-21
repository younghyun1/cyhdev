-- The search vector is now written by the application, which tokenizes Korean,
-- Chinese, and Japanese into bigrams. Whitespace tokens from to_tsvector cannot
-- match a Korean stem inside a particle form or any unsegmented CJK word.
-- Existing rows start with an empty vector; rerun the import to fill it.
DROP INDEX kcna_documents_search_gin_idx;
ALTER TABLE kcna_documents DROP COLUMN document_search;
ALTER TABLE kcna_documents
    ADD COLUMN document_search tsvector NOT NULL DEFAULT ''::tsvector;
CREATE INDEX kcna_documents_search_gin_idx
    ON kcna_documents USING gin (document_search);

-- Queries are bound as tsquery values, so the text-parsing wrapper is unused.
DROP FUNCTION kcna_websearch_query(text);

-- Browsing filters and groups by publication date. Without these every month
-- list, its count, and the timeline scanned the whole table.
CREATE INDEX kcna_documents_publication_date_idx
    ON kcna_documents (document_publication_date, document_published_at, document_id);
CREATE INDEX kcna_documents_language_publication_date_idx
    ON kcna_documents (
        document_language,
        document_publication_date,
        document_published_at,
        document_id
    );
-- Superseded by the language and publication date index above.
DROP INDEX kcna_documents_language_published_at_idx;
