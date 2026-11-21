ALTER TABLE kcna_documents
    ADD COLUMN document_search tsvector NOT NULL DEFAULT ''::tsvector;
CREATE INDEX kcna_documents_search_gin_idx
    ON kcna_documents USING gin (document_search);
