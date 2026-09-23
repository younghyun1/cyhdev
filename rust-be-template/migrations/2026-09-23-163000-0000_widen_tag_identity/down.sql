-- Narrowing is only safe while every stored and future identity value still
-- fits in smallint; otherwise rollback is refused rather than truncating keys.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.tags WHERE tag_id > 32767)
        OR (SELECT last_value FROM public.tags_tag_id_seq) > 32767
    THEN
        RAISE EXCEPTION USING
            ERRCODE = 'object_not_in_prerequisite_state',
            MESSAGE = 'cannot narrow tag identifiers that exceed the smallint range';
    END IF;
END
$$;

ALTER TABLE public.post_tags ALTER COLUMN tag_id TYPE smallint;
ALTER TABLE public.tags ALTER COLUMN tag_id TYPE smallint;
