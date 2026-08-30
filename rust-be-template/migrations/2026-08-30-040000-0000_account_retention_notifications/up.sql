CREATE TYPE public.account_retention_notification_stage AS ENUM (
    'seven_days_before_purge',
    'one_day_before_purge'
);

CREATE TABLE public.account_retention_notifications (
    account_retention_notification_id UUID PRIMARY KEY DEFAULT uuidv7 (),
    account_retention_notification_user_id UUID NOT NULL,
    account_retention_notification_stage public.account_retention_notification_stage NOT NULL,
    account_retention_notification_scheduled_for TIMESTAMPTZ NOT NULL,
    account_retention_notification_next_attempt_at TIMESTAMPTZ NOT NULL,
    account_retention_notification_attempt_count INTEGER NOT NULL DEFAULT 0,
    account_retention_notification_claim_token UUID,
    account_retention_notification_claimed_at TIMESTAMPTZ,
    account_retention_notification_claim_expires_at TIMESTAMPTZ,
    account_retention_notification_sent_at TIMESTAMPTZ,
    account_retention_notification_cancelled_at TIMESTAMPTZ,
    account_retention_notification_last_error VARCHAR(512),
    account_retention_notification_created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    account_retention_notification_updated_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    CONSTRAINT account_retention_notifications_user_stage_unique
        UNIQUE (
            account_retention_notification_user_id,
            account_retention_notification_stage
        ),
    CONSTRAINT account_retention_notifications_user_fk
        FOREIGN KEY (account_retention_notification_user_id)
        REFERENCES public.users (user_id) ON DELETE RESTRICT,
    CONSTRAINT account_retention_notifications_attempt_count_nonnegative
        CHECK (account_retention_notification_attempt_count >= 0),
    CONSTRAINT account_retention_notifications_attempt_after_schedule
        CHECK (
            account_retention_notification_next_attempt_at
                >= account_retention_notification_scheduled_for
        ),
    CONSTRAINT account_retention_notifications_claim_fields_paired
        CHECK (
            (
                account_retention_notification_claim_token IS NULL
                AND account_retention_notification_claimed_at IS NULL
                AND account_retention_notification_claim_expires_at IS NULL
            )
            OR (
                account_retention_notification_claim_token IS NOT NULL
                AND account_retention_notification_claimed_at IS NOT NULL
                AND account_retention_notification_claim_expires_at IS NOT NULL
                AND account_retention_notification_claim_expires_at
                    > account_retention_notification_claimed_at
            )
        ),
    CONSTRAINT account_retention_notifications_sent_is_terminal
        CHECK (
            account_retention_notification_sent_at IS NULL
            OR (
                account_retention_notification_claim_token IS NULL
                AND account_retention_notification_claimed_at IS NULL
                AND account_retention_notification_claim_expires_at IS NULL
                AND account_retention_notification_last_error IS NULL
            )
        ),
    CONSTRAINT account_retention_notifications_terminal_state_exclusive
        CHECK (
            NOT (
                account_retention_notification_sent_at IS NOT NULL
                AND account_retention_notification_cancelled_at IS NOT NULL
            )
        ),
    CONSTRAINT account_retention_notifications_cancelled_is_unclaimed
        CHECK (
            account_retention_notification_cancelled_at IS NULL
            OR (
                account_retention_notification_claim_token IS NULL
                AND account_retention_notification_claimed_at IS NULL
                AND account_retention_notification_claim_expires_at IS NULL
            )
        ),
    CONSTRAINT account_retention_notifications_updated_after_created
        CHECK (
            account_retention_notification_updated_at
                >= account_retention_notification_created_at
        )
);

CREATE FUNCTION public.validate_account_retention_notification_insert()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    deleted_at TIMESTAMPTZ;
    purge_after TIMESTAMPTZ;
    hard_purged_at TIMESTAMPTZ;
    expected_schedule TIMESTAMPTZ;
BEGIN
    SELECT
        account.user_deleted_at,
        account.user_purge_after,
        account.user_hard_purged_at
    INTO deleted_at, purge_after, hard_purged_at
    FROM public.users AS account
    WHERE account.user_id = NEW.account_retention_notification_user_id;

    IF NOT FOUND
        OR deleted_at IS NULL
        OR purge_after IS NULL
        OR hard_purged_at IS NOT NULL
        OR NOT EXISTS (
            SELECT 1
            FROM public.deleted_account_retention AS retained
            WHERE retained.deleted_account_retention_user_id
                = NEW.account_retention_notification_user_id
        )
    THEN
        RAISE EXCEPTION USING
            ERRCODE = 'check_violation',
            MESSAGE = 'retention notifications require active private retention';
    END IF;

    expected_schedule := purge_after - CASE NEW.account_retention_notification_stage
        WHEN 'seven_days_before_purge' THEN INTERVAL '7 days'
        WHEN 'one_day_before_purge' THEN INTERVAL '1 day'
    END;
    IF NEW.account_retention_notification_scheduled_for
            IS DISTINCT FROM expected_schedule
        OR NEW.account_retention_notification_next_attempt_at
            IS DISTINCT FROM expected_schedule
        OR NEW.account_retention_notification_attempt_count <> 0
        OR NEW.account_retention_notification_claim_token IS NOT NULL
        OR NEW.account_retention_notification_claimed_at IS NOT NULL
        OR NEW.account_retention_notification_claim_expires_at IS NOT NULL
        OR NEW.account_retention_notification_sent_at IS NOT NULL
        OR NEW.account_retention_notification_last_error IS NOT NULL
    THEN
        RAISE EXCEPTION USING
            ERRCODE = 'check_violation',
            MESSAGE = 'retention notification initial state does not match its purge schedule';
    END IF;
    RETURN NEW;
