use diesel::{Insertable, Queryable, Selectable};
use ipnet::IpNet;
use jiff::Timestamp;
use uuid::Uuid;

use crate::schema::{
    live_chat_bans, live_chat_call_participants, live_chat_calls, live_chat_messages,
};

use super::super::domain::{ban::LiveChatBan, message::LiveChatMessage};

#[derive(Queryable, Selectable)]
#[diesel(table_name = live_chat_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct MessageRecord {
    live_chat_message_id: Uuid,
    room_key: String,
    user_id: Option<Uuid>,
    guest_ip: Option<IpNet>,
    sender_kind: i16,
    sender_display_name: String,
    message_body: String,
    #[diesel(deserialize_as = jiff_diesel::Timestamp)]
    message_created_at: Timestamp,
    #[diesel(deserialize_as = jiff_diesel::NullableTimestamp)]
    message_edited_at: Option<Timestamp>,
    #[diesel(deserialize_as = jiff_diesel::NullableTimestamp)]
    message_deleted_at: Option<Timestamp>,
}

impl From<MessageRecord> for LiveChatMessage {
    fn from(row: MessageRecord) -> Self {
        Self {
            live_chat_message_id: row.live_chat_message_id,
            room_key: row.room_key,
            user_id: row.user_id,
            guest_ip: row.guest_ip,
            sender_kind: row.sender_kind,
            sender_display_name: row.sender_display_name,
            message_body: row.message_body,
            message_created_at: row.message_created_at,
            message_edited_at: row.message_edited_at,
            message_deleted_at: row.message_deleted_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = live_chat_messages)]
pub(super) struct NewMessageRecord {
    pub live_chat_message_id: Uuid,
    pub room_key: String,
    pub user_id: Option<Uuid>,
    pub guest_ip: Option<IpNet>,
    pub sender_kind: i16,
    pub sender_display_name: String,
    pub message_body: String,
    #[diesel(serialize_as = jiff_diesel::Timestamp)]
    pub message_created_at: Timestamp,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = live_chat_bans)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct BanRecord {
    live_chat_ban_id: Uuid,
    user_id: Option<Uuid>,
    banned_ip: Option<IpNet>,
    reason: String,
    ban_source: String,
    #[diesel(deserialize_as = jiff_diesel::Timestamp)]
    banned_at: Timestamp,
    #[diesel(deserialize_as = jiff_diesel::NullableTimestamp)]
    expires_at: Option<Timestamp>,
}

impl From<BanRecord> for LiveChatBan {
    fn from(row: BanRecord) -> Self {
        Self {
            live_chat_ban_id: row.live_chat_ban_id,
            user_id: row.user_id,
            banned_ip: row.banned_ip,
            reason: row.reason,
            ban_source: row.ban_source,
            banned_at: row.banned_at,
            expires_at: row.expires_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = live_chat_bans)]
pub(super) struct NewBanRecord {
    pub live_chat_ban_id: Uuid,
    pub user_id: Option<Uuid>,
    pub banned_ip: Option<IpNet>,
    pub reason: String,
    pub ban_source: String,
    #[diesel(serialize_as = jiff_diesel::Timestamp)]
    pub banned_at: Timestamp,
    #[diesel(serialize_as = jiff_diesel::NullableTimestamp)]
    pub expires_at: Option<Timestamp>,
}

#[derive(Insertable)]
#[diesel(table_name = live_chat_calls)]
pub(super) struct NewCallRecord {
    pub live_chat_call_id: Uuid,
    pub room_key: String,
    #[diesel(serialize_as = jiff_diesel::Timestamp)]
    pub call_started_at: Timestamp,
}

#[derive(Insertable)]
#[diesel(table_name = live_chat_call_participants)]
pub(super) struct NewParticipantRecord {
    pub live_chat_call_participant_id: Uuid,
    pub live_chat_call_id: Uuid,
    pub user_id: Option<Uuid>,
    pub guest_ip: Option<IpNet>,
    pub participant_sender_kind: i16,
    pub participant_display_name: String,
    #[diesel(serialize_as = jiff_diesel::Timestamp)]
    pub participant_joined_at: Timestamp,
    pub participant_had_audio: bool,
    pub participant_had_video: bool,
}
