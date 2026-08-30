use serde_derive::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::features::{
    accounts::domain::public_author::PublicAuthor,
    forum::domain::enums::{ForumContentState, ForumNotificationKind, ForumTopicAccessState},
};

#[derive(Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ForumContentStateResponse { Visible, Hidden, Deleted }

impl From<ForumContentState> for ForumContentStateResponse {
    fn from(value: ForumContentState) -> Self { match value { ForumContentState::Visible => Self::Visible, ForumContentState::Hidden => Self::Hidden, ForumContentState::Deleted => Self::Deleted } }
}

#[derive(Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ForumTopicAccessStateResponse { Open, Locked }

impl From<ForumTopicAccessState> for ForumTopicAccessStateResponse {
    fn from(value: ForumTopicAccessState) -> Self { match value { ForumTopicAccessState::Open => Self::Open, ForumTopicAccessState::Locked => Self::Locked } }
}

#[derive(Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ForumNotificationKindResponse { TopicReply }

impl From<ForumNotificationKind> for ForumNotificationKindResponse {
    fn from(_: ForumNotificationKind) -> Self { Self::TopicReply }
}

#[derive(Serialize, ToSchema)]
pub struct ForumAuthorResponse {
    pub public_user_id: Uuid,
    pub display_name: String,
    #[schema(required)]
    pub country_code: Option<i32>,
    #[schema(required)]
    pub profile_picture_url: Option<String>,
    pub is_deleted: bool,
}

impl From<PublicAuthor> for ForumAuthorResponse {
    fn from(author: PublicAuthor) -> Self {
        Self {
            public_user_id: author.public_user_id(),
            display_name: author.user_name().to_owned(),
            country_code: author.country_code(),
            profile_picture_url: (!author.profile_picture_url().is_empty())
                .then(|| author.profile_picture_url().to_owned()),
            is_deleted: author.is_deleted(),
        }
    }
}
