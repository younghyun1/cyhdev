//! PostgreSQL coverage for account deletion, retention, and permanent tombstones.

mod support;

use chrono::{DateTime, Duration, Utc};
use diesel::{Connection, QueryDsl, pg::PgConnection};
use diesel_async::RunQueryDsl;
use diesel_migrations::MigrationHarness;

use rust_be_template::{
    features::accounts::{
        domain::{
            account::DELETED_USER_DISPLAY_NAME,
            lifecycle::{ACCOUNT_RETENTION_DAYS, SYSTEM_ACTOR_USER_ID},
            role::RoleType,
        },
        error::AccountError,
    },
    init::db_migrations::MIGRATIONS,
    schema::users,
};

use support::{
    database::{BoxError, DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{VALID_PASSWORD, account_test_context, seed_account},
    lifecycle::{
        PROFILE_OBJECT_URL, require_authored_content_retained, seed_authored_content,
    },
    lifecycle_assertions::{
        require_account_authority_cleared, require_login_rejected, require_permanent_tombstone,
        require_public_identity_is_generic,
        retained_and_tombstone_identity,
    },
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn soft_delete_and_due_purge_preserve_tombstones_and_authored_content() -> TestResult {
    run_database_test(account_lifecycle_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn protected_system_actor_rejects_account_lifecycle_mutations() -> TestResult {
    run_database_test(system_actor_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn account_lifecycle_migration_reverts_and_reapplies() -> TestResult {
    run_database_test(lifecycle_migration_round_trip_case).await
}

fn account_lifecycle_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let admin = seed_account(&context, "LifecycleAdmin").await?;
        context
            .accounts
            .assign_role(admin.user_id, RoleType::Younghyun)
            .await?;
        let unauthorized = seed_account(&context, "LifecycleNoRole").await?;
        let account = seed_account(&context, "LifecycleBoundary").await?;
        let content = seed_authored_content(&context, account.user_id, &account.user_name).await?;
        let login = context
            .accounts
            .login(&account.email, VALID_PASSWORD, None)
            .await?;

        let deleted = context
            .accounts
            .soft_delete_account(account.user_id, VALID_PASSWORD)
            .await?;
        require(
            deleted.user_id == account.user_id
                && deleted.purge_after - deleted.deleted_at
                    == Duration::days(ACCOUNT_RETENTION_DAYS),
            "soft-delete receipt did not contain the configured retention deadline",
        )?;
        require(
            context
                .sessions
                .lookup(login.session_token.expose())
                .await
                .is_none(),
            "soft deletion did not immediately revoke the active session",
        )?;
        require_login_rejected(&context, &account.email).await?;

        let (retained_name, retained_email, tombstone_name, tombstone_email) =
            retained_and_tombstone_identity(&context, account.user_id).await?;
        require(
            retained_name == account.user_name && retained_email == account.email,
            "private retention did not preserve the original identity",
        )?;
        require(
            tombstone_name != account.user_name && tombstone_email != account.email,
            "public tombstone retained private identity",
        )?;
        require_account_authority_cleared(&context, account.user_id).await?;
        require_public_identity_is_generic(&context, &account.user_name).await?;
        require_authored_content_retained(
            &context,
            &content,
            account.user_id,
            DELETED_USER_DISPLAY_NAME,
        )
        .await?;

        match context
            .accounts
            .hard_purge_account(unauthorized.user_id, account.user_id)
            .await
        {
            Err(AccountError::HardPurgeRequesterUnauthorized) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "non-superuser authorized hard purge"),
        }
        context
            .accounts
            .assign_role(admin.user_id, RoleType::User)
            .await?;
        match context
            .accounts
            .hard_purge_account(admin.user_id, account.user_id)
            .await
        {
            Err(AccountError::HardPurgeRequesterUnauthorized) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "demoted superuser authorized hard purge"),
        }
        context
            .accounts
            .assign_role(admin.user_id, RoleType::Younghyun)
            .await?;
        match context
            .accounts
            .hard_purge_account(admin.user_id, account.user_id)
            .await
        {
            Err(AccountError::RetentionPeriodActive { purge_after }) => {
                require(
                    purge_after == deleted.purge_after,
                    "premature purge reported the wrong retention deadline",
                )?;
            }
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "hard purge succeeded before the retention deadline"),
        }

        let purged = context
            .repository
            .hard_purge_account(
                admin.user_id,
                account.user_id,
                deleted.purge_after + Duration::seconds(1),
            )
            .await?;
        let profile_picture_id = purged
            .profile_objects
            .iter()
            .find(|object| object.object_url.as_deref() == Some(PROFILE_OBJECT_URL))
            .map(|object| object.profile_picture_id);
        let profile_picture_id = match profile_picture_id {
            Some(profile_picture_id) => profile_picture_id,
            None => return require(false, "hard purge did not retain profile cleanup work"),
        };
        require(
            purged.user_id == account.user_id,
            "hard purge finalized cloud profile metadata before object deletion",
        )?;
        let repeated = context
            .accounts
            .hard_purge_account(admin.user_id, account.user_id)
            .await?;
        require(
            repeated.hard_purged_at == purged.hard_purged_at
                && repeated
                    .profile_objects
                    .iter()
                    .any(|object| object.profile_picture_id == profile_picture_id),
            "hard purge retry did not return the same cleanup ledger",
        )?;
        let finalized = context
            .accounts
            .finalize_profile_cleanup(admin.user_id, account.user_id, &[profile_picture_id])
            .await?;
        require(
            finalized.metadata_deleted == 1 && finalized.metadata_remaining == 0,
            "confirmed profile cleanup did not finalize metadata",
        )?;
        let finalized_again = context
            .accounts
            .finalize_profile_cleanup(admin.user_id, account.user_id, &[profile_picture_id])
            .await?;
        require(
            finalized_again.metadata_deleted == 0 && finalized_again.metadata_remaining == 0,
            "profile cleanup finalization was not idempotent",
        )?;
        require_permanent_tombstone(&context, account.user_id, &tombstone_name, &tombstone_email)
            .await?;
        require_authored_content_retained(
            &context,
            &content,
            account.user_id,
            DELETED_USER_DISPLAY_NAME,
        )
        .await
    })
}

