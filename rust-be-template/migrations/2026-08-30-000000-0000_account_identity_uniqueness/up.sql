-- Identity collisions are never resolved automatically. Merging or deleting an
-- account requires an operator decision, so this migration locks the table and
-- fails before changing the schema when existing exact values collide.
LOCK TABLE users IN ACCESS EXCLUSIVE MODE;

DO $$
BEGIN
    IF EXISTS (
        SELECT user_email
        FROM users
        GROUP BY user_email
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION USING
            ERRCODE = 'unique_violation',
            MESSAGE = 'cannot enforce users_user_email_unique: duplicate user_email values exist',
            HINT = 'Resolve the duplicate accounts explicitly, then rerun the migration.';
    END IF;

    IF EXISTS (
        SELECT user_name
        FROM users
        GROUP BY user_name
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION USING
            ERRCODE = 'unique_violation',
            MESSAGE = 'cannot enforce users_user_name_unique: duplicate user_name values exist',
            HINT = 'Resolve the duplicate accounts explicitly, then rerun the migration.';
    END IF;
END
$$;

DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_name;

ALTER TABLE users
    ADD CONSTRAINT users_user_email_unique UNIQUE (user_email),
    ADD CONSTRAINT users_user_name_unique UNIQUE (user_name);
