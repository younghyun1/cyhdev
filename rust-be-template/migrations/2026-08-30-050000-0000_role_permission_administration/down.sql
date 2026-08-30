DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.authorization_audit_events) THEN
        RAISE EXCEPTION USING
            ERRCODE = 'object_not_in_prerequisite_state',
            MESSAGE = 'cannot roll back authorization administration with audit events';
    END IF;
END
$$;

DROP TRIGGER IF EXISTS authorization_audit_events_update_delete_guard
    ON public.authorization_audit_events;
DROP TRIGGER IF EXISTS authorization_audit_events_truncate_guard
    ON public.authorization_audit_events;
DROP FUNCTION IF EXISTS public.reject_authorization_audit_mutation();
DROP TABLE IF EXISTS public.authorization_audit_events;
DROP TYPE IF EXISTS public.authorization_audit_kind;

DROP INDEX IF EXISTS public.authorization_users_email_prefix_idx;
DROP INDEX IF EXISTS public.authorization_users_name_prefix_idx;

DELETE FROM public.permissions
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

ALTER TABLE public.permissions
    DROP CONSTRAINT IF EXISTS permissions_name_format;
