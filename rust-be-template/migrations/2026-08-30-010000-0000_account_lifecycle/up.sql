ALTER TABLE public.users
    ADD COLUMN user_deleted_at TIMESTAMPTZ,
    ADD COLUMN user_purge_after TIMESTAMPTZ,
    ADD COLUMN user_hard_purged_at TIMESTAMPTZ,
    ADD COLUMN user_is_system_actor BOOLEAN NOT NULL DEFAULT FALSE;

DO $$
BEGIN
    UPDATE public.users
    SET user_is_system_actor = TRUE
    WHERE user_id = '00000000-0000-0000-0000-000000000000'::uuid;

    IF NOT FOUND THEN
        RAISE EXCEPTION USING
            ERRCODE = 'foreign_key_violation',
            MESSAGE = 'cannot install account lifecycle without the protected system actor';
    END IF;
END
$$;

ALTER TABLE public.users
    ADD CONSTRAINT users_deletion_schedule_pair CHECK (
        (user_deleted_at IS NULL AND user_purge_after IS NULL)
        OR (user_deleted_at IS NOT NULL AND user_purge_after IS NOT NULL)
    ),
    ADD CONSTRAINT users_purge_after_not_before_deletion CHECK (
        user_purge_after IS NULL OR user_purge_after >= user_deleted_at
    ),
    ADD CONSTRAINT users_hard_purge_requires_deletion CHECK (
        user_hard_purged_at IS NULL
        OR (user_deleted_at IS NOT NULL AND user_purge_after IS NOT NULL)
    ),
    ADD CONSTRAINT users_hard_purge_not_before_purge_after CHECK (
        user_hard_purged_at IS NULL OR user_hard_purged_at >= user_purge_after
    ),
    ADD CONSTRAINT users_system_actor_cannot_be_deleted CHECK (
        NOT user_is_system_actor
        OR (
            user_deleted_at IS NULL
            AND user_purge_after IS NULL
            AND user_hard_purged_at IS NULL
        )
    );

CREATE FUNCTION public.protect_users_permanent_tombstones()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF OLD.user_is_system_actor OR OLD.user_deleted_at IS NOT NULL THEN
        RAISE EXCEPTION USING
            ERRCODE = 'integrity_constraint_violation',
            MESSAGE = 'protected system actors and account tombstones cannot be deleted';
    END IF;
    RETURN OLD;
END
$$;

CREATE TRIGGER users_permanent_tombstone_delete_guard
BEFORE DELETE ON public.users
FOR EACH ROW
EXECUTE FUNCTION public.protect_users_permanent_tombstones();

CREATE FUNCTION public.protect_users_lifecycle_updates()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF OLD.user_is_system_actor IS DISTINCT FROM NEW.user_is_system_actor THEN
        RAISE EXCEPTION USING
            ERRCODE = 'integrity_constraint_violation',
            MESSAGE = 'system actor status cannot be changed';
    END IF;

    IF OLD.user_deleted_at IS NULL
        AND NEW.user_deleted_at IS NOT NULL
        AND NEW.user_hard_purged_at IS NOT NULL
    THEN
        RAISE EXCEPTION USING
            ERRCODE = 'integrity_constraint_violation',
            MESSAGE = 'account deletion and hard purge must be separate transitions';
    END IF;

    IF OLD.user_deleted_at IS NOT NULL THEN
        IF NEW.user_id IS DISTINCT FROM OLD.user_id
            OR NEW.user_deleted_at IS NULL
            OR NEW.user_deleted_at IS DISTINCT FROM OLD.user_deleted_at
            OR NEW.user_purge_after IS DISTINCT FROM OLD.user_purge_after
            OR NEW.user_name IS DISTINCT FROM OLD.user_name
            OR NEW.user_email IS DISTINCT FROM OLD.user_email
            OR NEW.user_password_hash IS DISTINCT FROM OLD.user_password_hash
            OR NEW.user_created_at IS DISTINCT FROM OLD.user_created_at
            OR NEW.user_is_email_verified IS DISTINCT FROM OLD.user_is_email_verified
            OR NEW.user_country IS DISTINCT FROM OLD.user_country
            OR NEW.user_language IS DISTINCT FROM OLD.user_language
            OR NEW.user_subdivision IS DISTINCT FROM OLD.user_subdivision
            OR NEW.user_is_system_actor IS DISTINCT FROM OLD.user_is_system_actor
            OR (
                OLD.user_hard_purged_at IS NOT NULL
                AND NEW.user_hard_purged_at IS DISTINCT FROM OLD.user_hard_purged_at
            )
        THEN
            RAISE EXCEPTION USING
                ERRCODE = 'integrity_constraint_violation',
                MESSAGE = 'deleted account tombstone identity cannot be changed or reactivated';
        END IF;
    END IF;

    RETURN NEW;
END
$$;

CREATE TRIGGER users_lifecycle_update_guard
BEFORE UPDATE ON public.users
FOR EACH ROW
EXECUTE FUNCTION public.protect_users_lifecycle_updates();

