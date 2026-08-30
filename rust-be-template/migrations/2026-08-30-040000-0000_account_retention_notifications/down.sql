DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.account_retention_notifications) THEN
        RAISE EXCEPTION USING
            ERRCODE = 'object_not_in_prerequisite_state',
            MESSAGE = 'cannot roll back account retention notifications after scheduling delivery',
            HINT = 'Retain the notification ledger or restore from a backup predating account deletion.';
    END IF;
END
$$;

DROP TABLE public.account_retention_notifications;
DROP FUNCTION public.validate_account_retention_notification_insert();
DROP FUNCTION public.protect_account_retention_notification_updates();
DROP FUNCTION public.protect_account_retention_notification_deletes();
DROP TYPE public.account_retention_notification_stage;
