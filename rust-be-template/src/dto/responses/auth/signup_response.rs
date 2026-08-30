use utoipa::ToSchema;

#[derive(serde_derive::Serialize, ToSchema)]
pub struct SignupResponse {
    pub message: String,
}
