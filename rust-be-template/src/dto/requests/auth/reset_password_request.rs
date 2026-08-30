use utoipa::ToSchema;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct ResetPasswordRequest {
    #[schema(max_length = 254)]
    pub user_email: String,
}
