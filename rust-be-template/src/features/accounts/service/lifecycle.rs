//! Account lifecycle use cases and post-commit in-memory coordination.

use chrono::{Days, Utc};
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::lifecycle::{
            ACCOUNT_RETENTION_DAYS, HardPurgeAccountReceipt, SoftDeleteAccountReceipt,
        },
        error::AccountError,
        service::account_service::AccountService,
    },
    util::crypto::verify_pw::verify_pw,
};

impl AccountService {
    /// Confirms the current password, commits a tombstone, then revokes all RAM sessions.
    pub async fn soft_delete_account(
        &self,
        user_id: Uuid,
        current_password: &str,
    ) -> Result<SoftDeleteAccountReceipt, AccountError> {
        let session_consistency_read = self.session_consistency.read().await;
        let candidate = self.repository.account_deletion_candidate(user_id).await?;
        if candidate.is_system_actor {
            return Err(AccountError::SystemActorProtected);
        }
        let password_matches = verify_pw(current_password, &candidate.password_hash)
            .await
            .map_err(AccountError::PasswordVerification)?;
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
        let receipt = self
            .repository
            .soft_delete_account(
                candidate.user_id,
                &candidate.password_hash,
                deleted_at,
                purge_after,
            )
            .await?;

        self.sessions.remove_for_user(user_id).await;
        drop(session_consistency);
        self.live_chat_cache.anonymize_deleted_user(user_id).await;
        Ok(receipt)
    }

    /// Purges grace-period identity metadata without deleting the authored-content tombstone.
    pub async fn hard_purge_account(
        &self,
        requester_id: Uuid,
        user_id: Uuid,
    ) -> Result<HardPurgeAccountReceipt, AccountError> {
        let session_consistency = self.session_consistency.write().await;
        let receipt = self
            .repository
            .hard_purge_account(requester_id, user_id, Utc::now())
            .await?;
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
