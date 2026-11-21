-- Accounts allowed to read the KCNA research archive. Superusers always may
-- and are not listed; an account that is not listed cannot read it.
CREATE TABLE kcna_archive_reader (
    kcna_archive_reader_user_id uuid PRIMARY KEY REFERENCES users (user_id) ON DELETE CASCADE,
    kcna_archive_reader_granted_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
