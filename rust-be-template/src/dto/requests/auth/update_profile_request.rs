use utoipa::ToSchema;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Full editable profile update; account email is intentionally absent.
#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct UpdateProfileRequest {
    #[schema(max_length = 128)]
    pub current_password: String,
    #[schema(max_length = 20)]
    pub user_name: String,
    pub user_country: i32,
    pub user_language: i32,
    pub user_subdivision: Option<i32>,
}