END
$$;

CREATE TRIGGER account_retention_notifications_insert_guard
BEFORE INSERT ON public.account_retention_notifications
FOR EACH ROW
EXECUTE FUNCTION public.validate_account_retention_notification_insert();

CREATE INDEX account_retention_notifications_due_idx
    ON public.account_retention_notifications (
        account_retention_notification_next_attempt_at,
        account_retention_notification_id
    )
    WHERE account_retention_notification_sent_at IS NULL
        AND account_retention_notification_cancelled_at IS NULL;

CREATE INDEX account_retention_notifications_status_idx
    ON public.account_retention_notifications (
        account_retention_notification_next_attempt_at,
        account_retention_notification_id
    );

CREATE INDEX account_retention_notifications_claim_expiry_idx
    ON public.account_retention_notifications (
        account_retention_notification_claim_expires_at,
        account_retention_notification_id
    )
    WHERE account_retention_notification_claim_token IS NOT NULL;

CREATE FUNCTION public.protect_account_retention_notification_updates()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF NEW.account_retention_notification_id
            IS DISTINCT FROM OLD.account_retention_notification_id
        OR NEW.account_retention_notification_user_id
            IS DISTINCT FROM OLD.account_retention_notification_user_id
        OR NEW.account_retention_notification_stage
            IS DISTINCT FROM OLD.account_retention_notification_stage
        OR NEW.account_retention_notification_scheduled_for
            IS DISTINCT FROM OLD.account_retention_notification_scheduled_for
        OR NEW.account_retention_notification_created_at
            IS DISTINCT FROM OLD.account_retention_notification_created_at
        OR NEW.account_retention_notification_attempt_count
            < OLD.account_retention_notification_attempt_count
        OR (
            OLD.account_retention_notification_sent_at IS NOT NULL
            AND NEW IS DISTINCT FROM OLD
        )
        OR (
            OLD.account_retention_notification_cancelled_at IS NOT NULL
            AND NEW IS DISTINCT FROM OLD
        )
    THEN
        RAISE EXCEPTION USING
            ERRCODE = 'integrity_constraint_violation',
            MESSAGE = 'retention notification identity and terminal audit state are immutable';
    END IF;
    RETURN NEW;
END
$$;

CREATE TRIGGER account_retention_notifications_update_guard
BEFORE UPDATE ON public.account_retention_notifications
FOR EACH ROW
EXECUTE FUNCTION public.protect_account_retention_notification_updates();

CREATE FUNCTION public.protect_account_retention_notification_deletes()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION USING
        ERRCODE = 'integrity_constraint_violation',
        MESSAGE = 'retention notification audit rows cannot be deleted';
END
$$;

CREATE TRIGGER account_retention_notifications_delete_guard
BEFORE DELETE ON public.account_retention_notifications
FOR EACH ROW
EXECUTE FUNCTION public.protect_account_retention_notification_deletes();

INSERT INTO public.account_retention_notifications (
    account_retention_notification_user_id,
    account_retention_notification_stage,
    account_retention_notification_scheduled_for,
    account_retention_notification_next_attempt_at,
    account_retention_notification_cancelled_at,
    account_retention_notification_created_at,
    account_retention_notification_updated_at
)
SELECT
    retained.deleted_account_retention_user_id,
    schedule.stage,
    account.user_purge_after - schedule.notice_before,
    account.user_purge_after - schedule.notice_before,
    CASE WHEN account.user_purge_after <= now () THEN now () END,
    now (),
    now ()
FROM public.deleted_account_retention AS retained
INNER JOIN public.users AS account
    ON account.user_id = retained.deleted_account_retention_user_id
CROSS JOIN (
    VALUES
        ('seven_days_before_purge'::public.account_retention_notification_stage, INTERVAL '7 days'),
        ('one_day_before_purge'::public.account_retention_notification_stage, INTERVAL '1 day')
) AS schedule(stage, notice_before)
WHERE account.user_deleted_at IS NOT NULL
    AND account.user_purge_after IS NOT NULL
    AND account.user_hard_purged_at IS NULL;
