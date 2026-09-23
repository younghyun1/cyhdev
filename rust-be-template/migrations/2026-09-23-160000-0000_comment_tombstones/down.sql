-- Restoring cascading deletes cannot recover tombstoned content, and the prior
-- content check rejects empty tombstone bodies, so rollback is refused once any
-- tombstone exists. Recovery at that point requires a backup.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.comments WHERE comment_deleted_at IS NOT NULL)
        OR EXISTS (
            SELECT 1 FROM public.photograph_comments
            WHERE photograph_comment_deleted_at IS NOT NULL
        )
    THEN
        RAISE EXCEPTION USING
            ERRCODE = 'object_not_in_prerequisite_state',
            MESSAGE = 'cannot roll back comment tombstones while tombstoned comments exist';
    END IF;
END
$$;

ALTER TABLE public.photograph_comments
    DROP CONSTRAINT photograph_comments_content_character_length;

ALTER TABLE public.photograph_comments
    DROP CONSTRAINT fk_photograph_comments_parent,
    ADD CONSTRAINT fk_photograph_comments_parent FOREIGN KEY (parent_photograph_comment_id)
        REFERENCES public.photograph_comments (photograph_comment_id) ON DELETE CASCADE;

ALTER TABLE public.photograph_comments
    DROP COLUMN photograph_comment_deleted_at;

ALTER TABLE public.comments
    DROP CONSTRAINT comments_comment_content_character_length,
    ADD CONSTRAINT comments_comment_content_character_length
        CHECK (char_length(comment_content) BETWEEN 1 AND 4000) NOT VALID;

COMMENT ON CONSTRAINT comments_comment_content_character_length ON public.comments IS
    'Enforces the public blog comment boundary for new and updated rows; legacy rows remain readable until separately remediated.';

ALTER TABLE public.comments
    DROP CONSTRAINT fk_comments_parent,
    ADD CONSTRAINT fk_comments_parent FOREIGN KEY (parent_comment_id)
        REFERENCES public.comments (comment_id) ON DELETE CASCADE;

ALTER TABLE public.comments
    DROP COLUMN comment_deleted_at;
