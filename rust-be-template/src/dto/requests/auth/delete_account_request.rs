use utoipa::ToSchema;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Current credential required to authorize self-service account deletion.
#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct DeleteAccountRequest {
    pub current_password: String,
}
