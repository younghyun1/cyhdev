CREATE OR REPLACE FUNCTION kcna_documents_assign_uuidv7()
RETURNS trigger
LANGUAGE plpgsql
VOLATILE
PARALLEL UNSAFE
AS $$
BEGIN
    IF NEW.document_id IS NULL THEN
        NEW.document_id := kcna_uuidv7_at(NEW.document_published_at);
    END IF;
    RETURN NEW;
END;
$$;

