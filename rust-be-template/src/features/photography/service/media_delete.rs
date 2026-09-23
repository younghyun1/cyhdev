//! Bulk photograph deletion with durable object cleanup.

use uuid::Uuid;

use super::{media::log_cleanup, photography_service::PhotographyService};
use crate::{
    features::photography::{domain::media::PhotographDeleteReport, error::PhotographyError},
    util::media::{cleanup::settle_durable_cleanup, persistence::cleanup_committed_objects},
};

const MAX_DELETE_PHOTOGRAPHS: usize = 1_000;
const PHOTOGRAPH_CLEANUP_CONCURRENCY: usize = 8;

impl PhotographyService {
    pub async fn delete_photographs(
        &self,
        requester_id: Uuid,
        mut ids: Vec<Uuid>,
    ) -> Result<PhotographDeleteReport, PhotographyError> {
        normalize_ids(&mut ids)?;
        if ids.is_empty() {
            return Ok(PhotographDeleteReport {
                deleted_count: 0,
                s3_deleted_count: 0,
                cleanup_failure_count: 0,
                cleanup_remaining_count: 0,
                unresolved_cleanup_count: 0,
            });
        }
        let retired = self
            .repository
            .retire_photographs(requester_id, &ids)
            .await?;
        let cleanup_total = retired.cleanup.resolved.len() + retired.cleanup.unresolved_count;
        let locations = retired
            .cleanup
            .resolved
            .iter()
            .map(|cleanup| cleanup.location.clone())
            .collect();
        let (cleaned, failures) = cleanup_committed_objects(
            self.media.object_store.as_ref(),
            locations,
            PHOTOGRAPH_CLEANUP_CONCURRENCY,
        )
        .await;
        log_cleanup(&failures);
        let settlement = settle_durable_cleanup(
            &self.media.accounts,
            retired.cleanup.resolved,
            &cleaned,
            &failures,
        )
        .await;
        Ok(PhotographDeleteReport {
            deleted_count: retired.deleted_rows,
            s3_deleted_count: cleaned.len(),
            cleanup_failure_count: failures.len() + settlement.ledger_errors,
            cleanup_remaining_count: cleanup_total.saturating_sub(settlement.finalized),
            unresolved_cleanup_count: retired.cleanup.unresolved_count,
        })
    }
}

fn normalize_ids(ids: &mut Vec<Uuid>) -> Result<(), PhotographyError> {
    ids.sort_unstable();
    ids.dedup();
    if ids.len() > MAX_DELETE_PHOTOGRAPHS {
        Err(PhotographyError::InvalidInput)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_DELETE_PHOTOGRAPHS, normalize_ids};
    use uuid::Uuid;
    #[test]
    fn deletion_deduplicates_before_cap() {
        let mut ids = vec![Uuid::nil(); MAX_DELETE_PHOTOGRAPHS + 1];
        assert!(normalize_ids(&mut ids).is_ok());
        assert_eq!(ids.len(), 1);
    }
    #[test]
    fn deletion_rejects_distinct_overflow() {
        let mut ids = (0..=MAX_DELETE_PHOTOGRAPHS)
            .map(|value| Uuid::from_u128(value as u128))
            .collect();
        assert!(normalize_ids(&mut ids).is_err());
    }
}
