use chrono::{DateTime, Utc};
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(serde_derive::Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ForumTopicListQuery {
    pub search: Option<String>,
    pub before_pinned: Option<bool>,
    pub before_activity_at: Option<DateTime<Utc>>,
    pub before_topic_id: Option<Uuid>,
    pub limit: Option<u16>,
}

#[derive(serde_derive::Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ForumTopicDetailQuery {
    pub after_reply_created_at: Option<DateTime<Utc>>,
    pub after_reply_id: Option<Uuid>,
    pub reply_limit: Option<u16>,
}

#[derive(serde_derive::Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ForumNotificationListQuery {
    pub before_created_at: Option<DateTime<Utc>>,
    pub before_notification_id: Option<Uuid>,
    pub limit: Option<u16>,
}

#[derive(serde_derive::Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ForumModerationAuditListQuery {
    pub before_created_at: Option<DateTime<Utc>>,
    pub before_audit_id: Option<Uuid>,
    pub limit: Option<u16>,
}
