CREATE OR REPLACE FUNCTION kcna_documents_assign_uuidv7()
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

