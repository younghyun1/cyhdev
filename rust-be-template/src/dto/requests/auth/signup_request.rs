use utoipa::ToSchema;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct SignupRequest {
    #[schema(max_length = 20)]
    pub user_name: String,
    #[schema(max_length = 254)]
    pub user_email: String,
    #[schema(max_length = 128)]
    pub user_password: String,
    pub user_country: i32,
    pub user_language: i32,
    pub user_subdivision: Option<i32>,
}
