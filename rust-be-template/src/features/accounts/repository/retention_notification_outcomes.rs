//! Claim completion and hard-purge cancellation for retention notices.

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use jiff::Timestamp;
use jiff_diesel::ToDiesel;
use uuid::Uuid;

use crate::{
    features::accounts::{error::AccountError, repository::account_repository::AccountRepository},
    schema::account_retention_notifications,
};

impl AccountRepository {
    pub async fn mark_retention_notification_sent(
        &self,
        notification_id: Uuid,
        claim_token: Uuid,
        sent_at: Timestamp,
    ) -> Result<bool, AccountError> {
        let mut connection = self.connection().await?;
        let affected = diesel::update(
            account_retention_notifications::table
                .filter(
                    account_retention_notifications::account_retention_notification_id
                        .eq(notification_id),
                )
                .filter(
                    account_retention_notifications::account_retention_notification_claim_token
                        .eq(claim_token),
                )
                .filter(
                    account_retention_notifications::account_retention_notification_sent_at
                        .is_null(),
                )
                .filter(
                    account_retention_notifications::account_retention_notification_cancelled_at
                        .is_null(),
                ),
        )
        .set((
            account_retention_notifications::account_retention_notification_sent_at
                .eq(sent_at.to_diesel()),
            account_retention_notifications::account_retention_notification_claim_token
                .eq(Option::<Uuid>::None),
            account_retention_notifications::account_retention_notification_claimed_at
                .eq(Option::<Timestamp>::None.to_diesel()),
            account_retention_notifications::account_retention_notification_claim_expires_at
                .eq(Option::<Timestamp>::None.to_diesel()),
            account_retention_notifications::account_retention_notification_last_error
                .eq(Option::<String>::None),
            account_retention_notifications::account_retention_notification_updated_at
                .eq(sent_at.to_diesel()),
        ))
        .execute(&mut connection)
        .await?;
        Ok(affected == 1)
    }

    pub async fn mark_retention_notification_failed(
        &self,
        notification_id: Uuid,
        claim_token: Uuid,
        failed_at: Timestamp,
        next_attempt_at: Timestamp,
        error: &str,
    ) -> Result<bool, AccountError> {
        let mut connection = self.connection().await?;
        let affected = diesel::update(
            account_retention_notifications::table
                .filter(
                    account_retention_notifications::account_retention_notification_id
                        .eq(notification_id),
                )
                .filter(
                    account_retention_notifications::account_retention_notification_claim_token
                        .eq(claim_token),
                )
                .filter(
                    account_retention_notifications::account_retention_notification_sent_at
                        .is_null(),
                )
                .filter(
                    account_retention_notifications::account_retention_notification_cancelled_at
                        .is_null(),
                ),
        )
        .set((
            account_retention_notifications::account_retention_notification_next_attempt_at
                .eq(next_attempt_at.to_diesel()),
            account_retention_notifications::account_retention_notification_claim_token
                .eq(Option::<Uuid>::None),
            account_retention_notifications::account_retention_notification_claimed_at
                .eq(Option::<Timestamp>::None.to_diesel()),
            account_retention_notifications::account_retention_notification_claim_expires_at
                .eq(Option::<Timestamp>::None.to_diesel()),
            account_retention_notifications::account_retention_notification_last_error.eq(error),
            account_retention_notifications::account_retention_notification_updated_at
                .eq(failed_at.to_diesel()),
        ))
        .execute(&mut connection)
        .await?;
        Ok(affected == 1)
    }
}

pub(super) async fn cancel_retention_notifications_for_hard_purge(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
    cancelled_at: Timestamp,
) -> Result<(), AccountError> {
    diesel::update(
        account_retention_notifications::table
            .filter(
                account_retention_notifications::account_retention_notification_user_id.eq(user_id),
            )
            .filter(
                account_retention_notifications::account_retention_notification_sent_at.is_null(),
            )
            .filter(
                account_retention_notifications::account_retention_notification_cancelled_at
                    .is_null(),
            ),
    )
    .set((
        account_retention_notifications::account_retention_notification_cancelled_at
            .eq(cancelled_at.to_diesel()),
        account_retention_notifications::account_retention_notification_claim_token
            .eq(Option::<Uuid>::None),
        account_retention_notifications::account_retention_notification_claimed_at
            .eq(Option::<Timestamp>::None.to_diesel()),
        account_retention_notifications::account_retention_notification_claim_expires_at
            .eq(Option::<Timestamp>::None.to_diesel()),
        account_retention_notifications::account_retention_notification_updated_at
            .eq(cancelled_at.to_diesel()),
    ))
    .execute(&mut *connection)
    .await?;
    Ok(())
}