fn system_actor_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let admin = seed_account(&context, "SystemActorAdmin").await?;
        context
            .accounts
            .assign_role(admin.user_id, RoleType::Younghyun)
            .await?;
        match context
            .accounts
            .soft_delete_account(SYSTEM_ACTOR_USER_ID, "not-a-system-password")
            .await
        {
            Err(AccountError::SystemActorProtected) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "system actor accepted soft deletion"),
        }
        match context
            .accounts
            .hard_purge_account(admin.user_id, SYSTEM_ACTOR_USER_ID)
            .await
        {
            Err(AccountError::SystemActorProtected) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "system actor accepted hard purge"),
        }

        let mut connection = context.pool.get().await?;
        let lifecycle = users::table
            .find(SYSTEM_ACTOR_USER_ID)
            .select((
                users::user_is_system_actor,
                users::user_deleted_at,
                users::user_purge_after,
                users::user_hard_purged_at,
            ))
            .first::<(
                bool,
                Option<DateTime<Utc>>,
                Option<DateTime<Utc>>,
                Option<DateTime<Utc>>,
            )>(&mut connection)
            .await?;
        drop(connection);
        require(
            lifecycle == (true, None, None, None),
            "system actor lifecycle fields changed",
        )
    })
}

struct MigrationRoundTrip {
    applied_before: usize,
    reverted: usize,
    reapplied: usize,
    applied_after: usize,
    pending_after: usize,
}

fn lifecycle_migration_round_trip_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
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

        require(summary.applied_before > 0, "no migrations existed before lifecycle rollback")?;
        require(
            summary.reverted == summary.applied_before,
            "migration harness did not revert the complete lifecycle chain",
        )?;
        require(
            summary.reapplied == summary.applied_before,
            "migration harness did not reapply the complete lifecycle chain",
        )?;
        require(
            summary.applied_after == summary.applied_before,
            "migration count changed after lifecycle round trip",
        )?;
        require(summary.pending_after == 0, "lifecycle round trip left a pending migration")
    })
}
