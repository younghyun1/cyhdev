use serde_derive::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct OidcStatusResponse {
    pub enabled: bool,
    #[schema(required)]
    pub provider_name: Option<String>,
    pub linked: bool,
}

#[derive(Serialize, ToSchema)]
pub struct OidcAuthorizationResponse {
    pub authorization_url: String,
}

#[derive(Serialize, ToSchema)]
pub struct OidcLinkResponse {
    pub linked: bool,
}
