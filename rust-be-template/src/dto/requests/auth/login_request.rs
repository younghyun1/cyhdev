use utoipa::ToSchema;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct LoginRequest {
    #[schema(max_length = 254)]
    pub user_email: String,
    #[schema(max_length = 128)]
    pub user_password: String,
}
