use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use uuid::Uuid;

/// Irreversible identity purge result and explicit remote-object cleanup work.
#[derive(serde_derive::Serialize, ToSchema)]
pub struct HardPurgeAccountResponse {
    pub user_id: Uuid,
    pub hard_purged_at: DateTime<Utc>,
    pub profile_objects_deleted: usize,
    pub profile_metadata_deleted: usize,
    pub profile_cleanup_remaining: usize,
    pub profile_cleanup_failures: Vec<ProfileObjectCleanupFailure>,
}

/// Profile object that remains retryable through retained metadata.
#[derive(serde_derive::Serialize, ToSchema)]
pub struct ProfileObjectCleanupFailure {
    pub profile_picture_id: Uuid,
    pub object_url: Option<String>,
    pub reason: String,
    pub retryable: bool,
}
