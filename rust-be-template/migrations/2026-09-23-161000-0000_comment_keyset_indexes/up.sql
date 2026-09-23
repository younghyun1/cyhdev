-- Comment pages read one post or photograph in (created_at, id) order and bound
-- the scan with a row comparison against the previous page's last row. The
-- composite indexes serve that range and, through their leading column, the
-- foreign-key lookups the single-column indexes served before.

CREATE INDEX comments_post_page_idx
    ON public.comments (post_id, comment_created_at, comment_id);

DROP INDEX public.idx_comments_post_id;

CREATE INDEX photograph_comments_photograph_page_idx
    ON public.photograph_comments (photograph_id, photograph_comment_created_at, photograph_comment_id);

DROP INDEX public.idx_photograph_comments_photograph_id;
