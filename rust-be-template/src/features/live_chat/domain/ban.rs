use chrono::{DateTime, Utc};
use ipnet::IpNet;
use uuid::Uuid;

pub const LIVE_CHAT_BAN_SOURCE_ABNORMAL_MESSAGING: &str = "abnormal_messaging";

#[derive(Debug, Clone)]
pub struct LiveChatBan {
    pub live_chat_ban_id: Uuid,
    pub user_id: Option<Uuid>,
    pub banned_ip: Option<IpNet>,
    pub reason: String,
    pub ban_source: String,
    pub banned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
