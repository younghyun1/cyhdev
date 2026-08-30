DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM public.permissions
        WHERE permission_id = '019d0000-0000-7000-8000-000000000009'::uuid
           OR permission_name = 'forum.moderate'
    ) THEN
        RAISE EXCEPTION USING
            ERRCODE = 'unique_violation',
            MESSAGE = 'cannot install forum permission over a preexisting identifier or name';
    END IF;
END
$$;

INSERT INTO public.permissions (permission_id, permission_name, permission_description)
VALUES (
    '019d0000-0000-7000-8000-000000000009',
    'forum.moderate',
    'Hide and restore forum content, and lock or pin forum topics.'
);

INSERT INTO public.role_permissions (role_id, permission_id)
VALUES (
    '019a6c86-8bca-7b91-b9c0-1d4cc96b3263',
    '019d0000-0000-7000-8000-000000000009'
);

CREATE TYPE public.forum_content_state AS ENUM ('visible', 'hidden', 'deleted');
CREATE TYPE public.forum_topic_access_state AS ENUM ('open', 'locked');
CREATE TYPE public.forum_moderation_action AS ENUM (
    'topic_hidden',
    'topic_restored',
    'topic_locked',
    'topic_unlocked',
    'topic_pinned',
    'topic_unpinned',
    'reply_hidden',
    'reply_restored'
);
CREATE TYPE public.forum_notification_kind AS ENUM ('topic_reply');

CREATE FUNCTION public.forum_websearch_to_tsquery(query_text TEXT)
RETURNS TSQUERY
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
AS $$
    SELECT websearch_to_tsquery('simple'::regconfig, query_text)
$$;

CREATE TABLE public.forum_topics (
    forum_topic_id UUID PRIMARY KEY DEFAULT uuidv7(),
    forum_topic_author_user_id UUID NOT NULL
        REFERENCES public.users(user_id) ON DELETE RESTRICT,
    forum_topic_title VARCHAR(512) NOT NULL,
    forum_topic_body TEXT NOT NULL,
    forum_topic_content_state public.forum_content_state NOT NULL DEFAULT 'visible',
    forum_topic_access_state public.forum_topic_access_state NOT NULL DEFAULT 'open',
    forum_topic_is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
    forum_topic_revision INTEGER NOT NULL DEFAULT 1,
    -- Total retained reply rows, including stable hidden/deleted tombstones.
    forum_topic_reply_count BIGINT NOT NULL DEFAULT 0,
    forum_topic_created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    forum_topic_updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    forum_topic_last_activity_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    forum_topic_edited_at TIMESTAMPTZ,
    forum_topic_hidden_at TIMESTAMPTZ,
    forum_topic_deleted_at TIMESTAMPTZ,
    forum_topic_search_vector TSVECTOR GENERATED ALWAYS AS (
        setweight(to_tsvector('simple'::regconfig, forum_topic_title), 'A')
        || setweight(to_tsvector('simple'::regconfig, forum_topic_body), 'B')
    ) STORED,
    CONSTRAINT forum_topic_title_bounds CHECK (
        char_length(forum_topic_title) BETWEEN 3 AND 160
        AND octet_length(forum_topic_title) <= 512
        AND forum_topic_title = btrim(forum_topic_title)
    ),
    CONSTRAINT forum_topic_body_bounds CHECK (
        char_length(forum_topic_body) BETWEEN 1 AND 20000
        AND octet_length(forum_topic_body) <= 65536
        AND forum_topic_body = btrim(forum_topic_body)
    ),
    CONSTRAINT forum_topic_reply_count_nonnegative CHECK (forum_topic_reply_count >= 0),
    CONSTRAINT forum_topic_revision_positive CHECK (forum_topic_revision >= 1),
    CONSTRAINT forum_topic_timestamp_order CHECK (
        forum_topic_updated_at >= forum_topic_created_at
        AND forum_topic_last_activity_at >= forum_topic_created_at
        AND (forum_topic_edited_at IS NULL OR forum_topic_edited_at >= forum_topic_created_at)
        AND (forum_topic_hidden_at IS NULL OR forum_topic_hidden_at >= forum_topic_created_at)
        AND (forum_topic_deleted_at IS NULL OR forum_topic_deleted_at >= forum_topic_created_at)
    ),
    CONSTRAINT forum_topic_content_state_shape CHECK (
        (forum_topic_content_state = 'visible' AND forum_topic_hidden_at IS NULL AND forum_topic_deleted_at IS NULL)
        OR (forum_topic_content_state = 'hidden' AND forum_topic_hidden_at IS NOT NULL AND forum_topic_deleted_at IS NULL)
        OR (forum_topic_content_state = 'deleted' AND forum_topic_hidden_at IS NULL AND forum_topic_deleted_at IS NOT NULL)
    )
);

