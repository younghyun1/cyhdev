//! A password reset removes linked sign-in methods and every session.

mod support;

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use zeroize::Zeroizing;

use rust_be_template::{
    features::accounts::{
        domain::{capability_token::CapabilityToken, oidc::OidcIdentityClaims},
        error::AccountError,
    },
    schema::password_reset_tokens,
};

use support::{
    database::{
        BoxError, DatabaseTestFuture, HarnessError, TestDatabase, TestResult, require,
        run_database_test,
    },
    fixtures::{AccountTestContext, VALID_PASSWORD, account_test_context, seed_verified_account},
};

const NEW_PASSWORD: &str = "ResetPass456";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn password_reset_removes_links_and_sessions() -> TestResult {
    run_database_test(reset_case).await
}

fn reset_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let account = seed_verified_account(&context, "ResetUnlinks").await?;
        let identity = OidcIdentityClaims {
            issuer: "https://id.example.test".to_owned(),
            subject: "reset-subject".to_owned(),
            provider_email: account.email.clone(),
        };
        let first_link = context
            .repository
            .link_oidc_identity(account.user_id, &identity)
            .await?;
        let repeated_link = context
            .repository
            .link_oidc_identity(account.user_id, &identity)
            .await?;
        require(
            first_link.newly_linked && !repeated_link.newly_linked,
            "link receipts did not distinguish a new link from a refreshed one",
        )?;
        require(
            first_link.owner_email == account.email,
            "link receipt did not carry the owner's email for the notice",
        )?;
        let login = context
            .accounts
            .login(&account.email, VALID_PASSWORD, None)
            .await?;

        context
            .accounts
            .request_password_reset(&account.email.to_uppercase())
            .await?;
        let reset_token = known_password_reset_token(&context, account.user_id).await?;
        let receipt = context
            .accounts
            .reset_password(&reset_token, Zeroizing::new(NEW_PASSWORD.to_owned()))
            .await?;
        require(
            receipt.oidc_links_removed == 1,
            "password reset did not report removing the linked identity",
        )?;
        require(
            context
                .repository
                .oidc_account_for_login(&identity)
                .await?
                .is_none(),
            "linked identity survived the password reset",
        )?;
        require(
            context
                .sessions
                .lookup(login.session_token.expose())
                .await
                .is_none(),
            "password reset left an existing session alive",
        )?;
        match context
            .accounts
            .reset_password(&reset_token, Zeroizing::new(NEW_PASSWORD.to_owned()))
            .await
        {
            Err(AccountError::PasswordResetTokenAlreadyUsed) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "a reset token was accepted twice"),
        }
        match context
            .accounts
            .login(&account.email, VALID_PASSWORD, None)
            .await
        {
            Err(AccountError::InvalidCredentials) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "the replaced password still authenticated"),
        }
        context
            .accounts
            .login(&account.email, NEW_PASSWORD, None)
            .await?;
        Ok(())
    })
}

/// Replaces the unused reset token's digest with one for a token the test knows.
async fn known_password_reset_token(
    context: &AccountTestContext,
    user_id: uuid::Uuid,
) -> TestResult<String> {
    let (token, digest) = CapabilityToken::generate()?;
    let mut connection = context.pool.get().await?;
    let updated = diesel::update(
        password_reset_tokens::table
            .filter(password_reset_tokens::user_id.eq(user_id))
            .filter(password_reset_tokens::password_reset_token_used_at.is_null()),
    )
    .set(password_reset_tokens::password_reset_token_hash.eq(digest.as_bytes()))
    .execute(&mut connection)
    .await?;
    if updated == 1 {
        Ok(token.expose().to_owned())
    } else {
        Err(Box::new(HarnessError::Assertion {
            message: "reset request did not leave exactly one unused token",
        }))
    }
}
