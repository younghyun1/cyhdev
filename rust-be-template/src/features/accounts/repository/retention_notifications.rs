//! Durable scheduling, claiming, and delivery outcomes for retention notices.

use chrono::{DateTime, TimeDelta, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    features::accounts::{
        domain::retention_notifications::{
            ClaimedRetentionNotification, RETENTION_NOTIFICATION_BATCH_SIZE,
            RETENTION_NOTIFICATION_CLAIM_MINUTES, RetentionNotificationClaimBatch,
            RetentionNotificationDeliveryItem, RetentionNotificationSchedule,
            RetentionNotificationStage,
        },
        error::AccountError,
        repository::{
            account_repository::AccountRepository,
            sql_enums::StoredRetentionNotificationStage,
        },
    },
    schema::{account_retention_notifications, deleted_account_retention, users},
};

#[derive(diesel::Insertable)]
#[diesel(table_name = account_retention_notifications)]
struct NewRetentionNotification {
    account_retention_notification_user_id: Uuid,
    account_retention_notification_stage: StoredRetentionNotificationStage,
    account_retention_notification_scheduled_for: DateTime<Utc>,
    account_retention_notification_next_attempt_at: DateTime<Utc>,
    account_retention_notification_created_at: DateTime<Utc>,
    account_retention_notification_updated_at: DateTime<Utc>,
}

