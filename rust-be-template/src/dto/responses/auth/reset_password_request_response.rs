use utoipa::ToSchema;

#[derive(serde_derive::Serialize, ToSchema)]
pub struct ResetPasswordRequestResponse {
    pub message: String,
}
