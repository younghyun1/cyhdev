use utoipa::ToSchema;

#[derive(serde_derive::Deserialize, ToSchema)]
pub struct CreateForumReplyRequest {
    pub body: String,
}

#[derive(serde_derive::Deserialize, ToSchema)]
pub struct UpdateForumReplyRequest {
    pub body: String,
    pub expected_revision: i32,
}
