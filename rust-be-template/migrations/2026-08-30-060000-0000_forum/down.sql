DO $$
DECLARE
    forum_permission_id CONSTANT UUID := '019d0000-0000-7000-8000-000000000009';
    younghyun_role_id CONSTANT UUID := '019a6c86-8bca-7b91-b9c0-1d4cc96b3263';
BEGIN
    IF EXISTS (SELECT 1 FROM public.forum_topics)
        OR EXISTS (SELECT 1 FROM public.forum_replies)
        OR EXISTS (SELECT 1 FROM public.forum_topic_subscriptions)
        OR EXISTS (SELECT 1 FROM public.forum_notifications)
        OR EXISTS (SELECT 1 FROM public.forum_moderation_audit_events)
    THEN
        RAISE EXCEPTION USING
            ERRCODE = 'object_not_in_prerequisite_state',
            MESSAGE = 'cannot roll back forum while forum content or audit state exists';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM public.permissions
        WHERE permission_id = forum_permission_id
          AND permission_name = 'forum.moderate'
          AND permission_description = 'Hide and restore forum content, and lock or pin forum topics.'
    ) OR (SELECT count(*) FROM public.role_permissions WHERE permission_id = forum_permission_id) <> 1
      OR NOT EXISTS (
          SELECT 1 FROM public.role_permissions
          WHERE permission_id = forum_permission_id AND role_id = younghyun_role_id
      )
      OR EXISTS (
          SELECT 1 FROM public.authorization_audit_events
          WHERE authorization_audit_event_permission_id = forum_permission_id
      )
    THEN
        RAISE EXCEPTION USING
            ERRCODE = 'object_not_in_prerequisite_state',
            MESSAGE = 'cannot remove modified or audited forum permission seed';
    END IF;
END
$$;

DROP TRIGGER forum_moderation_audit_update_delete_guard ON public.forum_moderation_audit_events;
DROP TRIGGER forum_moderation_audit_truncate_guard ON public.forum_moderation_audit_events;
DROP FUNCTION public.reject_forum_moderation_audit_mutation();
DROP TABLE public.forum_moderation_audit_events;
DROP TABLE public.forum_notifications;
DROP TABLE public.forum_topic_subscriptions;
DROP TABLE public.forum_replies;
DROP TABLE public.forum_topics;
DROP FUNCTION public.forum_websearch_to_tsquery(TEXT);
DROP TYPE public.forum_notification_kind;
DROP TYPE public.forum_moderation_action;
DROP TYPE public.forum_topic_access_state;
DROP TYPE public.forum_content_state;

DELETE FROM public.role_permissions
WHERE permission_id = '019d0000-0000-7000-8000-000000000009';
DELETE FROM public.permissions
WHERE permission_id = '019d0000-0000-7000-8000-000000000009';
