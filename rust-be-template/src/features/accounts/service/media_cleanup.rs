//! Authorized durable-media reconciliation use cases.

use uuid::Uuid;

use crate::features::accounts::{
    domain::lifecycle::{ResolvedMediaCleanup, UnresolvedMediaCleanup},
    error::AccountError,
    service::account_service::AccountService,
};

impl AccountService {
    pub async fn unresolved_media_cleanup(
        &self,
        requester_id: Uuid,
    ) -> Result<Vec<UnresolvedMediaCleanup>, AccountError> {
        let session_consistency = self.session_consistency.read().await;
        let records = self
            .repository
            .unresolved_media_cleanup(requester_id)
            .await;
        drop(session_consistency);
        records
    }

    pub async fn resolve_media_cleanup(
        &self,
        requester_id: Uuid,
        cleanup_id: Uuid,
        expected_original_url: &str,
        bucket: &str,
        key: &str,
    ) -> Result<ResolvedMediaCleanup, AccountError> {
        validate_location(expected_original_url, bucket, key)?;
        let session_consistency = self.session_consistency.read().await;
        let resolution = self
            .repository
            .resolve_media_cleanup(
                requester_id,
                cleanup_id,
                expected_original_url,
                bucket,
                key,
            )
            .await;
        drop(session_consistency);
        resolution
    }
}

fn validate_location(original_url: &str, bucket: &str, key: &str) -> Result<(), AccountError> {
    let valid = !original_url.is_empty()
        && original_url.len() <= 4_096
        && !bucket.is_empty()
        && bucket.len() <= 255
        && !key.is_empty()
        && key.len() <= 1_024;
    if valid {
        Ok(())
    } else {
        Err(AccountError::InvalidMediaCleanupLocation)
    }
}
