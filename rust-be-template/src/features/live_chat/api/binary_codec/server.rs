use crate::features::live_chat::service::cache::LiveChatServerEvent;

use super::{
    SERVER_ERROR, SERVER_HELLO, SERVER_MESSAGE, SERVER_MESSAGE_ACK, SERVER_PONG,
    SERVER_PRESENCE, SERVER_RTC, SERVER_TYPING_SET,
    rtc, saturating::{saturating_u8, saturating_u16, saturating_u32},
    writer::BinaryWriter,
};

pub fn encode_server_event(event: &LiveChatServerEvent) -> anyhow::Result<Vec<u8>> {
    let mut writer = BinaryWriter::default();
    match event {
        LiveChatServerEvent::Hello { actor, recent_messages, connected_count } => {
            writer.write_u8(SERVER_HELLO);
            writer.write_actor(actor)?;
            writer.write_u32(saturating_u32(*connected_count));
            writer.write_u16(saturating_u16(recent_messages.len()));
            for message in recent_messages.iter().take(u16::MAX as usize) {
                writer.write_message(message)?;
            }
        }
        LiveChatServerEvent::Message { message } => {
            writer.write_u8(SERVER_MESSAGE); writer.write_message(message)?;
        }
        LiveChatServerEvent::MessageAck { client_message_id, message } => {
            writer.write_u8(SERVER_MESSAGE_ACK);
            writer.write_uuid_string(client_message_id)?; writer.write_message(message)?;
        }
        LiveChatServerEvent::TypingSet { actors, expires_at } => {
            writer.write_u8(SERVER_TYPING_SET); writer.write_time(*expires_at);
            writer.write_u8(saturating_u8(actors.len()));
            for actor in actors.iter().take(u8::MAX as usize) { writer.write_actor(actor)?; }
        }
        LiveChatServerEvent::Typing { actor, is_typing, expires_at } => {
            writer.write_u8(SERVER_TYPING_SET); writer.write_time(*expires_at);
            writer.write_u8(u8::from(*is_typing));
            if *is_typing { writer.write_actor(actor)?; }
        }
        LiveChatServerEvent::Presence { connected_count } => {
            writer.write_u8(SERVER_PRESENCE); writer.write_u32(saturating_u32(*connected_count));
        }
        LiveChatServerEvent::HeartbeatAck { nonce } => {
            writer.write_u8(SERVER_PONG);
            writer.write_u64(nonce.parse::<u64>().unwrap_or(0));
        }
        LiveChatServerEvent::Error { code, message } => {
            writer.write_u8(SERVER_ERROR); writer.write_string(code)?; writer.write_string(message)?;
        }
        LiveChatServerEvent::Rtc(signal) => {
            writer.write_u8(SERVER_RTC); rtc::encode_rtc_server_signal(&mut writer, signal)?;
        }
    }
    Ok(writer.into_inner())
}
