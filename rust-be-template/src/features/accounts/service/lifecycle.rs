//! Account lifecycle use cases and post-commit in-memory coordination.

use std::sync::Arc;

use chrono::{Days, Utc};
use futures_util::{StreamExt, stream};
use tracing::warn;
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::lifecycle::{
            ACCOUNT_RETENTION_DAYS, HardPurgeAccountOutcome, HardPurgeAccountReceipt,
            ProfileObjectCleanup, ProfileObjectCleanupFailure, SoftDeleteAccountReceipt,
        },
        domain::retention_notifications::RetentionNotificationSchedule,
        error::AccountError,
        service::{
            account_service::AccountService,
            authentication::password_within_auth_bound,
        },
    },
    util::{
        crypto::verify_pw::verify_pw,
        media::object_store::{MediaObjectStore, ObjectLocation},
        s3::AWS_S3_BUCKET_NAME,
    },
};

const PROFILE_OBJECT_DELETE_CONCURRENCY: usize = 8;

impl AccountService {
    /// Confirms the current password, commits a tombstone, then revokes all RAM sessions.
    pub async fn soft_delete_account(
        &self,
        user_id: Uuid,
        current_password: &str,
    ) -> Result<SoftDeleteAccountReceipt, AccountError> {
        if !password_within_auth_bound(current_password) {
            return Err(AccountError::InvalidPassword);
        }
        let session_consistency_read = self.session_consistency.read().await;
        let candidate = self.repository.account_deletion_candidate(user_id).await?;
        if candidate.is_system_actor {
            return Err(AccountError::SystemActorProtected);
        }
        let password_job = self.try_password_job()?;
        let password_matches = verify_pw(current_password, &candidate.password_hash)
            .await
            .map_err(AccountError::PasswordVerification)?;
        drop(password_job);
        if !password_matches {
            return Err(AccountError::WrongPassword);
        }
        drop(session_consistency_read);

        let session_consistency = self.session_consistency.write().await;
        let deleted_at = Utc::now();
        let retention_days = u64::try_from(ACCOUNT_RETENTION_DAYS)
            .map_err(|_| AccountError::RetentionScheduleOverflow)?;
        let purge_after = deleted_at
            .checked_add_days(Days::new(retention_days))
            .ok_or(AccountError::RetentionScheduleOverflow)?;
        let notification_schedule = RetentionNotificationSchedule::from_purge_after(purge_after)
            .ok_or(AccountError::RetentionScheduleOverflow)?;
        let receipt = self
            .repository
            .soft_delete_account(
                candidate.user_id,
                &candidate.password_hash,
                deleted_at,
                purge_after,
                notification_schedule,
            )
            .await?;

        self.sessions.remove_for_user(user_id).await;
        drop(session_consistency);
        self.live_chat_lifecycle
            .anonymize_deleted_account(user_id)
            .await;
        Ok(receipt)
    }

    /// Purges grace-period identity metadata without deleting the authored-content tombstone.
    pub async fn hard_purge_account(
        &self,
        requester_id: Uuid,
        user_id: Uuid,
    ) -> Result<HardPurgeAccountReceipt, AccountError> {
        let retention_delivery = self.retention_notification_delivery_gate.write().await;
        let session_consistency = self.session_consistency.write().await;
        let receipt = self
            .repository
            .hard_purge_account(requester_id, user_id, Utc::now())
            .await?;
        drop(retention_delivery);
        self.sessions.remove_for_user(user_id).await;
        let finalization = self
            .repository
            .finalize_profile_cleanup(requester_id, user_id, &receipt.non_cloud_profile_ids)
            .await?;
        drop(session_consistency);
        Ok(HardPurgeAccountReceipt {
            user_id: receipt.user_id,
            hard_purged_at: receipt.hard_purged_at,
            profile_objects: receipt.profile_objects,
            profile_metadata_deleted: finalization.metadata_deleted,
        })
    }

    /// Purges retained identity and executes bounded profile-object cleanup through the service port.
    pub async fn hard_purge_account_with_cleanup(
        &self,
        requester_id: Uuid,
        user_id: Uuid,
    ) -> Result<HardPurgeAccountOutcome, AccountError> {
        let receipt = self.hard_purge_account(requester_id, user_id).await?;
        let store = Arc::clone(&self.media_object_store);
        let cleanup_results = stream::iter(receipt.profile_objects.into_iter().map(
            |profile_object| {
                delete_profile_object(Arc::clone(&store), user_id, profile_object)
            },
        ))
        .buffer_unordered(PROFILE_OBJECT_DELETE_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;
        let mut deleted_profile_ids = Vec::with_capacity(cleanup_results.len());
        let mut failures = Vec::new();
        for result in cleanup_results {
            match result {
                Ok(profile_picture_id) => deleted_profile_ids.push(profile_picture_id),
                Err(failure) => failures.push(failure),
            }
        }
        let profile_objects_deleted = deleted_profile_ids.len();
        let finalized = self
            .finalize_profile_cleanup(requester_id, user_id, &deleted_profile_ids)
            .await?;
        Ok(HardPurgeAccountOutcome {
            user_id: receipt.user_id,
            hard_purged_at: receipt.hard_purged_at,
            profile_objects_deleted,
            profile_metadata_deleted: receipt
                .profile_metadata_deleted
                .saturating_add(finalized.metadata_deleted),
            profile_cleanup_remaining: finalized.metadata_remaining,
            profile_cleanup_failures: failures,
        })
    }

    /// Removes profile metadata after confirmed idempotent object-store deletion.
    pub async fn finalize_profile_cleanup(
        &self,
        requester_id: Uuid,
        user_id: Uuid,
        profile_picture_ids: &[Uuid],
    ) -> Result<crate::features::accounts::domain::lifecycle::ProfileCleanupFinalization, AccountError>
    {
        let session_consistency = self.session_consistency.read().await;
        let finalization = self
            .repository
            .finalize_profile_cleanup(requester_id, user_id, profile_picture_ids)
            .await;
        drop(session_consistency);
        finalization
    }
}

async fn delete_profile_object(
    store: Arc<dyn MediaObjectStore>,
    user_id: Uuid,
    profile_object: ProfileObjectCleanup,
) -> Result<Uuid, ProfileObjectCleanupFailure> {
    let object_url = profile_object.object_url.ok_or_else(|| ProfileObjectCleanupFailure {
        profile_picture_id: profile_object.profile_picture_id,
        object_url: None,
        reason: "profile metadata has no object URL".to_owned(),
        retryable: false,
    })?;
    let location = ObjectLocation::from_public_s3_url(AWS_S3_BUCKET_NAME, &object_url)
        .ok_or_else(|| ProfileObjectCleanupFailure {
            profile_picture_id: profile_object.profile_picture_id,
            object_url: Some(object_url.clone()),
            reason: "profile metadata has an invalid object URL".to_owned(),
            retryable: false,
        })?;
    match store.delete(location).await {
        Ok(()) => Ok(profile_object.profile_picture_id),
        Err(error) => {
            warn!(
                user_id = %user_id,
                profile_picture_id = %profile_object.profile_picture_id,
                retryable = error.is_retryable(),
                error = %error,
                "Profile object cleanup failed after hard purge"
            );
            Err(ProfileObjectCleanupFailure {
                profile_picture_id: profile_object.profile_picture_id,
                object_url: Some(object_url),
                reason: "object-store deletion failed".to_owned(),
                retryable: error.is_retryable(),
            })
        }
    }
}
