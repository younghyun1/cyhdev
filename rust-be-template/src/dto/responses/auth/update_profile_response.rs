use utoipa::ToSchema;
use uuid::Uuid;

#[derive(serde_derive::Serialize, ToSchema)]
pub struct UpdateProfileResponse {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_country: i32,
    pub user_language: i32,
    pub user_subdivision: Option<i32>,
}
