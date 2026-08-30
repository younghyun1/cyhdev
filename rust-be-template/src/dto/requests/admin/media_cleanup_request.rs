use utoipa::ToSchema;

/// Optimistic reconciliation of an unresolved legacy object address.
#[derive(serde_derive::Deserialize, ToSchema)]
pub struct ResolveMediaCleanupRequest {
    pub expected_original_url: String,
    pub bucket: String,
    pub key: String,
}
