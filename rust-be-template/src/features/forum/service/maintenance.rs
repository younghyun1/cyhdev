//! Scheduled forum maintenance.

use std::time::{Duration, Instant};

use chrono::Utc;

use crate::features::forum::{
    domain::models::ForumNotificationPruneReport, error::ForumError,
    service::forum_service::ForumService,
};

/// Wall-clock budget for one hourly cleanup run. One reply can fan out to
/// 4,095 notifications, so a single 512-row batch per hour could never keep
/// up; repeated short batches drain the backlog without one long transaction.
const NOTIFICATION_PRUNE_BUDGET: Duration = Duration::from_secs(5);
/// Upper bound on batches per run, independent of how fast each one is.
const NOTIFICATION_PRUNE_MAX_BATCHES: usize = 256;

impl ForumService {
    /// Deletes expired notifications in bounded batches until none remain,
    /// the time budget is spent, or the batch cap is reached. Each batch is
    /// its own short transaction; the report carries the total deleted and
    /// whether expired rows are still left for the next run.
    pub async fn prune_notifications(&self) -> Result<ForumNotificationPruneReport, ForumError> {
        let started = Instant::now();
        let now = Utc::now();
        let mut deleted = 0_usize;
        for _ in 0..NOTIFICATION_PRUNE_MAX_BATCHES {
            let batch = self.repository.prune_expired_notifications(now).await?;
            deleted = deleted.saturating_add(batch.deleted);
            if !batch.remaining_expired
                || batch.deleted == 0
                || started.elapsed() >= NOTIFICATION_PRUNE_BUDGET
            {
                return Ok(ForumNotificationPruneReport {
                    deleted,
                    remaining_expired: batch.remaining_expired,
                });
            }
        }
        Ok(ForumNotificationPruneReport {
            deleted,
            remaining_expired: true,
        })
    }
}
