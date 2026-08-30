CREATE TABLE account_oidc_identities (
    account_oidc_identity_id UUID PRIMARY KEY DEFAULT uuidv7(),
    account_oidc_identity_user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE RESTRICT,
    account_oidc_identity_issuer VARCHAR(1024) NOT NULL,
    account_oidc_identity_subject VARCHAR(255) NOT NULL,
    account_oidc_identity_provider_email VARCHAR(254) NOT NULL,
    account_oidc_identity_created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    account_oidc_identity_updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    account_oidc_identity_last_authenticated_at TIMESTAMPTZ,
    CONSTRAINT account_oidc_identities_issuer_length_check
        CHECK (octet_length(account_oidc_identity_issuer) BETWEEN 1 AND 1024),
    CONSTRAINT account_oidc_identities_subject_length_check
        CHECK (octet_length(account_oidc_identity_subject) BETWEEN 1 AND 255),
    CONSTRAINT account_oidc_identities_provider_email_length_check
        CHECK (octet_length(account_oidc_identity_provider_email) BETWEEN 3 AND 254),
    CONSTRAINT account_oidc_identities_timestamp_order_check
        CHECK (
            account_oidc_identity_updated_at >= account_oidc_identity_created_at
            AND (
                account_oidc_identity_last_authenticated_at IS NULL
                OR account_oidc_identity_last_authenticated_at >= account_oidc_identity_created_at
            )
        ),
    CONSTRAINT account_oidc_identities_issuer_subject_unique
        UNIQUE (account_oidc_identity_issuer, account_oidc_identity_subject),
    CONSTRAINT account_oidc_identities_user_issuer_unique
        UNIQUE (account_oidc_identity_user_id, account_oidc_identity_issuer)
);

CREATE INDEX account_oidc_identities_created_at_idx
    ON account_oidc_identities (account_oidc_identity_created_at);

CREATE INDEX account_oidc_identities_last_authenticated_at_idx
    ON account_oidc_identities (account_oidc_identity_last_authenticated_at)
    WHERE account_oidc_identity_last_authenticated_at IS NOT NULL;
