use jiff::Timestamp;
use utoipa::ToSchema;
use uuid::Uuid;

/// Grace-period schedule created by a successful account deletion.
#[derive(serde_derive::Serialize, ToSchema)]
pub struct DeleteAccountResponse {
    pub user_id: Uuid,
    pub deleted_at: Timestamp,
    pub purge_after: Timestamp,
}