CREATE INDEX users_purge_after_pending_idx
    ON public.users (user_purge_after, user_id)
    WHERE user_deleted_at IS NOT NULL AND user_hard_purged_at IS NULL;

CREATE INDEX user_profile_pictures_history_idx
    ON public.user_profile_pictures (
        user_id,
        user_profile_picture_created_at DESC,
        user_profile_picture_id DESC
    );

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.user_profile_pictures
        WHERE user_profile_picture_is_on_cloud
            IS DISTINCT FROM (user_profile_picture_link IS NOT NULL)
    ) THEN
        RAISE EXCEPTION USING
            ERRCODE = 'check_violation',
            MESSAGE = 'cannot enforce user_profile_pictures_cloud_link_pair: legacy rows are inconsistent',
            HINT = 'Reconcile cloud status and link values explicitly, then rerun the migration.';
    END IF;
END
$$;

ALTER TABLE public.user_profile_pictures
    ADD CONSTRAINT user_profile_pictures_cloud_link_pair CHECK (
        user_profile_picture_is_on_cloud
            = (user_profile_picture_link IS NOT NULL)
    );

ALTER TABLE public.user_profile_pictures
    ADD COLUMN user_profile_picture_is_active BOOLEAN NOT NULL DEFAULT FALSE;

WITH ranked_profile_pictures AS (
    SELECT
        user_profile_picture_id,
        row_number() OVER (
            PARTITION BY user_id
            ORDER BY
                user_profile_picture_created_at DESC,
                user_profile_picture_id DESC
        ) AS profile_rank
    FROM public.user_profile_pictures
    WHERE user_profile_picture_is_on_cloud = TRUE
        AND user_profile_picture_link IS NOT NULL
)
UPDATE public.user_profile_pictures AS profile_picture
SET user_profile_picture_is_active = TRUE
FROM ranked_profile_pictures
WHERE profile_picture.user_profile_picture_id
    = ranked_profile_pictures.user_profile_picture_id
    AND ranked_profile_pictures.profile_rank = 1;

CREATE UNIQUE INDEX user_profile_pictures_one_active_per_user
    ON public.user_profile_pictures (user_id)
    WHERE user_profile_picture_is_active;

ALTER TABLE public.user_profile_pictures
    ADD CONSTRAINT user_profile_pictures_active_requires_cloud CHECK (
        NOT user_profile_picture_is_active
        OR (
            user_profile_picture_is_on_cloud
            AND user_profile_picture_link IS NOT NULL
        )
    );

ALTER TABLE public.posts
    DROP CONSTRAINT fk_posts_user,
    ADD CONSTRAINT posts_user_retention_fk
        FOREIGN KEY (user_id) REFERENCES public.users (user_id) ON DELETE RESTRICT;

ALTER TABLE public.comments
    DROP CONSTRAINT fk_comments_user,
    ADD CONSTRAINT comments_user_retention_fk
        FOREIGN KEY (user_id) REFERENCES public.users (user_id) ON DELETE RESTRICT;

ALTER TABLE public.photographs
    DROP CONSTRAINT photographs_users_fk,
    ADD CONSTRAINT photographs_user_retention_fk
        FOREIGN KEY (user_id) REFERENCES public.users (user_id)
        ON DELETE RESTRICT ON UPDATE CASCADE;

ALTER TABLE public.photograph_comments
    DROP CONSTRAINT fk_photograph_comments_user,
    ADD CONSTRAINT photograph_comments_user_retention_fk
        FOREIGN KEY (user_id) REFERENCES public.users (user_id) ON DELETE RESTRICT;

ALTER TABLE public.live_chat_messages
    DROP CONSTRAINT live_chat_messages_user_id_fkey,
    ADD CONSTRAINT live_chat_messages_user_retention_fk
        FOREIGN KEY (user_id) REFERENCES public.users (user_id) ON DELETE RESTRICT;

ALTER TABLE public.live_chat_call_participants
    DROP CONSTRAINT live_chat_call_participants_user_id_fkey,
    ADD CONSTRAINT live_chat_call_participants_user_retention_fk
        FOREIGN KEY (user_id) REFERENCES public.users (user_id) ON DELETE RESTRICT;

CREATE TABLE public.deleted_account_retention (
    deleted_account_retention_id UUID PRIMARY KEY DEFAULT uuidv7 (),
    deleted_account_retention_user_id UUID NOT NULL,
    deleted_account_retention_user_name VARCHAR NOT NULL,
    deleted_account_retention_email VARCHAR NOT NULL,
    deleted_account_retention_country INTEGER NOT NULL,
    deleted_account_retention_language INTEGER NOT NULL,
    deleted_account_retention_subdivision INTEGER,
    deleted_account_retention_created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    CONSTRAINT deleted_account_retention_user_unique
        UNIQUE (deleted_account_retention_user_id),
    CONSTRAINT deleted_account_retention_user_fk
        FOREIGN KEY (deleted_account_retention_user_id)
        REFERENCES public.users (user_id) ON DELETE RESTRICT,
    CONSTRAINT deleted_account_retention_country_fk
        FOREIGN KEY (deleted_account_retention_country)
        REFERENCES public.iso_country (country_code) ON DELETE RESTRICT,
    CONSTRAINT deleted_account_retention_language_fk
        FOREIGN KEY (deleted_account_retention_language)
        REFERENCES public.iso_language (language_code) ON DELETE RESTRICT,
    CONSTRAINT deleted_account_retention_subdivision_fk
        FOREIGN KEY (deleted_account_retention_subdivision)
        REFERENCES public.iso_country_subdivision (subdivision_id) ON DELETE RESTRICT
);

