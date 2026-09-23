use utoipa::ToSchema;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct OidcLinkCompleteRequest {
    pub completion_token: String,
}

/// Current-password confirmation required before a provider link may start.
#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct OidcLinkStartRequest {
    #[schema(max_length = 128)]
    pub current_password: String,
}

#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct OidcUnlinkRequest {
    pub current_password: String,
}
