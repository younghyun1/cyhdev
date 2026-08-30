use serde_derive::Deserialize;
use utoipa::ToSchema;

#[derive(Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ForumTopicModerationActionRequest {
    Hide,
    Restore,
    Lock,
    Unlock,
    Pin,
    Unpin,
}

#[derive(Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ForumReplyModerationActionRequest {
    Hide,
    Restore,
}

#[derive(Deserialize, ToSchema)]
pub struct ModerateForumTopicRequest {
    pub action: ForumTopicModerationActionRequest,
    pub reason: String,
    pub expected_revision: i32,
}

#[derive(Deserialize, ToSchema)]
pub struct ModerateForumReplyRequest {
    pub action: ForumReplyModerationActionRequest,
    pub reason: String,
    pub expected_revision: i32,
}
