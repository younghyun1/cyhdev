use chrono::{DateTime, Duration, Utc};
use ipnet::IpNet;
use uuid::Uuid;

pub const LIVE_CHAT_BAN_SOURCE_ABNORMAL_MESSAGING: &str = "abnormal_messaging";

/// Automatic abuse bans expire so a false positive, or a later holder of the
/// same address or IPv6 prefix, is not locked out forever. Manual bans keep
/// their own expiry, including none.
pub const LIVE_CHAT_ABUSE_BAN_DURATION: Duration = Duration::hours(24);

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
