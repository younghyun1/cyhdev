-- The list is entered by hand and cannot be rebuilt, so clear it deliberately first.
DO $$ BEGIN
    IF EXISTS (SELECT 1 FROM kcna_archive_reader) THEN
        RAISE EXCEPTION 'Remove the KCNA archive readers before rolling back their storage';
    END IF;
END $$;
DROP TABLE kcna_archive_reader;
