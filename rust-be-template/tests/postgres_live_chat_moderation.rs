//! Database-current authority, content erasure, and tombstone pagination coverage.

mod support;

use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use rust_be_template::{
    features::{
        accounts::domain::role::RoleType,
        live_chat::{
            domain::{actor::ChatActor, guest_identity::GuestIdentityKey},
            error::LiveChatError,
            repository::live_chat_repository::LiveChatRepository,
        },
    },
    schema::live_chat_messages,
};
use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{VALID_PASSWORD, account_test_context, seed_account},
};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn moderation_rechecks_authority_and_erases_content() -> TestResult {
    run_database_test(moderation_case).await
}

fn moderation_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let admin = seed_account(&context, "ChatModerator").await?;
        let repository = LiveChatRepository::new(context.pool.clone());
        let guest = ChatActor::guest(
            "192.0.2.1".parse()?,
            &GuestIdentityKey::from_secret(&[0x5a; 32]),
            None,
        );
        let first = repository
            .insert_message(&guest, "retained".to_owned())
            .await?;
        let second = repository
            .insert_message(&guest, "removed".to_owned())
            .await?;
        let id = second.live_chat_message_id;

        require(
            matches!(
                repository.delete_message(admin.user_id, id).await,
                Err(LiveChatError::Forbidden)
            ),
            "ordinary user could moderate chat",
        )?;
        require(
            repository.recent_messages("main", 10).await?.len() == 2,
            "denied deletion modified messages",
        )?;
        context
            .accounts
            .assign_role(admin.user_id, RoleType::Younghyun)
            .await?;
        repository.delete_message(admin.user_id, id).await?;
        repository.delete_message(admin.user_id, id).await?;
        repository
            .delete_message(admin.user_id, Uuid::now_v7())
            .await?;

        let mut connection = context.pool.get().await?;
        let (body, deleted_at) = live_chat_messages::table
            .find(id)
            .select((
                live_chat_messages::message_body,
                live_chat_messages::message_deleted_at,
            ))
            .first::<(String, Option<chrono::DateTime<chrono::Utc>>)>(&mut connection)
            .await?;
        drop(connection);
        require(
            body.is_empty() && deleted_at.is_some(),
            "moderated content was retained",
        )?;
        let recent = repository.recent_messages("main", 10).await?;
        require(
            recent.len() == 1 && recent[0].live_chat_message_id == first.live_chat_message_id,
            "recent history exposed deleted message",
        )?;
        let older = repository.messages_before(id, 10).await?;
        require(
            older.len() == 1 && older[0].live_chat_message_id == first.live_chat_message_id,
            "deleted cursor broke older-message pagination",
        )?;

        context
            .accounts
            .assign_role(admin.user_id, RoleType::User)
            .await?;
        require(
            matches!(
                repository
                    .delete_message(admin.user_id, first.live_chat_message_id)
                    .await,
                Err(LiveChatError::Forbidden)
            ),
            "demoted superuser could delete messages",
        )?;
        context
            .accounts
            .assign_role(admin.user_id, RoleType::Younghyun)
            .await?;
        let remaining_owner = seed_account(&context, "ChatRemainingOwner").await?;
        context
            .accounts
            .assign_role(remaining_owner.user_id, RoleType::Younghyun)
            .await?;
        context
            .accounts
            .soft_delete_account(admin.user_id, VALID_PASSWORD)
            .await?;
        require(
            matches!(
                repository
                    .delete_message(admin.user_id, first.live_chat_message_id)
                    .await,
                Err(LiveChatError::Unauthorized)
            ),
            "deleted superuser could delete messages",
        )
    })
}
