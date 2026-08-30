use utoipa::ToSchema;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct OidcLinkCompleteRequest {
    pub completion_token: String,
}

#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct OidcUnlinkRequest {
    pub current_password: String,
}
