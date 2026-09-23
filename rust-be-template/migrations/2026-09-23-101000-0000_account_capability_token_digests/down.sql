-- A digest cannot be turned back into the emailed token. Rolling back gives every row a
-- fresh random UUID, which invalidates outstanding reset and verification links; those
-- capabilities are short-lived and can be requested again.
ALTER TABLE email_verification_tokens
    ADD COLUMN email_verification_token UUID NOT NULL DEFAULT gen_random_uuid();
ALTER TABLE email_verification_tokens
    ALTER COLUMN email_verification_token DROP DEFAULT,
    DROP CONSTRAINT IF EXISTS email_verification_tokens_hash_unique,
    DROP CONSTRAINT IF EXISTS email_verification_tokens_hash_length,
    DROP COLUMN email_verification_token_hash;
CREATE INDEX idx_email_verification_tokens_token
    ON email_verification_tokens (email_verification_token);

ALTER TABLE password_reset_tokens
    ADD COLUMN password_reset_token UUID NOT NULL DEFAULT gen_random_uuid();
ALTER TABLE password_reset_tokens
    ALTER COLUMN password_reset_token DROP DEFAULT,
    DROP CONSTRAINT IF EXISTS password_reset_tokens_hash_unique,
    DROP CONSTRAINT IF EXISTS password_reset_tokens_hash_length,
    DROP COLUMN password_reset_token_hash;
CREATE INDEX idx_password_reset_tokens_token
    ON password_reset_tokens (password_reset_token);
