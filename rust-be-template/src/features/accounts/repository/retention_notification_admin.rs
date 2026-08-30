//! Database-authoritative superuser status and retry controls.

use chrono::{DateTime, Utc};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl,
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::retention_notifications::{
            RetentionNotificationCursor, RetentionNotificationRetryReceipt,
            RetentionNotificationStage, RetentionNotificationStatus,
        },
        error::AccountError,
        repository::{
            account_repository::AccountRepository,
            hard_purge::lock_hard_purge_requester,
        },
    },
    schema::{account_retention_notifications, deleted_account_retention, users},
};

impl AccountRepository {
    pub async fn retention_notification_status(
        &self,
        requester_id: Uuid,
        cursor: Option<RetentionNotificationCursor>,
        limit: i64,
    ) -> Result<Vec<RetentionNotificationStatus>, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<Vec<RetentionNotificationStatus>, AccountError, _>(
                async move |connection| {
                    lock_hard_purge_requester(connection, requester_id).await?;
                    let mut query = account_retention_notifications::table
                        .select((
                            account_retention_notifications::account_retention_notification_id,
                            account_retention_notifications::account_retention_notification_user_id,
                            account_retention_notifications::account_retention_notification_stage,
                            account_retention_notifications::account_retention_notification_scheduled_for,
                            account_retention_notifications::account_retention_notification_next_attempt_at,
                            account_retention_notifications::account_retention_notification_attempt_count,
                            account_retention_notifications::account_retention_notification_claim_expires_at,
                            account_retention_notifications::account_retention_notification_sent_at,
                            account_retention_notifications::account_retention_notification_cancelled_at,
                            account_retention_notifications::account_retention_notification_last_error,
                        ))
                        .into_boxed();
                    if let Some(cursor) = cursor {
                        query = query.filter(
                            account_retention_notifications::account_retention_notification_next_attempt_at
                                .gt(cursor.next_attempt_at)
                                .or(account_retention_notifications::account_retention_notification_next_attempt_at
                                    .eq(cursor.next_attempt_at)
                                    .and(account_retention_notifications::account_retention_notification_id
                                        .gt(cursor.notification_id))),
                        );
                    }
                    let rows = query
                        .order((
                            account_retention_notifications::account_retention_notification_next_attempt_at
                                .asc(),
                            account_retention_notifications::account_retention_notification_id.asc(),
                        ))
                        .limit(limit)
                        .load::<(
                            Uuid,
                            Uuid,
                            RetentionNotificationStage,
                            DateTime<Utc>,
                            DateTime<Utc>,
                            i32,
                            Option<DateTime<Utc>>,
                            Option<DateTime<Utc>>,
                            Option<DateTime<Utc>>,
                            Option<String>,
                        )>(&mut *connection)
                        .await?;
                    Ok(rows
                        .into_iter()
                        .map(
                            |(
                                notification_id,
                                user_id,
                                stage,
                                scheduled_for,
                                next_attempt_at,
                                attempt_count,
                                claim_expires_at,
                                sent_at,
                                cancelled_at,
                                last_error,
                            )| RetentionNotificationStatus {
                                notification_id,
                                user_id,
                                stage,
                                scheduled_for,
                                next_attempt_at,
                                attempt_count,
                                claim_expires_at,
                                sent_at,
                                cancelled_at,
                                last_error,
                            },
                        )
                        .collect())
                },
            )
            .await
    }

    pub async fn retry_retention_notification(
        &self,
        requester_id: Uuid,
        notification_id: Uuid,
        retry_at: DateTime<Utc>,
    ) -> Result<RetentionNotificationRetryReceipt, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<RetentionNotificationRetryReceipt, AccountError, _>(
                async move |connection| {
                    lock_hard_purge_requester(connection, requester_id).await?;
                    let state = account_retention_notifications::table
                        .inner_join(
                            deleted_account_retention::table.on(
                                deleted_account_retention::deleted_account_retention_user_id.eq(
                                    account_retention_notifications::account_retention_notification_user_id,
                                ),
                            ),
                        )
                        .inner_join(
                            users::table.on(users::user_id.eq(
                                account_retention_notifications::account_retention_notification_user_id,
                            )),
                        )
                        .filter(
                            account_retention_notifications::account_retention_notification_id
                                .eq(notification_id),
                        )
                        .select((
                            account_retention_notifications::account_retention_notification_scheduled_for,
                            account_retention_notifications::account_retention_notification_sent_at,
                            account_retention_notifications::account_retention_notification_cancelled_at,
                            users::user_purge_after,
                        ))
                        .for_update()
                        .first::<(
                            DateTime<Utc>,
                            Option<DateTime<Utc>>,
                            Option<DateTime<Utc>>,
                            Option<DateTime<Utc>>,
                        )>(&mut *connection)
                        .await
                        .optional()?;
                    let (scheduled_for, sent_at, cancelled_at, purge_after) =
                        state.ok_or(AccountError::AccountChanged)?;
                    let purge_after = purge_after.ok_or(AccountError::AccountChanged)?;
                    if sent_at.is_some()
                        || cancelled_at.is_some()
                        || retry_at < scheduled_for
                        || retry_at >= purge_after
                    {
                        return Err(AccountError::AccountChanged);
                    }
                    let affected = diesel::update(
                        account_retention_notifications::table.filter(
                            account_retention_notifications::account_retention_notification_id
                                .eq(notification_id),
                        ),
                    )
                    .set((
                        account_retention_notifications::account_retention_notification_next_attempt_at
                            .eq(retry_at),
                        account_retention_notifications::account_retention_notification_claim_token
                            .eq(Option::<Uuid>::None),
                        account_retention_notifications::account_retention_notification_claimed_at
                            .eq(Option::<DateTime<Utc>>::None),
                        account_retention_notifications::account_retention_notification_claim_expires_at
                            .eq(Option::<DateTime<Utc>>::None),
                        account_retention_notifications::account_retention_notification_updated_at
                            .eq(retry_at),
                    ))
                    .execute(&mut *connection)
                    .await?;
                    if affected != 1 {
                        return Err(AccountError::AccountChanged);
                    }
                    Ok(RetentionNotificationRetryReceipt {
                        notification_id,
                        next_attempt_at: retry_at,
                    })
                },
            )
            .await
    }
}
