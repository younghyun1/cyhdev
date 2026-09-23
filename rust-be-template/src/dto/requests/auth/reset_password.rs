use utoipa::ToSchema;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct ResetPasswordProcessRequest {
    /// Capability from the emailed link fragment: 43 unpadded base64url characters.
    #[schema(min_length = 43, max_length = 43, pattern = "^[A-Za-z0-9_-]{43}$")]
    pub password_reset_token: String,
    #[schema(max_length = 128)]
    pub new_password: String,
}
