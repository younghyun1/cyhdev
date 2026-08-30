//! Durable object cleanup after profile-picture history deletion.

use std::sync::Arc;

use tracing::error;
use uuid::Uuid;

use crate::{
    features::accounts::{error::AccountError, service::account_service::AccountService},
    util::media::{
        cleanup::settle_durable_cleanup,
        persistence::cleanup_committed_objects,
    },
};

const PROFILE_CLEANUP_CONCURRENCY: usize = 4;

pub struct ProfilePictureCleanupReceipt {
    pub deleted_profile_picture_id: Uuid,
    pub active_profile_picture_id: Option<Uuid>,
    pub cleanup_deleted_count: usize,
    pub cleanup_failure_count: usize,
    pub cleanup_remaining_count: usize,
}

impl AccountService {
    pub async fn delete_profile_picture_and_cleanup(
        self: &Arc<Self>,
        user_id: Uuid,
        profile_picture_id: Uuid,
    ) -> Result<Option<ProfilePictureCleanupReceipt>, AccountError> {
        let deletion = match self
            .delete_profile_picture(user_id, profile_picture_id)
            .await?
        {
            Some(deletion) => deletion,
            None => return Ok(None),
        };
        let cleanup_total = deletion
            .cleanup_objects
            .len()
            .saturating_add(deletion.unresolved_cleanup_count);
        let locations = deletion
            .cleanup_objects
            .iter()
            .map(|cleanup| cleanup.location.clone())
            .collect();
        let (cleaned, failures) = cleanup_committed_objects(
            self.media_object_store.as_ref(),
            locations,
            PROFILE_CLEANUP_CONCURRENCY,
        )
        .await;
        for failure in &failures {
            error!(
                user_id = %user_id,
                profile_picture_id = %profile_picture_id,
                key = %failure.location.key(),
                retryable = failure.is_retryable(),
                error = %failure.error,
                "Profile-picture object cleanup remains pending"
            );
        }
        let settlement = settle_durable_cleanup(
            self,
            deletion.cleanup_objects,
            &cleaned,
            &failures,
        )
        .await;
        Ok(Some(ProfilePictureCleanupReceipt {
            deleted_profile_picture_id: deletion.deleted_profile_picture_id,
            active_profile_picture_id: deletion.active_profile_picture_id,
            cleanup_deleted_count: cleaned.len(),
            cleanup_failure_count: failures.len().saturating_add(settlement.ledger_errors),
            cleanup_remaining_count: cleanup_total.saturating_sub(settlement.finalized),
        }))
    }
}
