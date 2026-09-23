use chrono::{DateTime, Utc};
use ipnet::IpNet;
use uuid::Uuid;

use crate::features::live_chat::domain::ban::LiveChatBan;

#[derive(Debug, Clone)]
pub struct CachedLiveChatBan {
    pub live_chat_ban_id: Uuid,
    pub user_id: Option<Uuid>,
    /// Banned network in canonical form: a /32 or /128 host, or an IPv6 /64.
    pub banned_ip: Option<IpNet>,
    pub reason: String,
    pub ban_source: String,
    pub banned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl CachedLiveChatBan {
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        match self.expires_at {
            Some(expires_at) => expires_at > now,
            None => true,
        }
    }
}

impl From<LiveChatBan> for CachedLiveChatBan {
    fn from(ban: LiveChatBan) -> Self {
        Self {
            live_chat_ban_id: ban.live_chat_ban_id,
            user_id: ban.user_id,
            // Truncation matches lookup keys even if a row stored host bits
            // under a shorter prefix.
            banned_ip: ban.banned_ip.map(|network| network.trunc()),
            reason: ban.reason,
            ban_source: ban.ban_source,
            banned_at: ban.banned_at,
            expires_at: ban.expires_at,
        }
    }
}
