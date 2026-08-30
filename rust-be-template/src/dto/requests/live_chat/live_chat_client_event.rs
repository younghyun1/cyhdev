use serde_derive::Deserialize;

use crate::features::live_chat::domain::rtc::RtcClientSignal;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LiveChatClientEvent {
    SendMessage {
        client_message_id: String,
        body: String,
    },
    Typing {
        is_typing: bool,
    },
    Heartbeat {
        nonce: String,
    },
    /// WebRTC signaling from the client. Inner enum is tagged with `kind`.
    Rtc(RtcClientSignal),
}
