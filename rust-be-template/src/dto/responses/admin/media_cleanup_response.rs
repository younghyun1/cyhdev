use jiff::Timestamp;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(serde_derive::Serialize, ToSchema)]
pub struct UnresolvedMediaCleanupResponse {
    pub records: Vec<UnresolvedMediaCleanupItem>,
}

#[derive(serde_derive::Serialize, ToSchema)]
pub struct UnresolvedMediaCleanupItem {
    pub cleanup_id: Uuid,
    pub original_url: String,
    pub reason: String,
    pub source_id: Uuid,
    pub created_at: Timestamp,
}

#[derive(serde_derive::Serialize, ToSchema)]
pub struct ResolveMediaCleanupResponse {
    pub cleanup_id: Uuid,
    pub bucket: String,
    pub key: String,
    pub original_url: String,
}
