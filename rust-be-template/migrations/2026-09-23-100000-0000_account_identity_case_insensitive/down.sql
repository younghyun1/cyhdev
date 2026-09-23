-- Case-insensitive uniqueness implies exact uniqueness, so restoring the exact constraints
-- cannot fail on data written while this migration was applied.
LOCK TABLE users IN ACCESS EXCLUSIVE MODE;

DROP INDEX IF EXISTS users_user_email_lower_unique;
DROP INDEX IF EXISTS users_user_name_lower_unique;

ALTER TABLE users
    ADD CONSTRAINT users_user_email_unique UNIQUE (user_email),
    ADD CONSTRAINT users_user_name_unique UNIQUE (user_name);
