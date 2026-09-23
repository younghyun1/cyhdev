-- Deleting a comment must not delete other users' replies. Parents become
-- tombstones (empty content plus a deletion timestamp) and the self-referencing
-- foreign keys refuse physical deletion of a comment that still has replies.
-- Deleting a post still cascades its whole comment set in one statement, which
-- RESTRICT permits because parent and child rows disappear together.

ALTER TABLE public.comments
    ADD COLUMN comment_deleted_at TIMESTAMPTZ;

ALTER TABLE public.comments
    DROP CONSTRAINT fk_comments_parent,
    ADD CONSTRAINT fk_comments_parent FOREIGN KEY (parent_comment_id)
        REFERENCES public.comments (comment_id) ON DELETE RESTRICT;

ALTER TABLE public.comments
    DROP CONSTRAINT comments_comment_content_character_length,
    ADD CONSTRAINT comments_comment_content_character_length CHECK (
        (comment_deleted_at IS NULL AND char_length(comment_content) BETWEEN 1 AND 4000)
        OR (comment_deleted_at IS NOT NULL AND comment_content = '')
    ) NOT VALID;

COMMENT ON CONSTRAINT comments_comment_content_character_length ON public.comments IS
    'Live blog comments carry 1-4000 characters and tombstones carry none; legacy rows remain readable until separately remediated.';

ALTER TABLE public.photograph_comments
    ADD COLUMN photograph_comment_deleted_at TIMESTAMPTZ;

ALTER TABLE public.photograph_comments
    DROP CONSTRAINT fk_photograph_comments_parent,
    ADD CONSTRAINT fk_photograph_comments_parent FOREIGN KEY (parent_photograph_comment_id)
        REFERENCES public.photograph_comments (photograph_comment_id) ON DELETE RESTRICT;

-- Mirrors MAX_PHOTOGRAPH_COMMENT_CHARS; NOT VALID keeps any legacy row readable
-- while every new or updated row is checked.
ALTER TABLE public.photograph_comments
    ADD CONSTRAINT photograph_comments_content_character_length CHECK (
        (photograph_comment_deleted_at IS NULL
            AND char_length(photograph_comment_content) BETWEEN 1 AND 4000)
        OR (photograph_comment_deleted_at IS NOT NULL AND photograph_comment_content = '')
    ) NOT VALID;

COMMENT ON CONSTRAINT photograph_comments_content_character_length ON public.photograph_comments IS
    'Live photograph comments carry 1-4000 characters and tombstones carry none; legacy rows remain readable until separately remediated.';
