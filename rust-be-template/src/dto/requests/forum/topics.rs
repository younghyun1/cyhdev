use utoipa::ToSchema;

#[derive(serde_derive::Deserialize, ToSchema)]
pub struct CreateForumTopicRequest {
    pub title: String,
    pub body: String,
}

#[derive(serde_derive::Deserialize, ToSchema)]
pub struct UpdateForumTopicRequest {
    pub title: String,
    pub body: String,
    pub expected_revision: i32,
}

#[derive(serde_derive::Deserialize, ToSchema)]
pub struct DeleteForumContentRequest {
    pub expected_revision: i32,
}
