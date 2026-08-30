//! Account deletion, retention, and permanent tombstone values.

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zeroize::Zeroizing;

/// The protected actor used for system-owned records and neutral ISO defaults.
pub const SYSTEM_ACTOR_USER_ID: Uuid = Uuid::from_u128(0);
/// Grace period before an administrator may irreversibly purge retained identity.
pub const ACCOUNT_RETENTION_DAYS: i64 = 30;
/// Maximum profile cleanup ledger rows processed by one admin request.
pub const PROFILE_CLEANUP_BATCH_SIZE: i64 = 256;
/// Maximum unresolved cleanup records exposed by one reconciliation request.
pub const MEDIA_CLEANUP_RECONCILIATION_LIMIT: i64 = 100;

/// Credential state required to confirm a self-service deletion.
pub struct AccountDeletionCandidate {
    pub user_id: Uuid,
    pub password_hash: Zeroizing<String>,
    pub is_system_actor: bool,
}

/// Private identity retained only for grace-period notification and lifecycle policy.
pub(crate) struct RetainedAccountIdentity {
    pub(crate) user_id: Uuid,
    pub(crate) user_name: String,
    pub(crate) email: String,
    pub(crate) country: i32,
    pub(crate) language: i32,
    pub(crate) subdivision: Option<i32>,
}

/// Receipt proving that an active account became a permanent authored-content tombstone.
#[derive(Debug, Clone)]
pub struct SoftDeleteAccountReceipt {
    pub user_id: Uuid,
    pub deleted_at: DateTime<Utc>,
    pub purge_after: DateTime<Utc>,
}

/// Receipt for the irreversible private-data purge.
#[derive(Debug, Clone)]
pub struct HardPurgeAccountReceipt {
    pub user_id: Uuid,
    pub hard_purged_at: DateTime<Utc>,
    /// Remote objects still backed by profile metadata until cleanup succeeds.
    pub profile_objects: Vec<ProfileObjectCleanup>,
    pub profile_metadata_deleted: usize,
}

/// Idempotent object-store cleanup item retained in PostgreSQL until finalization.
#[derive(Debug, Clone)]
pub struct ProfileObjectCleanup {
    pub profile_picture_id: Uuid,
    pub object_url: Option<String>,
}

/// Repository plan used to finalize non-cloud metadata without losing cloud cleanup work.
pub struct HardPurgeAccountPlan {
    pub user_id: Uuid,
    pub hard_purged_at: DateTime<Utc>,
    pub profile_objects: Vec<ProfileObjectCleanup>,
    pub non_cloud_profile_ids: Vec<Uuid>,
}

/// Exact profile cleanup state after an idempotent metadata finalization.
#[derive(Debug, Clone, Copy)]
pub struct ProfileCleanupFinalization {
    pub metadata_deleted: usize,
    pub metadata_remaining: usize,
}

/// Unresolved durable cleanup record exposed to an authorized reconciler.
#[derive(Debug, Clone)]
pub struct UnresolvedMediaCleanup {
    pub cleanup_id: Uuid,
    pub original_url: String,
    pub reason: String,
    pub source_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Durable cleanup record after its object address has been reconciled.
#[derive(Debug, Clone)]
pub struct ResolvedMediaCleanup {
    pub cleanup_id: Uuid,
    pub bucket: String,
    pub key: String,
    pub original_url: String,
}

/// Unique values that cannot authenticate and do not expose retained identity.
pub(crate) struct TombstoneIdentity {
    pub(crate) user_name: String,
    pub(crate) email: String,
    pub(crate) password_hash: String,
}

impl TombstoneIdentity {
    pub(crate) fn for_retention_id(retention_id: Uuid) -> Self {
        let identifier = retention_id.simple();
        Self {
            user_name: format!("deleted-{identifier}"),
            email: format!("deleted+{identifier}@account.invalid"),
            // This deliberately is not a password-hash encoding, so it is never a credential.
            password_hash: format!("!deleted:{identifier}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TombstoneIdentity;
    use uuid::Uuid;

    #[test]
    fn tombstone_values_are_unique_and_not_password_hashes() {
        let first = TombstoneIdentity::for_retention_id(Uuid::from_u128(1));
        let second = TombstoneIdentity::for_retention_id(Uuid::from_u128(2));

        assert_ne!(first.user_name, second.user_name);
        assert_ne!(first.email, second.email);
        assert_ne!(first.password_hash, second.password_hash);
        assert!(first.password_hash.starts_with("!deleted:"));
        assert!(!first.password_hash.starts_with("$argon2"));
    }
}
