//! PostgreSQL coverage for owner preservation across account deletion and demotion.

mod support;

use chrono::{Duration, Utc};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use rust_be_template::{
    features::accounts::{
        authorization_error::AuthorizationError,
        domain::{
            authorization::AuthorizationReason,
            lifecycle::{ACCOUNT_RETENTION_DAYS, SoftDeleteAccountReceipt},
            retention_notifications::RetentionNotificationSchedule,
            role::RoleType,
        },
        error::AccountError,
    },
    schema::{deleted_account_retention, user_roles, users},
};

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{AccountTestContext, VALID_PASSWORD, account_test_context, seed_verified_account},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn final_owner_deletion_preserves_identity_authority_and_session() -> TestResult {
    run_database_test(final_owner_case).await
}

fn final_owner_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let owner = seed_verified_account(&context, "FinalOwner").await?;
        context
            .accounts
            .assign_role(owner.user_id, RoleType::Younghyun)
            .await?;
        let login = context
            .accounts
            .login(&owner.email, VALID_PASSWORD, None)
            .await?;
        let deletion = context
            .accounts
            .soft_delete_account(owner.user_id, VALID_PASSWORD)
            .await;
        require(
            matches!(deletion, Err(AccountError::LastActiveYounghyun)),
            "final owner deletion did not return the lifecycle conflict",
        )?;
        require(
            context
                .sessions
                .lookup(login.session_token.expose())
                .await
                .is_some(),
            "rejected deletion revoked the owner's active session",
        )?;
        let mut connection = context.pool.get().await?;
        let identity = users::table
            .find(owner.user_id)
            .select((users::user_name, users::user_email, users::user_deleted_at))
            .first::<(String, String, Option<chrono::DateTime<Utc>>)>(&mut connection)
            .await?;
        require(
            identity == (owner.user_name, owner.email, None),
            "rejected deletion changed the owner's identity",
        )?;
        let retained = deleted_account_retention::table
            .filter(deleted_account_retention::deleted_account_retention_user_id.eq(owner.user_id))
            .count()
            .get_result::<i64>(&mut connection)
            .await?;
        drop(connection);
        require(retained == 0, "rejected deletion retained private identity")?;
        require_one_active_owner(&context).await
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn owner_deletion_succeeds_when_another_owner_remains() -> TestResult {
    run_database_test(second_owner_case).await
}

fn second_owner_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let (first, second) = two_owners(&context).await?;
        context
            .accounts
            .soft_delete_account(first, VALID_PASSWORD)
            .await?;
        require(
            context.repository.role_for_user(first).await?.is_none(),
            "deleted owner retained role authority",
        )?;
        require(
            context.repository.role_for_user(second).await? == Some(RoleType::Younghyun),
            "remaining owner lost role authority",
        )?;
        require_one_active_owner(&context).await
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn concurrent_owner_deletions_leave_one_owner() -> TestResult {
    run_database_test(concurrent_deletion_case).await
}

fn concurrent_deletion_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let (first, second) = two_owners(&context).await?;
        let first_candidate = context.repository.account_deletion_candidate(first).await?;
        let second_candidate = context
            .repository
            .account_deletion_candidate(second)
            .await?;
        // Repository calls exercise independent transactions without the service's
        // process-local authority gate hiding a database ordering regression.
        let (first_result, second_result) = tokio::join!(
            delete_via_repository(&context, first, &first_candidate.password_hash),
            delete_via_repository(&context, second, &second_candidate.password_hash),
        );
        require(
            matches!(
                (&first_result, &second_result),
                (Ok(_), Err(AccountError::LastActiveYounghyun))
                    | (Err(AccountError::LastActiveYounghyun), Ok(_))
            ),
            "competing deletions did not commit once and preserve the final owner",
        )?;
        require_one_active_owner(&context).await
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn owner_deletion_and_demotion_share_the_owner_lock_order() -> TestResult {
    run_database_test(deletion_and_demotion_case).await
}

fn deletion_and_demotion_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let (first, second) = two_owners(&context).await?;
        let candidate = context.repository.account_deletion_candidate(first).await?;
        let reason = AuthorizationReason::try_new("Concurrent owner deletion and demotion")?;
        let (deletion, demotion) = tokio::join!(
            delete_via_repository(&context, first, &candidate.password_hash),
            context
                .repository
                .assign_role_with_audit(first, second, RoleType::User, &reason, None),
        );
        require(
            matches!(
                (&deletion, &demotion),
                (Ok(_), Err(AuthorizationError::Unauthorized))
                    | (Err(AccountError::LastActiveYounghyun), Ok(_))
            ),
            "deletion and demotion did not serialize to one accepted owner removal",
        )?;
        require_one_active_owner(&context).await
    })
}

async fn two_owners(context: &AccountTestContext) -> TestResult<(Uuid, Uuid)> {
    let first = seed_verified_account(context, "OwnerOne").await?;
    let second = seed_verified_account(context, "OwnerTwo").await?;
    for user_id in [first.user_id, second.user_id] {
        context
            .accounts
            .assign_role(user_id, RoleType::Younghyun)
            .await?;
    }
    Ok((first.user_id, second.user_id))
}

async fn delete_via_repository(
    context: &AccountTestContext,
    user_id: Uuid,
    expected_password_hash: &str,
) -> Result<SoftDeleteAccountReceipt, AccountError> {
    let deleted_at = Utc::now();
    let purge_after = deleted_at + Duration::days(ACCOUNT_RETENTION_DAYS);
    let schedule = RetentionNotificationSchedule::from_purge_after(purge_after)
        .ok_or(AccountError::RetentionScheduleOverflow)?;
    context
        .repository
        .soft_delete_account(
            user_id,
            expected_password_hash,
            deleted_at,
            purge_after,
            schedule,
        )
        .await
}

async fn require_one_active_owner(context: &AccountTestContext) -> TestResult {
    let mut connection = context.pool.get().await?;
    let owner_count = user_roles::table
        .inner_join(users::table)
        .filter(user_roles::role_id.eq(RoleType::Younghyun.id()))
        .filter(users::user_deleted_at.is_null())
        .filter(users::user_hard_purged_at.is_null())
        .count()
        .get_result::<i64>(&mut connection)
        .await?;
    require(
        owner_count == 1,
        "owner removal did not preserve exactly one active owner",
    )
}
