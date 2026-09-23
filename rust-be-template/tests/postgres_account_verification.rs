//! Email verification: one-time tokens, session revocation, and pre-registration takeover.

mod support;

use chrono::{Duration, Utc};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

use rust_be_template::{
    features::accounts::{
        domain::{
            account::{SessionPrincipal, SignupOutcome},
            capability_token::CapabilityDigest,
            role::RoleType,
            session::SessionToken,
        },
        error::AccountError,
    },
    schema::{email_verification_tokens, users},
};

use support::{
    database::{
        BoxError, DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test,
    },
    fixtures::{
        AccountFixture, AccountTestContext, VALID_PASSWORD, account_test_context,
        known_verification_token, seed_account, seed_verified_account, signup_command,
    },
};

const SQUATTER_PASSWORD: &str = "SquatterPass123";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn email_verification_enforces_one_time_and_timestamp_boundaries() -> TestResult {
    run_database_test(email_verification_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn duplicate_signup_takes_over_only_unverified_accounts() -> TestResult {
    run_database_test(duplicate_signup_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn email_verification_revokes_existing_sessions() -> TestResult {
    run_database_test(verification_revocation_case).await
}

fn email_verification_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let consumed = seed_account(&context, "VerifyConsumed").await?;
        context
            .accounts
            .verify_email(&consumed.verification_token)
            .await?;
        match context
            .accounts
            .verify_email(&consumed.verification_token)
            .await
        {
            Err(AccountError::EmailVerificationTokenAlreadyUsed) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "consumed verification token was accepted twice"),
        }
        let expired = seed_account(&context, "VerifyExpired").await?;
        let fabricated = seed_account(&context, "VerifyFuture").await?;
        let now = Utc::now();
        let expired_digest = digest_of(&expired.verification_token)?;
        let fabricated_digest = digest_of(&fabricated.verification_token)?;
        let mut connection = context.pool.get().await?;
        diesel::update(email_verification_tokens::table.filter(
            email_verification_tokens::email_verification_token_hash.eq(expired_digest.as_bytes()),
        ))
        .set((
            email_verification_tokens::email_verification_token_created_at
                .eq(now - Duration::hours(2)),
            email_verification_tokens::email_verification_token_expires_at
                .eq(now - Duration::hours(1)),
        ))
        .execute(&mut connection)
        .await?;
        diesel::update(
            email_verification_tokens::table.filter(
                email_verification_tokens::email_verification_token_hash
                    .eq(fabricated_digest.as_bytes()),
            ),
        )
        .set((
            email_verification_tokens::email_verification_token_created_at
                .eq(now + Duration::hours(1)),
            email_verification_tokens::email_verification_token_expires_at
                .eq(now + Duration::hours(2)),
        ))
        .execute(&mut connection)
        .await?;
        drop(connection);
        match context
            .accounts
            .verify_email(&expired.verification_token)
            .await
        {
            Err(AccountError::EmailVerificationTokenExpired) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "expired verification token was accepted"),
        }
        match context
            .accounts
            .verify_email(&fabricated.verification_token)
            .await
        {
            Err(AccountError::EmailVerificationTokenFabricated) => Ok(()),
            Err(error) => Err(Box::new(error) as BoxError),
            Ok(_) => require(false, "future-created verification token was accepted"),
        }
    })
}

fn digest_of(token: &str) -> TestResult<CapabilityDigest> {
    match CapabilityDigest::from_submitted(token) {
        Some(digest) => Ok(digest),
        None => {
            require(false, "fixture issued a non-canonical verification token")?;
            Err(Box::new(AccountError::EmailVerificationTokenNotFound) as BoxError)
        }
    }
}

fn duplicate_signup_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let squatter = seed_account(&context, "Squatter").await?;
        let squatter_token = known_verification_token(&context, squatter.user_id).await?;
        let pre_existing = unverified_session(&context, &squatter).await?;

        let takeover = context
            .accounts
            .signup(signup_command(
                "RightfulOwner",
                &squatter.email.to_uppercase(),
                SQUATTER_PASSWORD,
                &squatter,
            ))
            .await?;
        require(
            matches!(takeover, SignupOutcome::ReplacedUnverified(_)),
            "second signup did not replace the unverified account",
        )?;
        require(
            context
                .sessions
                .lookup(pre_existing.expose())
                .await
                .is_none(),
            "replacing the unverified account left its session alive",
        )?;
        match context.accounts.verify_email(&squatter_token).await {
            Err(AccountError::EmailVerificationTokenNotFound) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "replaced verification token still verified"),
        }
        match context
            .accounts
            .login(&squatter.email, VALID_PASSWORD, None)
            .await
        {
            Err(AccountError::InvalidCredentials) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "replaced password still authenticated"),
        }
        match context
            .accounts
            .login(&squatter.email, SQUATTER_PASSWORD, None)
            .await
        {
            Err(AccountError::EmailNotVerified) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "unverified account received a session"),
        }

        let owner_token = known_verification_token(&context, squatter.user_id).await?;
        context.accounts.verify_email(&owner_token).await?;
        let login = context
            .accounts
            .login(&squatter.email, SQUATTER_PASSWORD, None)
            .await?;
        require(
            login.user_id == squatter.user_id,
            "verified owner signed in to the wrong account",
        )?;
        require(
            user_name_of(&context, squatter.user_id).await? == "RightfulOwner",
            "replacement did not take the new submission's user name",
        )?;

        let verified = seed_verified_account(&context, "VerifiedOwner").await?;
        let unchanged = context
            .accounts
            .signup(signup_command(
                "FreshNameForVerified",
                &verified.email,
                SQUATTER_PASSWORD,
                &verified,
            ))
            .await?;
        require(
            matches!(unchanged, SignupOutcome::AlreadyVerified),
            "duplicate signup changed a verified account",
        )?;
        match context
            .accounts
            .signup(signup_command(
                "rightfulowner",
                &verified.email,
                SQUATTER_PASSWORD,
                &verified,
            ))
            .await
        {
            Err(AccountError::UserNameUnavailable) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "a held user name was accepted for a duplicate email"),
        }
        context
            .accounts
            .login(&verified.email, VALID_PASSWORD, None)
            .await?;
        Ok(())
    })
}

fn verification_revocation_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let account = seed_account(&context, "VerifyRevokes").await?;
        let pre_existing = unverified_session(&context, &account).await?;
        context
            .accounts
            .verify_email(&account.verification_token)
            .await?;
        require(
            context
                .sessions
                .lookup(pre_existing.expose())
                .await
                .is_none(),
            "verification upgraded a pre-existing session instead of revoking it",
        )
    })
}

/// A session opened before verification, as older builds issued on unverified login.
async fn unverified_session(
    context: &AccountTestContext,
    account: &AccountFixture,
) -> TestResult<SessionToken> {
    Ok(context
        .sessions
        .create_principal(
            &SessionPrincipal {
                user_id: account.user_id,
                user_name: account.user_name.clone(),
                is_email_verified: false,
                country: account.country,
                language: account.language,
            },
            RoleType::User,
            None,
            None,
        )
        .await?)
}

async fn user_name_of(context: &AccountTestContext, user_id: uuid::Uuid) -> TestResult<String> {
    let mut connection = context.pool.get().await?;
    Ok(users::table
        .find(user_id)
        .select(users::user_name)
        .first::<String>(&mut connection)
        .await?)
}
