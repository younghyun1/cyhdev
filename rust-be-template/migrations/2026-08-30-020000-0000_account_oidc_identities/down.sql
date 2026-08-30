DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM account_oidc_identities LIMIT 1) THEN
        RAISE EXCEPTION
            'cannot remove account_oidc_identities while linked identities exist';
    END IF;
END
$$;

DROP TABLE account_oidc_identities;
