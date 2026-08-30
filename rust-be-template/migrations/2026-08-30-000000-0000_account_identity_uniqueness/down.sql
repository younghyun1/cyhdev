LOCK TABLE users IN ACCESS EXCLUSIVE MODE;

ALTER TABLE users
    DROP CONSTRAINT IF EXISTS users_user_email_unique,
    DROP CONSTRAINT IF EXISTS users_user_name_unique;

CREATE INDEX IF NOT EXISTS idx_users_email ON users (user_email);
CREATE INDEX IF NOT EXISTS idx_users_name ON users (user_name);
