use chrono::{DateTime, Utc};
use serde_derive::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::features::forum::domain::models::{
    ForumMutationReceipt, ForumReplyCursor, ForumReplyView, ForumTopicCursor, ForumTopicView,
};

use super::common::{
    ForumAuthorResponse, ForumContentStateResponse, ForumTopicAccessStateResponse,
};

#[derive(Serialize, ToSchema)]
pub struct ForumTopicResponse {
    pub topic_id: Uuid,
    pub author: ForumAuthorResponse,
    #[schema(required)]
    pub title: Option<String>,
    #[schema(required)]
    pub body: Option<String>,
    pub content_state: ForumContentStateResponse,
    pub access_state: ForumTopicAccessStateResponse,
    pub is_pinned: bool,
    pub revision: i32,
    pub reply_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    #[schema(required)]
    pub edited_at: Option<DateTime<Utc>>,
}

impl From<ForumTopicView> for ForumTopicResponse {
    fn from(view: ForumTopicView) -> Self {
        let topic = view.topic;
        Self {
            topic_id: topic.topic_id,
            author: view.author.into(),
            title: topic.title,
            body: topic.body,
            content_state: topic.content_state.into(),
            access_state: topic.access_state.into(),
            is_pinned: topic.is_pinned,
            revision: topic.revision,
            reply_count: topic.reply_count,
            created_at: topic.created_at,
            updated_at: topic.updated_at,
            last_activity_at: topic.last_activity_at,
            edited_at: topic.edited_at,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForumReplyResponse {
    pub reply_id: Uuid,
    pub topic_id: Uuid,
    pub author: ForumAuthorResponse,
    #[schema(required)]
    pub body: Option<String>,
    pub content_state: ForumContentStateResponse,
    pub revision: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[schema(required)]
    pub edited_at: Option<DateTime<Utc>>,
}

impl From<ForumReplyView> for ForumReplyResponse {
    fn from(view: ForumReplyView) -> Self {
        let reply = view.reply;
        Self {
            reply_id: reply.reply_id,
            topic_id: reply.topic_id,
            author: view.author.into(),
            body: reply.body,
            content_state: reply.content_state.into(),
            revision: reply.revision,
            created_at: reply.created_at,
            updated_at: reply.updated_at,
            edited_at: reply.edited_at,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForumTopicCursorResponse {
    pub before_pinned: bool,
    pub before_activity_at: DateTime<Utc>,
    pub before_topic_id: Uuid,
}
impl From<ForumTopicCursor> for ForumTopicCursorResponse {
    fn from(value: ForumTopicCursor) -> Self {
        Self {
            before_pinned: value.is_pinned,
            before_activity_at: value.last_activity_at,
            before_topic_id: value.topic_id,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForumReplyCursorResponse {
    pub after_reply_created_at: DateTime<Utc>,
    pub after_reply_id: Uuid,
}
impl From<ForumReplyCursor> for ForumReplyCursorResponse {
    fn from(value: ForumReplyCursor) -> Self {
        Self {
            after_reply_created_at: value.created_at,
            after_reply_id: value.reply_id,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForumTopicListResponse {
    pub topics: Vec<ForumTopicResponse>,
    #[schema(required)]
    pub next_cursor: Option<ForumTopicCursorResponse>,
}

#[derive(Serialize, ToSchema)]
pub struct ForumTopicDetailResponse {
    pub topic: ForumTopicResponse,
    pub replies: Vec<ForumReplyResponse>,
    #[schema(required)]
    pub next_reply_cursor: Option<ForumReplyCursorResponse>,
    pub is_subscribed: bool,
}

#[derive(Serialize, ToSchema)]
pub struct ForumTopicMutationResponse {
    pub topic_id: Uuid,
    pub revision: i32,
    pub updated_at: DateTime<Utc>,
}
impl From<ForumMutationReceipt> for ForumTopicMutationResponse {
    fn from(value: ForumMutationReceipt) -> Self {
        Self {
            topic_id: value.item_id,
            revision: value.revision,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForumReplyMutationResponse {
    pub reply_id: Uuid,
    pub revision: i32,
    pub updated_at: DateTime<Utc>,
}
impl From<ForumMutationReceipt> for ForumReplyMutationResponse {
    fn from(value: ForumMutationReceipt) -> Self {
        Self {
            reply_id: value.item_id,
            revision: value.revision,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForumSubscriptionResponse {
    pub topic_id: Uuid,
    pub subscribed: bool,
}

#[derive(Serialize, ToSchema)]
pub struct ForumCapabilitiesResponse {
    pub authenticated: bool,
    pub can_post: bool,
    pub can_moderate: bool,
}