CREATE INDEX forum_topics_author_idx
    ON public.forum_topics (forum_topic_author_user_id, forum_topic_created_at DESC);
CREATE INDEX forum_topics_public_page_idx
    ON public.forum_topics (
        forum_topic_is_pinned DESC,
        forum_topic_last_activity_at DESC,
        forum_topic_id DESC
    );
CREATE INDEX forum_topics_visible_recent_idx
    ON public.forum_topics (forum_topic_last_activity_at DESC, forum_topic_id DESC)
    WHERE forum_topic_content_state = 'visible';
CREATE INDEX forum_topics_search_gin_idx
    ON public.forum_topics USING GIN (forum_topic_search_vector)
    WHERE forum_topic_content_state = 'visible';

CREATE TABLE public.forum_replies (
    forum_reply_id UUID PRIMARY KEY DEFAULT uuidv7(),
    forum_reply_topic_id UUID NOT NULL
        REFERENCES public.forum_topics(forum_topic_id) ON DELETE RESTRICT,
    forum_reply_author_user_id UUID NOT NULL
        REFERENCES public.users(user_id) ON DELETE RESTRICT,
    forum_reply_body TEXT NOT NULL,
    forum_reply_content_state public.forum_content_state NOT NULL DEFAULT 'visible',
    forum_reply_revision INTEGER NOT NULL DEFAULT 1,
    forum_reply_created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    forum_reply_updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    forum_reply_edited_at TIMESTAMPTZ,
    forum_reply_hidden_at TIMESTAMPTZ,
    forum_reply_deleted_at TIMESTAMPTZ,
    CONSTRAINT forum_reply_body_bounds CHECK (
        char_length(forum_reply_body) BETWEEN 1 AND 20000
        AND octet_length(forum_reply_body) <= 65536
        AND forum_reply_body = btrim(forum_reply_body)
    ),
    CONSTRAINT forum_reply_revision_positive CHECK (forum_reply_revision >= 1),
    CONSTRAINT forum_reply_timestamp_order CHECK (
        forum_reply_updated_at >= forum_reply_created_at
        AND (forum_reply_edited_at IS NULL OR forum_reply_edited_at >= forum_reply_created_at)
        AND (forum_reply_hidden_at IS NULL OR forum_reply_hidden_at >= forum_reply_created_at)
        AND (forum_reply_deleted_at IS NULL OR forum_reply_deleted_at >= forum_reply_created_at)
    ),
    CONSTRAINT forum_reply_content_state_shape CHECK (
        (forum_reply_content_state = 'visible' AND forum_reply_hidden_at IS NULL AND forum_reply_deleted_at IS NULL)
        OR (forum_reply_content_state = 'hidden' AND forum_reply_hidden_at IS NOT NULL AND forum_reply_deleted_at IS NULL)
        OR (forum_reply_content_state = 'deleted' AND forum_reply_hidden_at IS NULL AND forum_reply_deleted_at IS NOT NULL)
    ),
    CONSTRAINT forum_replies_id_topic_unique UNIQUE (forum_reply_id, forum_reply_topic_id)
);

