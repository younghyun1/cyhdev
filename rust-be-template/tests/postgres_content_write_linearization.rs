//! PostgreSQL concurrency coverage for deletion-linearized content writes.

mod support;

use std::{sync::Arc, time::Duration};

use chrono::Utc;
use diesel::QueryDsl;
use diesel_async::{AsyncConnection, RunQueryDsl};
use tokio::sync::oneshot;
use uuid::Uuid;

use rust_be_template::{
    features::accounts::domain::account::DELETED_USER_DISPLAY_NAME,
    features::live_chat::{
        domain::message::LIVE_CHAT_SENDER_KIND_USER,
        repository::compatibility::LiveChatMessageInsertable,
    },
    persistence::active_user::{ActiveUserWriteError, lock_active_user},
    schema::live_chat_messages,
};

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{VALID_PASSWORD, account_test_context, seed_account},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn content_write_and_soft_delete_share_one_account_lock_order() -> TestResult {
    run_database_test(content_write_linearization_case).await
}

fn content_write_linearization_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let account = seed_account(&context, "ContentWriteLinearization").await?;
        let pool = context.pool.clone();
        let accounts = Arc::clone(&context.accounts);
        let user_id = account.user_id;
        let message_id = Uuid::now_v7();
        let original_name = account.user_name.clone();
        let (locked_tx, locked_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();

        let writer_pool = pool.clone();
        let writer = tokio::spawn(async move {
            let mut connection = writer_pool.get().await?;
            connection
                .transaction::<(), ActiveUserWriteError, _>(async |connection| {
                    lock_active_user(&mut *connection, user_id).await?;
                    let _ = locked_tx.send(());
                    release_rx.await.map_err(|_| ActiveUserWriteError::Denied)?;
                    diesel::insert_into(live_chat_messages::table)
                        .values(LiveChatMessageInsertable {
                            live_chat_message_id: message_id,
                            room_key: "main".to_owned(),
                            user_id: Some(user_id),
                            guest_ip: None,
                            sender_kind: LIVE_CHAT_SENDER_KIND_USER,
                            sender_display_name: original_name,
                            message_body: "write committed before deletion".to_owned(),
                            message_created_at: Utc::now(),
                        })
                        .execute(&mut *connection)
                        .await?;
                    Ok(())
                })
                .await?;
            Ok(()) as TestResult
        });

        locked_rx.await?;
        let deleting_accounts = Arc::clone(&accounts);
        let mut deletion = tokio::spawn(async move {
            deleting_accounts
                .soft_delete_account(user_id, VALID_PASSWORD)
                .await
        });
        let deletion_waited = tokio::time::timeout(Duration::from_millis(50), &mut deletion).await;
        require(
            deletion_waited.is_err(),
            "soft deletion did not wait for the active content-write transaction",
        )?;
        let _ = release_tx.send(());
        writer.await??;
        deletion.await??;

        let mut connection = pool.get().await?;
        let retained_name = live_chat_messages::table
            .find(message_id)
            .select(live_chat_messages::sender_display_name)
            .first::<String>(&mut connection)
            .await?;
        require(
            retained_name == DELETED_USER_DISPLAY_NAME,
            "deletion did not scrub the write that serialized before it",
        )?;

        let rejected_message_id = Uuid::now_v7();
        let rejected = connection
            .transaction::<(), ActiveUserWriteError, _>(async |connection| {
                lock_active_user(&mut *connection, user_id).await?;
                diesel::insert_into(live_chat_messages::table)
                    .values(LiveChatMessageInsertable {
                        live_chat_message_id: rejected_message_id,
                        room_key: "main".to_owned(),
                        user_id: Some(user_id),
                        guest_ip: None,
                        sender_kind: LIVE_CHAT_SENDER_KIND_USER,
                        sender_display_name: "must not persist".to_owned(),
                        message_body: "write after deletion".to_owned(),
                        message_created_at: Utc::now(),
                    })
                    .execute(&mut *connection)
                    .await?;
                Ok(())
            })
            .await;
        require(
            matches!(rejected, Err(ActiveUserWriteError::Inactive)),
            "content write did not reject the committed account tombstone",
        )
    })
}
