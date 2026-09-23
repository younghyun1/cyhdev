-- No query filters or orders by these columns, so the indexes only cost write
-- amplification. An index on a counter also turns every counter update into a
-- non-HOT row update, which is what blog and photograph view flushes and vote
-- recounts do most. Each was checked against every repository query, sort
-- option, and admin read before removal.

-- Denormalized counters: read with their rows, never used as a sort key.
DROP INDEX public.idx_posts_view_count;
DROP INDEX public.idx_posts_total_upvotes;
DROP INDEX public.idx_posts_total_downvotes;
DROP INDEX public.idx_comments_total_upvotes;
DROP INDEX public.idx_comments_total_downvotes;
DROP INDEX public.idx_photographs_view_count;
DROP INDEX public.idx_photographs_total_upvotes;
DROP INDEX public.idx_photographs_total_downvotes;

-- Boolean vote direction: always filtered together with the voted row, which
-- the per-row unique vote indexes already serve.
DROP INDEX public.idx_post_votes_is_upvote;
DROP INDEX public.idx_comment_votes_is_upvote;
DROP INDEX public.idx_photograph_votes_is_upvote;
DROP INDEX public.idx_photograph_comment_votes_is_upvote;

-- Leading-column duplicates of the (target, user) unique vote indexes.
DROP INDEX public.idx_post_votes_post_id;
DROP INDEX public.idx_photograph_votes_photograph_id;
DROP INDEX public.idx_photograph_comment_votes_comment_id;

-- Photograph storage flag and coordinates are returned, never searched.
DROP INDEX public.photographs_photograph_is_on_cloud_idx;
DROP INDEX public.photographs_lat_idx;
DROP INDEX public.photographs_lon_idx;

-- The inbox lists read and unread notifications through the recipient page
-- index; no query restricts to unread rows.
DROP INDEX public.forum_notifications_unread_idx;
