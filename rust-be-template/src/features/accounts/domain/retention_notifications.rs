//! Durable staged notification state for retained account identity.

use chrono::{DateTime, Days, TimeDelta, Utc};
use serde_derive::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;
use zeroize::Zeroizing;

pub const RETENTION_NOTIFICATION_BATCH_SIZE: i64 = 32;
pub const RETENTION_NOTIFICATION_DELIVERY_CONCURRENCY: usize = 4;
pub const RETENTION_NOTIFICATION_STATUS_LIMIT: i64 = 100;
pub const RETENTION_NOTIFICATION_CLAIM_MINUTES: i64 = 10;
pub const RETENTION_NOTIFICATION_ERROR_CHARS: usize = 512;
const RETRY_BASE_MINUTES: i64 = 5;
const RETRY_MAX_EXPONENT: i32 = 8;

/// Fixed notice stages keyed uniquely per retained account.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RetentionNotificationStage {
    SevenDaysBeforePurge,
    OneDayBeforePurge,
}

impl RetentionNotificationStage {
    pub const fn days_before_purge(self) -> u64 {
        match self {
            Self::SevenDaysBeforePurge => 7,
            Self::OneDayBeforePurge => 1,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SevenDaysBeforePurge => "seven_days_before_purge",
            Self::OneDayBeforePurge => "one_day_before_purge",
        }
    }
}

/// Both delivery deadlines derived from one immutable purge deadline.
pub struct RetentionNotificationSchedule {
    pub seven_days_before_purge: DateTime<Utc>,
    pub one_day_before_purge: DateTime<Utc>,
}

impl RetentionNotificationSchedule {
    pub fn from_purge_after(purge_after: DateTime<Utc>) -> Option<Self> {
        Some(Self {
            seven_days_before_purge: purge_after.checked_sub_days(Days::new(7))?,
            one_day_before_purge: purge_after.checked_sub_days(Days::new(1))?,
        })
    }
}

/// Stable keyset position for bounded due/status scans.
#[derive(Debug, Clone, Copy)]
pub struct RetentionNotificationCursor {
    pub next_attempt_at: DateTime<Utc>,
    pub notification_id: Uuid,
}

/// One transactionally claimed row without retained email material.
#[derive(Debug, Clone)]
pub struct ClaimedRetentionNotification {
    pub notification_id: Uuid,
    pub stage: RetentionNotificationStage,
    pub next_attempt_at: DateTime<Utc>,
    pub attempt_count: i32,
}

/// One keyset batch protected by a shared expiring claim token.
pub struct RetentionNotificationClaimBatch {
    pub claim_token: Uuid,
    pub notifications: Vec<ClaimedRetentionNotification>,
}

/// Revalidated private delivery material, zeroized when the batch is dropped.
pub struct RetentionNotificationDeliveryItem {
    pub notification_id: Uuid,
    pub stage: RetentionNotificationStage,
    pub retained_email: Zeroizing<String>,
    pub purge_after: DateTime<Utc>,
    pub attempt_count: i32,
}

/// Identity-minimized state exposed to an authorized administrator.
#[derive(Debug, Clone)]
pub struct RetentionNotificationStatus {
    pub notification_id: Uuid,
    pub user_id: Uuid,
    pub stage: RetentionNotificationStage,
    pub scheduled_for: DateTime<Utc>,
    pub next_attempt_at: DateTime<Utc>,
    pub attempt_count: i32,
    pub claim_expires_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct RetentionNotificationRetryReceipt {
    pub notification_id: Uuid,
    pub next_attempt_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RetentionNotificationRunReport {
    pub claimed: usize,
    pub delivered: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Capped exponential delay after the attempt already recorded by the claim.
pub fn retention_notification_retry_delay(attempt_count: i32) -> TimeDelta {
    let bounded_exponent = attempt_count
        .saturating_sub(1)
        .clamp(0, RETRY_MAX_EXPONENT);
    let exponent = match u32::try_from(bounded_exponent) {
        Ok(exponent) => exponent,
        Err(error) => {
            tracing::error!(
                bounded_exponent,
                error = %error,
                "Could not convert a clamped retention retry exponent"
            );
            0
        }
    };
    TimeDelta::minutes(RETRY_BASE_MINUTES * (1_i64 << exponent))
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{
        RetentionNotificationSchedule, retention_notification_retry_delay,
    };

    #[test]
    fn schedule_and_retry_bounds_are_exact() -> Result<(), &'static str> {
        let purge_after = Utc
            .with_ymd_and_hms(2026, 9, 30, 12, 0, 0)
            .single()
            .ok_or("fixed UTC test timestamp was invalid")?;
        let schedule = RetentionNotificationSchedule::from_purge_after(purge_after)
            .ok_or("fixed purge deadline did not produce a schedule")?;
        assert_eq!(
            purge_after - schedule.seven_days_before_purge,
            chrono::Duration::days(7)
        );
        assert_eq!(
            purge_after - schedule.one_day_before_purge,
            chrono::Duration::days(1)
        );
        assert_eq!(retention_notification_retry_delay(1).num_minutes(), 5);
        assert_eq!(retention_notification_retry_delay(2).num_minutes(), 10);
        assert_eq!(retention_notification_retry_delay(i32::MAX).num_minutes(), 1_280);
        Ok(())
    }
}
