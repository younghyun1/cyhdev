-- Reset and verification tokens are stored as the SHA-256 of their emailed text, so a
-- leaked table or backup cannot be replayed as a working link. Each existing row is
-- converted by hashing its UUID text. The application accepts only the new 43-character
-- tokens, so links emailed before this migration stop working; they expire within 30
-- minutes (reset) or 24 hours (verification) anyway, and a new link can be requested.
ALTER TABLE email_verification_tokens
    ADD COLUMN email_verification_token_hash BYTEA;
UPDATE email_verification_tokens
SET email_verification_token_hash = sha256(convert_to(email_verification_token::text, 'UTF8'));
ALTER TABLE email_verification_tokens
    ALTER COLUMN email_verification_token_hash SET NOT NULL,
    ADD CONSTRAINT email_verification_tokens_hash_length
        CHECK (octet_length(email_verification_token_hash) = 32),
    ADD CONSTRAINT email_verification_tokens_hash_unique UNIQUE (email_verification_token_hash);
DROP INDEX IF EXISTS idx_email_verification_tokens_token;
ALTER TABLE email_verification_tokens DROP COLUMN email_verification_token;

ALTER TABLE password_reset_tokens
    ADD COLUMN password_reset_token_hash BYTEA;
UPDATE password_reset_tokens
SET password_reset_token_hash = sha256(convert_to(password_reset_token::text, 'UTF8'));
ALTER TABLE password_reset_tokens
    ALTER COLUMN password_reset_token_hash SET NOT NULL,
    ADD CONSTRAINT password_reset_tokens_hash_length
        CHECK (octet_length(password_reset_token_hash) = 32),
    ADD CONSTRAINT password_reset_tokens_hash_unique UNIQUE (password_reset_token_hash);
DROP INDEX IF EXISTS idx_password_reset_tokens_token;
ALTER TABLE password_reset_tokens DROP COLUMN password_reset_token;
