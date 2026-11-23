use jiff::Timestamp;
use utoipa::ToSchema;

#[derive(serde_derive::Serialize, ToSchema)]
pub struct ResetPasswordResponse {
    pub user_id: uuid::Uuid,
    pub user_name: String,
    pub user_email: String,
    pub user_updated_at: Timestamp,
}
