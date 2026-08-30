use chrono::{DateTime, Utc};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::features::accounts::domain::retention_notifications::RetentionNotificationCursor;

#[derive(serde_derive::Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct RetentionNotificationStatusRequest {
    pub after_next_attempt_at: Option<DateTime<Utc>>,
    pub after_notification_id: Option<Uuid>,
    pub limit: Option<u16>,
}

#[derive(Debug)]
pub struct InvalidRetentionNotificationCursor;

impl RetentionNotificationStatusRequest {
    pub fn cursor(
        &self,
    ) -> Result<Option<RetentionNotificationCursor>, InvalidRetentionNotificationCursor> {
        match (self.after_next_attempt_at, self.after_notification_id) {
            (Some(next_attempt_at), Some(notification_id)) => {
                Ok(Some(RetentionNotificationCursor {
                    next_attempt_at,
                    notification_id,
                }))
            }
            (None, None) => Ok(None),
            _ => Err(InvalidRetentionNotificationCursor),
        }
    }

    pub fn requested_limit(&self) -> i64 {
        i64::from(self.limit.unwrap_or(50))
    }
}
