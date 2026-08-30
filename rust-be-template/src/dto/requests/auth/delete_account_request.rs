use utoipa::ToSchema;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Current credential required to authorize self-service account deletion.
#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct DeleteAccountRequest {
    #[schema(max_length = 128)]
    pub current_password: String,
}