CREATE INDEX forum_replies_topic_page_idx
    ON public.forum_replies (forum_reply_topic_id, forum_reply_created_at ASC, forum_reply_id ASC);
CREATE INDEX forum_replies_author_idx
    ON public.forum_replies (forum_reply_author_user_id, forum_reply_created_at DESC);

CREATE TABLE public.forum_topic_subscriptions (
    forum_topic_subscription_id UUID PRIMARY KEY DEFAULT uuidv7(),
    forum_topic_subscription_topic_id UUID NOT NULL
        REFERENCES public.forum_topics(forum_topic_id) ON DELETE RESTRICT,
    forum_topic_subscription_user_id UUID NOT NULL
        REFERENCES public.users(user_id) ON DELETE RESTRICT,
    forum_topic_subscription_created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT forum_topic_subscriptions_topic_user_unique UNIQUE (
        forum_topic_subscription_topic_id,
        forum_topic_subscription_user_id
    )
);

CREATE INDEX forum_topic_subscriptions_user_idx
    ON public.forum_topic_subscriptions (forum_topic_subscription_user_id, forum_topic_subscription_id);

CREATE TABLE public.forum_notifications (
    forum_notification_id UUID PRIMARY KEY DEFAULT uuidv7(),
    forum_notification_recipient_user_id UUID NOT NULL
        REFERENCES public.users(user_id) ON DELETE RESTRICT,
    forum_notification_actor_user_id UUID NOT NULL
        REFERENCES public.users(user_id) ON DELETE RESTRICT,
    forum_notification_topic_id UUID NOT NULL
        REFERENCES public.forum_topics(forum_topic_id) ON DELETE RESTRICT,
    forum_notification_reply_id UUID NOT NULL,
    forum_notification_kind public.forum_notification_kind NOT NULL,
    forum_notification_created_at TIMESTAMPTZ NOT NULL,
    forum_notification_expires_at TIMESTAMPTZ NOT NULL,
    forum_notification_read_at TIMESTAMPTZ,
    CONSTRAINT forum_notifications_recipient_reply_unique UNIQUE (
        forum_notification_recipient_user_id,
        forum_notification_reply_id
    ),
    CONSTRAINT forum_notification_actor_not_recipient CHECK (
        forum_notification_actor_user_id <> forum_notification_recipient_user_id
    ),
    CONSTRAINT forum_notification_expiry CHECK (
        forum_notification_expires_at = forum_notification_created_at + INTERVAL '90 days'
        AND (forum_notification_read_at IS NULL OR forum_notification_read_at >= forum_notification_created_at)
    ),
    CONSTRAINT forum_notifications_reply_topic_fkey FOREIGN KEY (
        forum_notification_reply_id,
        forum_notification_topic_id
    ) REFERENCES public.forum_replies (
        forum_reply_id,
        forum_reply_topic_id
    ) ON DELETE RESTRICT
);

CREATE INDEX forum_notifications_recipient_page_idx
    ON public.forum_notifications (
        forum_notification_recipient_user_id,
        forum_notification_created_at DESC,
        forum_notification_id DESC
    );
CREATE INDEX forum_notifications_unread_idx
    ON public.forum_notifications (
        forum_notification_recipient_user_id,
        forum_notification_created_at DESC,
        forum_notification_id DESC
    ) WHERE forum_notification_read_at IS NULL;
CREATE INDEX forum_notifications_expiry_idx
    ON public.forum_notifications (forum_notification_expires_at ASC, forum_notification_id ASC);
CREATE INDEX forum_notifications_actor_idx
    ON public.forum_notifications (forum_notification_actor_user_id, forum_notification_id DESC);
CREATE INDEX forum_notifications_topic_idx
    ON public.forum_notifications (forum_notification_topic_id, forum_notification_id DESC);
CREATE INDEX forum_notifications_reply_idx
    ON public.forum_notifications (forum_notification_reply_id, forum_notification_id DESC);

