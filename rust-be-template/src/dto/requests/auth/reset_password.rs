use utoipa::ToSchema;
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct ResetPasswordProcessRequest {
    #[zeroize(skip)]
    pub password_reset_token: Uuid,
    #[schema(max_length = 128)]
    pub new_password: String,
}
