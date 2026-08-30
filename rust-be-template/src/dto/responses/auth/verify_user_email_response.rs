use chrono::{DateTime, Utc};
use utoipa::ToSchema;

/// Public result of an explicitly confirmed email-verification mutation.
#[derive(serde_derive::Serialize, ToSchema)]
pub struct VerifyUserEmailResponse {
    pub verified_at: DateTime<Utc>,
}
