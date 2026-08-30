use chrono::{DateTime, Utc};
use uuid::Uuid;

pub const DEFAULT_LIVE_CHAT_ROOM: &str = "main";
pub const LIVE_CHAT_SENDER_KIND_GUEST: i16 = 0;
pub const LIVE_CHAT_SENDER_KIND_USER: i16 = 1;

#[derive(Debug, Clone)]
pub struct LiveChatMessage {
    pub live_chat_message_id: Uuid,
    pub room_key: String,
    pub user_id: Option<Uuid>,
    pub guest_ip: Option<ipnet::IpNet>,
    pub sender_kind: i16,
    pub sender_display_name: String,
    pub message_body: String,
    pub message_created_at: DateTime<Utc>,
    pub message_edited_at: Option<DateTime<Utc>>,
    pub message_deleted_at: Option<DateTime<Utc>>,
}
