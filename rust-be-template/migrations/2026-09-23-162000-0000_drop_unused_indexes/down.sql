CREATE INDEX forum_notifications_unread_idx
    ON public.forum_notifications (
        forum_notification_recipient_user_id,
        forum_notification_created_at DESC,
        forum_notification_id DESC
    )
    WHERE forum_notification_read_at IS NULL;

CREATE INDEX photographs_lon_idx ON public.photographs USING btree (photograph_lon);
CREATE INDEX photographs_lat_idx ON public.photographs USING btree (photograph_lat);
CREATE INDEX photographs_photograph_is_on_cloud_idx
    ON public.photographs USING btree (photograph_is_on_cloud);

CREATE INDEX idx_photograph_comment_votes_comment_id
    ON public.photograph_comment_votes USING btree (photograph_comment_id);
CREATE INDEX idx_photograph_votes_photograph_id
    ON public.photograph_votes USING btree (photograph_id);
CREATE INDEX idx_post_votes_post_id ON public.post_votes (post_id);

CREATE INDEX idx_photograph_comment_votes_is_upvote
    ON public.photograph_comment_votes USING btree (is_upvote);
CREATE INDEX idx_photograph_votes_is_upvote ON public.photograph_votes USING btree (is_upvote);
CREATE INDEX idx_comment_votes_is_upvote ON public.comment_votes USING btree (is_upvote);
CREATE INDEX idx_post_votes_is_upvote ON public.post_votes USING btree (is_upvote);

CREATE INDEX idx_photographs_total_downvotes
    ON public.photographs USING btree (photograph_total_downvotes DESC);
CREATE INDEX idx_photographs_total_upvotes
    ON public.photographs USING btree (photograph_total_upvotes DESC);
CREATE INDEX idx_photographs_view_count
    ON public.photographs USING btree (photograph_view_count DESC);
CREATE INDEX idx_comments_total_downvotes ON public.comments USING btree (total_downvotes DESC);
CREATE INDEX idx_comments_total_upvotes ON public.comments USING btree (total_upvotes DESC);
CREATE INDEX idx_posts_total_downvotes ON public.posts USING btree (total_downvotes DESC);
CREATE INDEX idx_posts_total_upvotes ON public.posts USING btree (total_upvotes DESC);
CREATE INDEX idx_posts_view_count ON public.posts USING btree (post_view_count DESC);
