use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde_derive::Serialize;
use uuid::Uuid;

use crate::features::live_chat::domain::{
    guest_nickname::normalize_guest_display_name, message::LiveChatMessage,
};
use crate::features::accounts::domain::account::DELETED_USER_DISPLAY_NAME;

use super::LIVE_CHAT_MESSAGE_FIXED_BYTES;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ChatTimelineKey {
    pub room_key: String,
    pub message_created_at_micros: i64,
    pub live_chat_message_id: Uuid,
}

impl ChatTimelineKey {
    pub fn from_message(message: &CachedChatMessage) -> Self {
        Self {
            room_key: message.room_key.clone(),
            message_created_at_micros: message.message_created_at.timestamp_micros(),
            live_chat_message_id: message.live_chat_message_id,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ChatEvictionKey {
    pub(super) live_chat_message_id: Uuid,
    pub(super) timeline_key: ChatTimelineKey,
    pub(super) estimated_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CachedChatMessage {
    pub live_chat_message_id: Uuid,
    pub room_key: String,
    pub user_id: Option<Uuid>,
    pub guest_ip: Option<IpAddr>,
    pub sender_kind: i16,
    pub sender_display_name: String,
    pub sender_country_flag: Option<String>,
    pub user_profile_picture_url: Option<String>,
    pub message_body: String,
    pub message_created_at: DateTime<Utc>,
    pub message_edited_at: Option<DateTime<Utc>>,
    pub message_deleted_at: Option<DateTime<Utc>>,
}

impl CachedChatMessage {
    pub fn estimated_bytes(&self) -> usize {
        LIVE_CHAT_MESSAGE_FIXED_BYTES
            + self.room_key.len()
            + self.sender_display_name.len()
            + match &self.sender_country_flag {
                Some(country_flag) => country_flag.len(),
                None => 0,
            }
            + match &self.user_profile_picture_url {
                Some(user_profile_picture_url) => user_profile_picture_url.len(),
                None => 0,
            }
            + self.message_body.len()
    }

    /// Remove the deleted account's public identity from a retained message.
    pub fn anonymize_deleted_user(&mut self, deleted_user_id: Uuid) -> bool {
        if self.user_id != Some(deleted_user_id) {
            return false;
        }
        self.user_id = None;
        self.guest_ip = None;
        self.sender_display_name = DELETED_USER_DISPLAY_NAME.to_owned();
        self.sender_country_flag = None;
        self.user_profile_picture_url = None;
        true
    }
}

impl From<LiveChatMessage> for CachedChatMessage {
    fn from(message: LiveChatMessage) -> Self {
        let guest_ip = message.guest_ip.map(|ip| ip.addr());
        let sender_display_name = normalize_guest_display_name(
            message.sender_display_name,
            message.sender_kind,
            guest_ip,
        );

        Self {
            live_chat_message_id: message.live_chat_message_id,
            room_key: message.room_key,
            user_id: message.user_id,
            guest_ip,
            sender_kind: message.sender_kind,
            sender_display_name,
            sender_country_flag: None,
            user_profile_picture_url: None,
            message_body: message.message_body,
            message_created_at: message.message_created_at,
            message_edited_at: message.message_edited_at,
            message_deleted_at: message.message_deleted_at,
        }
    }
}
