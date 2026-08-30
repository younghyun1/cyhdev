mod support;

use chrono::{Duration, Utc};
use diesel::{Connection, ExpressionMethods, QueryDsl, pg::PgConnection};
use diesel_async::RunQueryDsl;
use diesel_migrations::MigrationHarness;

use rust_be_template::{
    features::accounts::{domain::role::RoleType, error::AccountError},
    init::db_migrations::MIGRATIONS,
    schema::email_verification_tokens,
};

use support::{
    database::{
        BoxError, DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test,
    },
    fixtures::{VALID_PASSWORD, account_test_context, seed_account},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn authentication_uses_persisted_credentials() -> TestResult {
    run_database_test(authentication_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn persisted_roles_follow_the_domain_gate_hierarchy() -> TestResult {
    run_database_test(role_gate_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn session_cache_refreshes_and_revokes_after_committed_changes() -> TestResult {
    run_database_test(session_refresh_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn email_verification_enforces_one_time_and_timestamp_boundaries() -> TestResult {
    run_database_test(email_verification_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn embedded_migration_chain_reverts_and_reapplies() -> TestResult {
    run_database_test(migration_round_trip_case).await
}

fn authentication_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let fixture = seed_account(&context, "AuthBoundary").await?;

        require(
            context.accounts.email_exists(&fixture.email).await?,
            "registered email was not found",
        )?;
        match context
            .accounts
            .login(&fixture.email, "WrongPass123", None)
            .await
        {
            Err(AccountError::InvalidCredentials) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => {
                return require(false, "wrong password created a session");
            }
        }

        let receipt = context
            .accounts
            .login(&fixture.email, VALID_PASSWORD, None)
            .await?;
        let session = context
            .sessions
            .lookup(receipt.session_token.expose())
            .await;
        let session = match session {
            Some(session) => session,
            None => return require(false, "successful login did not create a session"),
        };
        require(
            receipt.user_id == fixture.user_id,
            "login returned the wrong account ID",
        )?;
        require(
            session.user_id == fixture.user_id,
            "session belongs to the wrong account",
        )?;
        require(
            context.sessions.len() == 1,
            "session count did not increase",
        )
    })
}

fn role_gate_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let fixture = seed_account(&context, "RoleBoundary").await?;

        let role = context.repository.role_for_user(fixture.user_id).await?;
        require(role == Some(RoleType::User), "signup role was not user")?;
        require(
            context
                .repository
                .has_role(fixture.user_id, RoleType::User)
                .await?,
            "repository did not recognize persisted user role",
        )?;
        require(
            !context
                .repository
                .has_role(fixture.user_id, RoleType::Moderator)
                .await?,
            "repository granted an unassigned moderator role",
        )?;
        require(
            RoleType::Younghyun.permits(RoleType::Moderator),
            "superuser did not satisfy moderator gate",
        )?;
        require(
            !RoleType::User.permits(RoleType::Moderator),
            "user satisfied moderator gate",
        )
    })
}

fn session_refresh_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let fixture = seed_account(&context, "SessionBoundary").await?;
        let receipt = context
            .accounts
            .login(&fixture.email, VALID_PASSWORD, None)
            .await?;

        context
            .accounts
            .verify_email(fixture.verification_token)
            .await?;
        context
            .accounts
            .assign_role(fixture.user_id, RoleType::Moderator)
            .await?;
        let session = match context
            .sessions
            .lookup(receipt.session_token.expose())
            .await
        {
            Some(session) => session,
            None => return require(false, "session disappeared during refresh"),
        };
        require(
            session.user_name.as_ref() == fixture.user_name.as_str(),
            "session retained the wrong account name",
        )?;
        require(
            session.is_email_verified,
            "session retained stale verification state",
        )?;
        require(
            session.role_type == RoleType::Moderator,
            "session retained stale role",
        )?;
        require(
            context
                .accounts
                .logout(receipt.session_token.expose())
                .await,
            "logout did not revoke the session",
        )?;
        require(
            context
                .sessions
                .lookup(receipt.session_token.expose())
                .await
                .is_none(),
            "revoked session remained in the cache",
        )?;
        require(
            !context
                .accounts
                .logout(receipt.session_token.expose())
                .await,
            "second logout reported a nonexistent revocation",
        )?;
        require(context.sessions.is_empty(), "session cache was not empty")
    })
}

fn email_verification_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let consumed = seed_account(&context, "VerifyConsumed").await?;
        context
            .accounts
            .verify_email(consumed.verification_token)
            .await?;
        match context
            .accounts
            .verify_email(consumed.verification_token)
            .await
        {
            Err(AccountError::EmailVerificationTokenAlreadyUsed) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "consumed verification token was accepted twice"),
        }
        let expired = seed_account(&context, "VerifyExpired").await?;
        let fabricated = seed_account(&context, "VerifyFuture").await?;
        let now = Utc::now();
        let mut connection = context.pool.get().await?;
        diesel::update(email_verification_tokens::table.filter(
            email_verification_tokens::email_verification_token.eq(expired.verification_token),
        ))
        .set((
            email_verification_tokens::email_verification_token_created_at
                .eq(now - Duration::hours(2)),
            email_verification_tokens::email_verification_token_expires_at
                .eq(now - Duration::hours(1)),
        ))
        .execute(&mut connection)
        .await?;
        diesel::update(email_verification_tokens::table.filter(
            email_verification_tokens::email_verification_token.eq(fabricated.verification_token),
        ))
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
            .verify_email(expired.verification_token)
            .await
        {
            Err(AccountError::EmailVerificationTokenExpired) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "expired verification token was accepted"),
        }
        match context
            .accounts
            .verify_email(fabricated.verification_token)
            .await
        {
            Err(AccountError::EmailVerificationTokenFabricated) => Ok(()),
            Err(error) => Err(Box::new(error) as BoxError),
            Ok(_) => require(false, "future-created verification token was accepted"),
        }
    })
}

struct MigrationRoundTrip {
    applied_before: usize,
    reverted: usize,
    reapplied: usize,
    applied_after: usize,
    pending_after: usize,
}

fn migration_round_trip_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let database_url = database.database_url().to_owned();
        let summary = tokio::task::spawn_blocking(move || -> TestResult<MigrationRoundTrip> {
            let mut connection = PgConnection::establish(&database_url)?;
            let applied_before = connection.applied_migrations()?.len();
            let reverted = connection.revert_all_migrations(MIGRATIONS)?.len();
            let reapplied = connection.run_pending_migrations(MIGRATIONS)?.len();
            let applied_after = connection.applied_migrations()?.len();
            let pending_after = connection.pending_migrations(MIGRATIONS)?.len();

            Ok(MigrationRoundTrip {
                applied_before,
                reverted,
                reapplied,
                applied_after,
                pending_after,
            })
        })
        .await??;

        require(
            summary.applied_before > 0,
            "no embedded migrations were applied during setup",
        )?;
        require(
            summary.reverted == summary.applied_before,
            "not every applied migration reverted",
        )?;
        require(
            summary.reapplied == summary.applied_before,
            "not every reverted migration reapplied",
        )?;
        require(
            summary.applied_after == summary.applied_before,
            "applied migration count changed after round trip",
        )?;
        require(
            summary.pending_after == 0,
            "migration round trip left pending migrations",
        )
    })
}
