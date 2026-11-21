-- Search moved to the on-disk index built from these rows (`database index`),
-- so the per-row search vector and its GIN index are no longer read.
DROP INDEX kcna_documents_search_gin_idx;
ALTER TABLE kcna_documents DROP COLUMN document_search;
