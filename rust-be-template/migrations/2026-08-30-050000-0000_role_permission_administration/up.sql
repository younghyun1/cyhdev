ALTER TABLE public.permissions
    ADD CONSTRAINT permissions_name_format CHECK (
        char_length(permission_name) BETWEEN 3 AND 64
        AND permission_name ~ '^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$'
    );

CREATE INDEX authorization_users_name_prefix_idx
    ON public.users (user_name varchar_pattern_ops, user_id)
    WHERE user_deleted_at IS NULL
        AND user_hard_purged_at IS NULL
        AND NOT user_is_system_actor;
CREATE INDEX authorization_users_email_prefix_idx
    ON public.users (user_email varchar_pattern_ops, user_id)
    WHERE user_deleted_at IS NULL
        AND user_hard_purged_at IS NULL
        AND NOT user_is_system_actor;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.permissions
        WHERE permission_id IN (
            '019d0000-0000-7000-8000-000000000001'::uuid,
            '019d0000-0000-7000-8000-000000000002'::uuid,
            '019d0000-0000-7000-8000-000000000003'::uuid,
            '019d0000-0000-7000-8000-000000000004'::uuid,
            '019d0000-0000-7000-8000-000000000005'::uuid,
            '019d0000-0000-7000-8000-000000000006'::uuid,
            '019d0000-0000-7000-8000-000000000007'::uuid,
            '019d0000-0000-7000-8000-000000000008'::uuid
        )
        OR permission_name IN (
            'authorization.roles.manage',
            'account.lifecycle.manage',
            'media.cleanup.manage',
            'content.blog.manage',
            'content.photography.manage',
            'content.wasm.manage',
            'chat.moderate',
            'i18n.manage'
        )
    ) THEN
        RAISE EXCEPTION USING
            ERRCODE = 'unique_violation',
            MESSAGE = 'cannot install authorization catalog over preexisting seeded permission identifiers or names';
    END IF;
END
$$;

INSERT INTO public.permissions (permission_id, permission_name, permission_description)
VALUES
    ('019d0000-0000-7000-8000-000000000001', 'authorization.roles.manage', 'Assign account roles and change role-permission bindings.'),
    ('019d0000-0000-7000-8000-000000000002', 'account.lifecycle.manage', 'Review and complete privileged account lifecycle operations.'),
    ('019d0000-0000-7000-8000-000000000003', 'media.cleanup.manage', 'Review and reconcile retained object-storage cleanup work.'),
    ('019d0000-0000-7000-8000-000000000004', 'content.blog.manage', 'Create, update, publish, and moderate blog content.'),
    ('019d0000-0000-7000-8000-000000000005', 'content.photography.manage', 'Upload, update, and moderate photography content.'),
    ('019d0000-0000-7000-8000-000000000006', 'content.wasm.manage', 'Upload, update, and remove WebAssembly projects.'),
    ('019d0000-0000-7000-8000-000000000007', 'chat.moderate', 'Moderate live-chat participants and retained messages.'),
    ('019d0000-0000-7000-8000-000000000008', 'i18n.manage', 'Synchronize and administer localized interface text.');

INSERT INTO public.role_permissions (role_id, permission_id)
SELECT
    '019a6c86-8bca-7b91-b9c0-1d4cc96b3263'::uuid,
    permission_id
FROM public.permissions
WHERE permission_id IN (
    '019d0000-0000-7000-8000-000000000001'::uuid,
    '019d0000-0000-7000-8000-000000000002'::uuid,
    '019d0000-0000-7000-8000-000000000003'::uuid,
    '019d0000-0000-7000-8000-000000000004'::uuid,
    '019d0000-0000-7000-8000-000000000005'::uuid,
    '019d0000-0000-7000-8000-000000000006'::uuid,
    '019d0000-0000-7000-8000-000000000007'::uuid,
    '019d0000-0000-7000-8000-000000000008'::uuid
);

