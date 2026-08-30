-- A rollback cannot reconstruct password credentials, role assignments, or consumed
-- tokens. Refuse to erase lifecycle metadata once the feature has processed an account.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.deleted_account_retention)
        OR EXISTS (SELECT 1 FROM public.media_object_cleanup)
        OR EXISTS (
            SELECT 1
            FROM public.users
            WHERE user_deleted_at IS NOT NULL OR user_hard_purged_at IS NOT NULL
        )
    THEN
        RAISE EXCEPTION USING
            ERRCODE = 'object_not_in_prerequisite_state',
            MESSAGE = 'cannot roll back account lifecycle after an account deletion',
            HINT = 'Restore from a pre-deletion backup or retain the lifecycle schema.';
    END IF;
END
$$;

DROP TRIGGER users_permanent_tombstone_delete_guard ON public.users;
DROP FUNCTION public.protect_users_permanent_tombstones();
DROP TRIGGER users_lifecycle_update_guard ON public.users;
DROP FUNCTION public.protect_users_lifecycle_updates();

ALTER TABLE public.posts
    DROP CONSTRAINT posts_user_retention_fk,
    ADD CONSTRAINT fk_posts_user
        FOREIGN KEY (user_id) REFERENCES public.users (user_id) ON DELETE CASCADE;

ALTER TABLE public.comments
    DROP CONSTRAINT comments_user_retention_fk,
    ADD CONSTRAINT fk_comments_user
        FOREIGN KEY (user_id) REFERENCES public.users (user_id) ON DELETE CASCADE;

ALTER TABLE public.photographs
    DROP CONSTRAINT photographs_user_retention_fk,
    ADD CONSTRAINT photographs_users_fk
        FOREIGN KEY (user_id) REFERENCES public.users (user_id)
        ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE public.photograph_comments
    DROP CONSTRAINT photograph_comments_user_retention_fk,
    ADD CONSTRAINT fk_photograph_comments_user
        FOREIGN KEY (user_id) REFERENCES public.users (user_id) ON DELETE CASCADE;

ALTER TABLE public.live_chat_messages
    DROP CONSTRAINT live_chat_messages_user_retention_fk,
    ADD CONSTRAINT live_chat_messages_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES public.users (user_id) ON DELETE SET NULL;

ALTER TABLE public.live_chat_call_participants
    DROP CONSTRAINT live_chat_call_participants_user_retention_fk,
    ADD CONSTRAINT live_chat_call_participants_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES public.users (user_id) ON DELETE SET NULL;

DROP TABLE public.media_object_cleanup;
DROP TABLE public.deleted_account_retention;

DROP INDEX public.user_profile_pictures_one_active_per_user;
DROP INDEX public.user_profile_pictures_history_idx;
ALTER TABLE public.user_profile_pictures
    DROP CONSTRAINT user_profile_pictures_cloud_link_pair,
    DROP CONSTRAINT user_profile_pictures_active_requires_cloud,
    DROP COLUMN user_profile_picture_is_active;
DROP INDEX public.users_purge_after_pending_idx;

ALTER TABLE public.users
    DROP CONSTRAINT users_system_actor_cannot_be_deleted,
    DROP CONSTRAINT users_hard_purge_not_before_purge_after,
    DROP CONSTRAINT users_hard_purge_requires_deletion,
    DROP CONSTRAINT users_purge_after_not_before_deletion,
    DROP CONSTRAINT users_deletion_schedule_pair,
    DROP COLUMN user_is_system_actor,
    DROP COLUMN user_hard_purged_at,
    DROP COLUMN user_purge_after,
    DROP COLUMN user_deleted_at;