pub(super) async fn insert_retention_notification_schedule(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
    schedule: &RetentionNotificationSchedule,
    created_at: DateTime<Utc>,
) -> Result<(), AccountError> {
    let notifications = [
        NewRetentionNotification {
            account_retention_notification_user_id: user_id,
            account_retention_notification_stage:
                RetentionNotificationStage::SevenDaysBeforePurge.into(),
            account_retention_notification_scheduled_for: schedule.seven_days_before_purge,
            account_retention_notification_next_attempt_at: schedule.seven_days_before_purge,
            account_retention_notification_created_at: created_at,
            account_retention_notification_updated_at: created_at,
        },
        NewRetentionNotification {
            account_retention_notification_user_id: user_id,
            account_retention_notification_stage:
                RetentionNotificationStage::OneDayBeforePurge.into(),
            account_retention_notification_scheduled_for: schedule.one_day_before_purge,
            account_retention_notification_next_attempt_at: schedule.one_day_before_purge,
            account_retention_notification_created_at: created_at,
            account_retention_notification_updated_at: created_at,
        },
    ];
    diesel::insert_into(account_retention_notifications::table)
        .values(&notifications)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

impl AccountRepository {
    /// Claims the first due keyset page without waiting on another worker's rows.
    pub async fn claim_due_retention_notifications(
        &self,
        now: DateTime<Utc>,
    ) -> Result<RetentionNotificationClaimBatch, AccountError> {
        let claim_token = Uuid::now_v7();
        let claim_expires_at = now
            .checked_add_signed(TimeDelta::minutes(RETENTION_NOTIFICATION_CLAIM_MINUTES))
            .ok_or(AccountError::RetentionScheduleOverflow)?;
        let mut connection = self.connection().await?;
        connection
            .transaction::<RetentionNotificationClaimBatch, AccountError, _>(
                async move |connection| {
                    let rows = account_retention_notifications::table
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
                            account_retention_notifications::account_retention_notification_sent_at
                                .is_null(),
                        )
                        .filter(
                            account_retention_notifications::account_retention_notification_cancelled_at
                                .is_null(),
                        )
                        .filter(
                            account_retention_notifications::account_retention_notification_next_attempt_at
                                .le(now),
                        )
                        .filter(
                            account_retention_notifications::account_retention_notification_claim_token
                                .is_null()
                                .or(account_retention_notifications::account_retention_notification_claim_expires_at
                                    .le(now)),
                        )
                        .filter(users::user_deleted_at.is_not_null())
                        .filter(users::user_hard_purged_at.is_null())
                        .filter(users::user_purge_after.gt(now))
                        .select((
                            account_retention_notifications::account_retention_notification_id,
                            account_retention_notifications::account_retention_notification_stage,
                            account_retention_notifications::account_retention_notification_next_attempt_at,
                            account_retention_notifications::account_retention_notification_attempt_count,
                        ))
                        .order((
                            account_retention_notifications::account_retention_notification_next_attempt_at
                                .asc(),
                            account_retention_notifications::account_retention_notification_id.asc(),
                        ))
                        .limit(RETENTION_NOTIFICATION_BATCH_SIZE)
                        .for_update()
                        .skip_locked()
                        .load::<(Uuid, StoredRetentionNotificationStage, DateTime<Utc>, i32)>(
                            &mut *connection,
                        )
                        .await?;
                    if rows.is_empty() {
                        return Ok(RetentionNotificationClaimBatch {
                            claim_token,
                            notifications: Vec::new(),
                        });
                    }
                    let notification_ids = rows
                        .iter()
                        .map(|(notification_id, _, _, _)| *notification_id)
                        .collect::<Vec<_>>();
                    let claimed = diesel::update(
                        account_retention_notifications::table.filter(
                            account_retention_notifications::account_retention_notification_id
                                .eq_any(&notification_ids),
                        ),
                    )
                    .set((
                        account_retention_notifications::account_retention_notification_claim_token
                            .eq(claim_token),
                        account_retention_notifications::account_retention_notification_claimed_at
                            .eq(now),
                        account_retention_notifications::account_retention_notification_claim_expires_at
                            .eq(claim_expires_at),
                        account_retention_notifications::account_retention_notification_attempt_count
                            .eq(account_retention_notifications::account_retention_notification_attempt_count + 1),
                        account_retention_notifications::account_retention_notification_updated_at
                            .eq(now),
                    ))
                    .execute(&mut *connection)
                    .await?;
                    if claimed != rows.len() {
                        return Err(AccountError::AccountChanged);
                    }
                    Ok(RetentionNotificationClaimBatch {
                        claim_token,
                        notifications: rows
                            .into_iter()
                            .map(
                                |(notification_id, stage, next_attempt_at, attempt_count)| {
                                    ClaimedRetentionNotification {
                                        notification_id,
                                        stage: stage.into_domain(),
                                        next_attempt_at,
                                        attempt_count: attempt_count.saturating_add(1),
                                    }
                                },
                            )
                            .collect(),
                    })
                },
            )
            .await
    }

    /// Revalidates every claimed row and fetches private email in one bounded query.
    pub async fn revalidate_retention_notification_claim(
        &self,
        claim_token: Uuid,
        notification_ids: &[Uuid],
        now: DateTime<Utc>,
    ) -> Result<Vec<RetentionNotificationDeliveryItem>, AccountError> {
        if notification_ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut connection = self.connection().await?;
        let rows = account_retention_notifications::table
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
                    .eq_any(notification_ids),
            )
            .filter(
                account_retention_notifications::account_retention_notification_claim_token
                    .eq(claim_token),
            )
            .filter(
                account_retention_notifications::account_retention_notification_claim_expires_at
                    .gt(now),
            )
            .filter(
                account_retention_notifications::account_retention_notification_sent_at.is_null(),
            )
            .filter(
                account_retention_notifications::account_retention_notification_cancelled_at
                    .is_null(),
            )
            .filter(users::user_deleted_at.is_not_null())
            .filter(users::user_hard_purged_at.is_null())
            .filter(users::user_purge_after.gt(now))
            .select((
                account_retention_notifications::account_retention_notification_id,
                account_retention_notifications::account_retention_notification_stage,
                deleted_account_retention::deleted_account_retention_email,
                users::user_purge_after,
                account_retention_notifications::account_retention_notification_attempt_count,
            ))
            .order(account_retention_notifications::account_retention_notification_id.asc())
            .load::<(Uuid, StoredRetentionNotificationStage, String, Option<DateTime<Utc>>, i32)>(
                &mut connection,
            )
            .await?;
        Ok(rows
            .into_iter()
            .filter_map(
                |(notification_id, stage, retained_email, purge_after, attempt_count)| {
                    purge_after.map(|purge_after| RetentionNotificationDeliveryItem {
                        notification_id,
                        stage: stage.into_domain(),
                        retained_email: Zeroizing::new(retained_email),
                        purge_after,
                        attempt_count,
                    })
                },
            )
            .collect())
    }

}
