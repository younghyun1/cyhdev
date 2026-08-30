use chrono::{DateTime, Utc};
use diesel::Insertable;
use ipnet::IpNet;
use uuid::Uuid;

use crate::schema::live_chat_messages;

#[derive(Insertable)]
#[diesel(table_name = live_chat_messages)]
pub struct LiveChatMessageInsertable {
    pub live_chat_message_id: Uuid,
    pub room_key: String,
    pub user_id: Option<Uuid>,
    pub guest_ip: Option<IpNet>,
    pub sender_kind: i16,
    pub sender_display_name: String,
    pub message_body: String,
    pub message_created_at: DateTime<Utc>,
}
