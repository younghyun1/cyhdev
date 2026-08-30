mod support;

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use rust_be_template::{
    features::accounts::{
        authorization_error::AuthorizationError,
        domain::{authorization::AuthorizationReason, role::RoleType},
    },
    schema::{authorization_audit_events, user_roles, users},
};

use support::{
    database::{
        BoxError, DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test,
    },
    fixtures::{VALID_PASSWORD, account_test_context, seed_account},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn role_and_permission_changes_are_current_audited_and_session_consistent() -> TestResult {
    run_database_test(role_and_permission_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn owner_locking_prevents_self_lockout_and_concurrent_last_owner_removal() -> TestResult {
    run_database_test(owner_invariant_case).await
}

fn role_and_permission_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let actor = seed_account(&context, "AuthorizationActor").await?;
        let target = seed_account(&context, "AuthorizationTarget").await?;
        context
            .accounts
            .assign_role(actor.user_id, RoleType::Younghyun)
            .await?;
        context
            .accounts
            .verify_email(target.verification_token)
            .await?;
        let login = context
            .accounts
            .login(&target.email, VALID_PASSWORD, None)
            .await?;

        let request_id = Uuid::now_v7();
        let receipt = context
            .accounts
            .assign_role_as_administrator(
                actor.user_id,
                target.user_id,
                RoleType::Moderator.id(),
                "Grant moderation responsibility".to_owned(),
                Some(request_id),
            )
            .await?;
        require(
            receipt.previous_role == RoleType::User && receipt.role_type == RoleType::Moderator,
            "role receipt did not preserve old and new authority",
        )?;
        let session = context.sessions.lookup(login.session_token.expose()).await;
        require(
            matches!(session, Some(session) if session.role_type == RoleType::Moderator),
            "committed role was not refreshed into the target RAM session",
        )?;

        let permissions = context
            .accounts
            .authorization_permissions(actor.user_id)
            .await?;
        let chat_permission = permissions
            .into_iter()
            .find(|permission| permission.permission_name.as_ref() == "chat.moderate");
        let chat_permission = match chat_permission {
            Some(permission) => permission,
            None => return require(false, "seeded chat permission was not present"),
        };
        context
            .accounts
            .set_role_permission_as_administrator(
                actor.user_id,
                RoleType::Moderator.id(),
                chat_permission.permission_id,
                true,
                "Enable moderation capability".to_owned(),
                Some(Uuid::now_v7()),
            )
            .await?;
        require(
            context
                .accounts
                .has_current_permission(target.user_id, "chat.moderate")
                .await?,
            "current permission query missed a committed binding",
        )?;
        context
            .accounts
            .set_role_permission_as_administrator(
                actor.user_id,
                RoleType::Moderator.id(),
                chat_permission.permission_id,
                false,
                "Remove moderation capability".to_owned(),
                Some(Uuid::now_v7()),
            )
            .await?;
        require(
            !context
                .accounts
                .has_current_permission(target.user_id, "chat.moderate")
                .await?,
            "current permission query retained a removed binding",
        )?;

        let renamed = "AuthorizationTargetRenamed";
        let mut connection = context.pool.get().await?;
        diesel::update(users::table.filter(users::user_id.eq(target.user_id)))
            .set(users::user_name.eq(renamed))
            .execute(&mut connection)
            .await?;
        drop(connection);
        let audit = context
            .accounts
            .authorization_audit_events(actor.user_id, None, Some(100))
            .await?;
        let role_event = audit
            .items
            .iter()
            .find(|event| event.audit_event_id == receipt.audit_event_id);
        require(
            matches!(role_event, Some(event)
                if event.target_display_name.as_deref() == Some(renamed)
                    && event.request_id == Some(request_id)),
            "audit read did not resolve the current privacy-safe display name",
        )?;
        assert_audit_guards(&context, receipt.audit_event_id).await
    })
}

async fn assert_audit_guards(
    context: &support::fixtures::AccountTestContext,
    audit_event_id: Uuid,
) -> TestResult {
    let mut connection = context.pool.get().await?;
    let update_result = diesel::update(
        authorization_audit_events::table
            .filter(authorization_audit_events::authorization_audit_event_id.eq(audit_event_id)),
    )
    .set(authorization_audit_events::authorization_audit_event_reason.eq("Rewrite audit record"))
    .execute(&mut connection)
    .await;
    require(
        update_result.is_err(),
        "audit update trigger allowed mutation",
    )?;
    let delete_result = diesel::delete(
        authorization_audit_events::table
            .filter(authorization_audit_events::authorization_audit_event_id.eq(audit_event_id)),
    )
    .execute(&mut connection)
    .await;
    require(
        delete_result.is_err(),
        "audit delete trigger allowed mutation",
    )?;
    // Diesel has no TRUNCATE query-builder node; this test-only DDL verifies the
    // separate statement-level guard rather than interpolating any input.
    let truncate_result = diesel::sql_query("TRUNCATE TABLE authorization_audit_events")
        .execute(&mut connection)
        .await;
    require(
        truncate_result.is_err(),
        "audit truncate trigger allowed mutation",
    )
}

fn owner_invariant_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let first = seed_account(&context, "AuthOwnerOne").await?;
        let second = seed_account(&context, "AuthOwnerTwo").await?;
        context
            .accounts
            .assign_role(first.user_id, RoleType::Younghyun)
            .await?;
        match context
            .accounts
            .assign_role_as_administrator(
                first.user_id,
                first.user_id,
                RoleType::User.id(),
                "Attempt final owner removal".to_owned(),
                None,
            )
            .await
        {
            Err(AuthorizationError::LastActiveYounghyun) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "last active Younghyun was removed"),
        }
        context
            .accounts
            .assign_role(second.user_id, RoleType::Younghyun)
            .await?;
        match context
            .accounts
            .assign_role_as_administrator(
                first.user_id,
                first.user_id,
                RoleType::User.id(),
                "Attempt owner self lockout".to_owned(),
                None,
            )
            .await
        {
            Err(AuthorizationError::SelfLockout) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "administrator removed their own owner role"),
        }

        let reason = AuthorizationReason::try_new("Concurrent owner transfer")?;
        let first_change = context.repository.assign_role_with_audit(
            first.user_id,
            second.user_id,
            RoleType::User,
            &reason,
            None,
        );
        let second_change = context.repository.assign_role_with_audit(
            second.user_id,
            first.user_id,
            RoleType::User,
            &reason,
            None,
        );
        let (first_result, second_result) = tokio::join!(first_change, second_change);
        require(
            first_result.is_ok() ^ second_result.is_ok(),
            "competing owner demotions did not serialize to one committed change",
        )?;
        let mut connection = context.pool.get().await?;
        let owner_count = user_roles::table
            .filter(user_roles::role_id.eq(RoleType::Younghyun.id()))
            .count()
            .get_result::<i64>(&mut connection)
            .await?;
        require(
            owner_count == 1,
            "owner race left an invalid Younghyun count",
        )
    })
}