CREATE INDEX deleted_account_retention_created_at_idx
    ON public.deleted_account_retention (deleted_account_retention_created_at);

CREATE INDEX deleted_account_retention_country_idx
    ON public.deleted_account_retention (deleted_account_retention_country);

CREATE INDEX deleted_account_retention_language_idx
    ON public.deleted_account_retention (deleted_account_retention_language);

CREATE INDEX deleted_account_retention_subdivision_idx
    ON public.deleted_account_retention (deleted_account_retention_subdivision)
    WHERE deleted_account_retention_subdivision IS NOT NULL;

CREATE TABLE public.media_object_cleanup (
    media_object_cleanup_id UUID PRIMARY KEY DEFAULT uuidv7 (),
    media_object_cleanup_bucket VARCHAR(255),
    media_object_cleanup_key TEXT,
    media_object_cleanup_original_url TEXT NOT NULL,
    media_object_cleanup_reason VARCHAR(64) NOT NULL,
    media_object_cleanup_source_id UUID NOT NULL,
    media_object_cleanup_attempt_count INTEGER NOT NULL DEFAULT 0,
    media_object_cleanup_created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    media_object_cleanup_last_attempt_at TIMESTAMPTZ,
    media_object_cleanup_last_error TEXT,
    CONSTRAINT media_object_cleanup_location_pair CHECK (
        (media_object_cleanup_bucket IS NULL AND media_object_cleanup_key IS NULL)
        OR (media_object_cleanup_bucket IS NOT NULL AND media_object_cleanup_key IS NOT NULL)
    ),
    CONSTRAINT media_object_cleanup_bucket_not_empty CHECK (
        media_object_cleanup_bucket IS NULL
        OR char_length(media_object_cleanup_bucket) BETWEEN 1 AND 255
    ),
    CONSTRAINT media_object_cleanup_key_not_empty CHECK (
        media_object_cleanup_key IS NULL
        OR char_length(media_object_cleanup_key) BETWEEN 1 AND 1024
    ),
    CONSTRAINT media_object_cleanup_original_url_valid CHECK (
        char_length(media_object_cleanup_original_url) BETWEEN 1 AND 4096
    ),
    CONSTRAINT media_object_cleanup_reason_valid CHECK (
        media_object_cleanup_reason IN (
            'superseded_profile_picture',
            'profile_picture_history_pruned',
            'profile_picture_deleted',
            'deleted_photograph_image',
            'deleted_photograph_thumbnail',
            'deleted_wasm_thumbnail',
            'superseded_wasm_thumbnail'
        )
    ),
    CONSTRAINT media_object_cleanup_attempt_count_nonnegative CHECK (
        media_object_cleanup_attempt_count >= 0
    ),
    CONSTRAINT media_object_cleanup_attempt_state_coherent CHECK (
        (
            media_object_cleanup_attempt_count = 0
            AND media_object_cleanup_last_attempt_at IS NULL
            AND media_object_cleanup_last_error IS NULL
        )
        OR (
            media_object_cleanup_attempt_count > 0
            AND media_object_cleanup_last_attempt_at IS NOT NULL
        )
    ),
    CONSTRAINT media_object_cleanup_last_error_bounded CHECK (
        media_object_cleanup_last_error IS NULL
        OR char_length(media_object_cleanup_last_error) BETWEEN 1 AND 2048
    )
);

CREATE UNIQUE INDEX media_object_cleanup_resolved_object_unique
    ON public.media_object_cleanup (
        media_object_cleanup_bucket,
        media_object_cleanup_key
    )
    WHERE media_object_cleanup_bucket IS NOT NULL
        AND media_object_cleanup_key IS NOT NULL;

CREATE INDEX media_object_cleanup_retry_idx
    ON public.media_object_cleanup (
        media_object_cleanup_last_attempt_at ASC NULLS FIRST,
        media_object_cleanup_created_at ASC,
        media_object_cleanup_id ASC
    )
    WHERE media_object_cleanup_bucket IS NOT NULL
        AND media_object_cleanup_key IS NOT NULL;

CREATE INDEX media_object_cleanup_unresolved_idx
    ON public.media_object_cleanup (
        media_object_cleanup_created_at ASC,
        media_object_cleanup_id ASC
    )
    WHERE media_object_cleanup_bucket IS NULL
        AND media_object_cleanup_key IS NULL;

CREATE INDEX media_object_cleanup_source_idx
    ON public.media_object_cleanup (
        media_object_cleanup_source_id,
        media_object_cleanup_created_at
    );