CREATE TYPE public.authorization_audit_kind AS ENUM (
    'user_role_assigned',
    'role_permission_granted',
    'role_permission_revoked'
);

CREATE TABLE public.authorization_audit_events (
    authorization_audit_event_id UUID PRIMARY KEY DEFAULT uuidv7(),
    authorization_audit_event_actor_user_id UUID NOT NULL
        REFERENCES public.users (user_id) ON DELETE RESTRICT,
    authorization_audit_event_kind public.authorization_audit_kind NOT NULL,
    authorization_audit_event_target_user_id UUID
        REFERENCES public.users (user_id) ON DELETE RESTRICT,
    authorization_audit_event_role_id UUID NOT NULL,
    authorization_audit_event_role_name VARCHAR(64) NOT NULL,
    authorization_audit_event_permission_id UUID,
    authorization_audit_event_permission_name VARCHAR(64),
    authorization_audit_event_old_value VARCHAR(128) NOT NULL,
    authorization_audit_event_new_value VARCHAR(128) NOT NULL,
    authorization_audit_event_reason VARCHAR(500) NOT NULL,
    authorization_audit_event_request_id UUID,
    authorization_audit_event_created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT authorization_audit_event_reason_length CHECK (
        char_length(authorization_audit_event_reason) BETWEEN 8 AND 500
        AND authorization_audit_event_reason = btrim(authorization_audit_event_reason)
    ),
    CONSTRAINT authorization_audit_event_permission_pair CHECK (
        (authorization_audit_event_permission_id IS NULL)
        = (authorization_audit_event_permission_name IS NULL)
    ),
    CONSTRAINT authorization_audit_event_shape CHECK (
        (
            authorization_audit_event_kind = 'user_role_assigned'
            AND authorization_audit_event_target_user_id IS NOT NULL
            AND authorization_audit_event_permission_id IS NULL
        )
        OR (
            authorization_audit_event_kind IN (
                'role_permission_granted',
                'role_permission_revoked'
            )
            AND authorization_audit_event_target_user_id IS NULL
            AND authorization_audit_event_permission_id IS NOT NULL
        )
    )
);

-- User UUIDs resolve through permanent masked tombstones after account purge. Role
-- and permission names remain snapshots because catalog entries may be retired.
CREATE INDEX authorization_audit_events_created_page_idx
    ON public.authorization_audit_events (
        authorization_audit_event_created_at DESC,
        authorization_audit_event_id DESC
    );
CREATE INDEX authorization_audit_events_actor_idx
    ON public.authorization_audit_events (
        authorization_audit_event_actor_user_id,
        authorization_audit_event_id DESC
    );
CREATE INDEX authorization_audit_events_target_idx
    ON public.authorization_audit_events (
        authorization_audit_event_target_user_id,
        authorization_audit_event_id DESC
    )
    WHERE authorization_audit_event_target_user_id IS NOT NULL;
CREATE INDEX authorization_audit_events_role_idx
    ON public.authorization_audit_events (
        authorization_audit_event_role_id,
        authorization_audit_event_id DESC
    );
CREATE INDEX authorization_audit_events_permission_idx
    ON public.authorization_audit_events (
        authorization_audit_event_permission_id,
        authorization_audit_event_id DESC
    )
    WHERE authorization_audit_event_permission_id IS NOT NULL;

CREATE FUNCTION public.reject_authorization_audit_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION USING
        ERRCODE = 'integrity_constraint_violation',
        MESSAGE = 'authorization audit events are append-only';
END
$$;

CREATE TRIGGER authorization_audit_events_update_delete_guard
BEFORE UPDATE OR DELETE ON public.authorization_audit_events
FOR EACH ROW
EXECUTE FUNCTION public.reject_authorization_audit_mutation();

CREATE TRIGGER authorization_audit_events_truncate_guard
BEFORE TRUNCATE ON public.authorization_audit_events
FOR EACH STATEMENT
EXECUTE FUNCTION public.reject_authorization_audit_mutation();
