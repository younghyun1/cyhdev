CREATE INDEX idx_photograph_comments_photograph_id
    ON public.photograph_comments USING btree (photograph_id);

DROP INDEX public.photograph_comments_photograph_page_idx;

CREATE INDEX idx_comments_post_id ON public.comments (post_id);

DROP INDEX public.comments_post_page_idx;
