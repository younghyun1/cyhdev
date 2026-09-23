-- Email addresses and user names become unique without regard to letter case. User names
-- keep the case a person typed for display; only their uniqueness ignores it. Existing rows
-- are never rewritten, merged, or deleted: when two accounts already differ only by case,
-- an operator must choose how to resolve them, so this migration stops before any change.
LOCK TABLE users IN ACCESS EXCLUSIVE MODE;

DO $$
BEGIN
    IF EXISTS (
        SELECT lower(user_email)
        FROM users
        GROUP BY lower(user_email)
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION USING
            ERRCODE = 'unique_violation',
            MESSAGE = 'cannot enforce users_user_email_lower_unique: user_email values collide when letter case is ignored',
            HINT = 'Find them with SELECT lower(user_email), array_agg(user_id) FROM users GROUP BY 1 HAVING count(*) > 1, resolve each account explicitly, then rerun the migration.';
    END IF;

    IF EXISTS (
        SELECT lower(user_name)
        FROM users
        GROUP BY lower(user_name)
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION USING
            ERRCODE = 'unique_violation',
            MESSAGE = 'cannot enforce users_user_name_lower_unique: user_name values collide when letter case is ignored',
            HINT = 'Find them with SELECT lower(user_name), array_agg(user_id) FROM users GROUP BY 1 HAVING count(*) > 1, resolve each account explicitly, then rerun the migration.';
    END IF;
END
$$;

-- The case-insensitive indexes imply exact uniqueness, and every lookup now compares
-- lower() values, so the exact constraints would only add write amplification.
ALTER TABLE users
    DROP CONSTRAINT users_user_email_unique,
    DROP CONSTRAINT users_user_name_unique;

CREATE UNIQUE INDEX users_user_email_lower_unique ON users (lower(user_email));
CREATE UNIQUE INDEX users_user_name_lower_unique ON users (lower(user_name));
