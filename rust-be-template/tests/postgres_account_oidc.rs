mod support;

use rust_be_template::features::accounts::{domain::oidc::OidcIdentityClaims, error::AccountError};

use support::{
    database::{
        BoxError, DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test,
    },
    fixtures::{VALID_PASSWORD, account_test_context, seed_account},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn oidc_requires_explicit_subject_link_and_enforces_unique_ownership() -> TestResult {
    run_database_test(oidc_identity_case).await
}

fn oidc_identity_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let first = seed_account(&context, "OidcFirst").await?;
        let second = seed_account(&context, "OidcSecond").await?;
        context
            .accounts
            .verify_email(first.verification_token)
            .await?;
        context
            .accounts
            .verify_email(second.verification_token)
            .await?;
        let identity = OidcIdentityClaims {
            issuer: "https://id.example.test".to_owned(),
            subject: "provider-subject-1".to_owned(),
            provider_email: first.email.clone(),
        };

        require(
            context
                .repository
                .oidc_account_for_login(&identity)
                .await?
                .is_none(),
            "provider email implicitly linked an unlinked local account",
        )?;
        context
            .repository
            .link_oidc_identity(first.user_id, &identity)
            .await?;
        let resolved = context
            .repository
            .oidc_account_for_login(&identity)
            .await?
            .ok_or_else(|| Box::new(AccountError::OidcIdentityNotLinked) as BoxError)?;
        require(
            resolved.user_id == first.user_id,
            "issuer-subject link resolved the wrong local account",
        )?;
        match context
            .repository
            .link_oidc_identity(second.user_id, &identity)
            .await
        {
            Err(AccountError::OidcIdentityConflict(_)) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "one provider identity linked to two accounts"),
        }

        let login = context
            .accounts
            .login(&first.email, VALID_PASSWORD, None)
            .await?;
        match context
            .accounts
            .unlink_oidc(
                first.user_id,
                &identity.issuer,
                "WrongPass123",
                Some(login.session_token.expose()),
            )
            .await
        {
            Err(AccountError::WrongPassword) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "wrong local password unlinked OIDC"),
        }
        let unlink = context
            .accounts
            .unlink_oidc(
                first.user_id,
                &identity.issuer,
                VALID_PASSWORD,
                Some(login.session_token.expose()),
            )
            .await?;
        require(
            context
                .sessions
                .lookup(login.session_token.expose())
                .await
                .is_none(),
            "unlink did not revoke the previous session",
        )?;
        require(
            context
                .sessions
                .lookup(unlink.session_token.expose())
                .await
                .is_some(),
            "unlink did not create the rotated session",
        )?;
        require(
            context
                .repository
                .oidc_account_for_login(&identity)
                .await?
                .is_none(),
            "unlinked provider identity remained usable for login",
        )
    })
}