CREATE TABLE public.forum_moderation_audit_events (
    forum_moderation_audit_event_id UUID PRIMARY KEY DEFAULT uuidv7(),
    forum_moderation_audit_event_actor_user_id UUID NOT NULL
        REFERENCES public.users(user_id) ON DELETE RESTRICT,
    forum_moderation_audit_event_topic_id UUID
        REFERENCES public.forum_topics(forum_topic_id) ON DELETE RESTRICT,
    forum_moderation_audit_event_reply_id UUID
        REFERENCES public.forum_replies(forum_reply_id) ON DELETE RESTRICT,
    forum_moderation_audit_event_action public.forum_moderation_action NOT NULL,
    forum_moderation_audit_event_reason VARCHAR(2000) NOT NULL,
    forum_moderation_audit_event_request_id UUID,
    forum_moderation_audit_event_created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT forum_moderation_audit_target_shape CHECK (
        (forum_moderation_audit_event_topic_id IS NOT NULL)
        <> (forum_moderation_audit_event_reply_id IS NOT NULL)
    ),
    CONSTRAINT forum_moderation_audit_action_target CHECK (
        (
            forum_moderation_audit_event_topic_id IS NOT NULL
            AND forum_moderation_audit_event_action IN (
                'topic_hidden', 'topic_restored', 'topic_locked',
                'topic_unlocked', 'topic_pinned', 'topic_unpinned'
            )
        ) OR (
            forum_moderation_audit_event_reply_id IS NOT NULL
            AND forum_moderation_audit_event_action IN ('reply_hidden', 'reply_restored')
        )
    ),
    CONSTRAINT forum_moderation_audit_reason_bounds CHECK (
        char_length(forum_moderation_audit_event_reason) BETWEEN 8 AND 500
        AND octet_length(forum_moderation_audit_event_reason) <= 2000
        AND forum_moderation_audit_event_reason = btrim(forum_moderation_audit_event_reason)
    )
);

CREATE INDEX forum_moderation_audit_page_idx
    ON public.forum_moderation_audit_events (
        forum_moderation_audit_event_created_at DESC,
        forum_moderation_audit_event_id DESC
    );
CREATE INDEX forum_moderation_audit_actor_idx
    ON public.forum_moderation_audit_events (
        forum_moderation_audit_event_actor_user_id,
        forum_moderation_audit_event_id DESC
    );
CREATE INDEX forum_moderation_audit_topic_idx
    ON public.forum_moderation_audit_events (
        forum_moderation_audit_event_topic_id,
        forum_moderation_audit_event_id DESC
    ) WHERE forum_moderation_audit_event_topic_id IS NOT NULL;
CREATE INDEX forum_moderation_audit_reply_idx
    ON public.forum_moderation_audit_events (
        forum_moderation_audit_event_reply_id,
        forum_moderation_audit_event_id DESC
    ) WHERE forum_moderation_audit_event_reply_id IS NOT NULL;
CREATE INDEX forum_moderation_audit_request_idx
    ON public.forum_moderation_audit_events (
        forum_moderation_audit_event_request_id,
        forum_moderation_audit_event_id DESC
    ) WHERE forum_moderation_audit_event_request_id IS NOT NULL;

CREATE FUNCTION public.reject_forum_moderation_audit_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION USING
        ERRCODE = 'integrity_constraint_violation',
        MESSAGE = 'forum moderation audit events are append-only';
END
$$;

CREATE TRIGGER forum_moderation_audit_update_delete_guard
BEFORE UPDATE OR DELETE ON public.forum_moderation_audit_events
FOR EACH ROW EXECUTE FUNCTION public.reject_forum_moderation_audit_mutation();

CREATE TRIGGER forum_moderation_audit_truncate_guard
BEFORE TRUNCATE ON public.forum_moderation_audit_events
FOR EACH STATEMENT EXECUTE FUNCTION public.reject_forum_moderation_audit_mutation();
