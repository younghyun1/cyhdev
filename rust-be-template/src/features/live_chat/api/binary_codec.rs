use crate::features::live_chat::domain::rtc::RtcClientSignal;
mod reader;
mod rtc;
mod saturating;
mod server;
mod writer;

use reader::BinaryReader;
pub use server::encode_server_event;

/// Version 2 replaced guest IP fields with opaque guest keys; a client still
/// offering v1 is not selected and falls back to the JSON protocol.
pub const LIVE_CHAT_BINARY_PROTOCOL: &str = "livechat.bin.v2";

const CLIENT_SEND_MESSAGE: u8 = 0x01;
const CLIENT_TYPING_START: u8 = 0x02;
const CLIENT_TYPING_STOP: u8 = 0x03;
const CLIENT_PING: u8 = 0x04;
const CLIENT_RTC: u8 = 0x05;

const SERVER_HELLO: u8 = 0x81;
const SERVER_MESSAGE: u8 = 0x82;
const SERVER_MESSAGE_ACK: u8 = 0x83;
const SERVER_TYPING_SET: u8 = 0x84;
const SERVER_PRESENCE: u8 = 0x85;
const SERVER_PONG: u8 = 0x86;
const SERVER_ERROR: u8 = 0x87;
const SERVER_MESSAGE_DELETED: u8 = 0x88;
const SERVER_RTC: u8 = 0x90;

const ACTOR_USER: u8 = 0x01;
const ACTOR_GUEST: u8 = 0x02;

const MESSAGE_FLAG_EDITED_AT: u8 = 0x01;
const MESSAGE_FLAG_DELETED_AT: u8 = 0x02;
const NONE_STRING_LEN: u16 = u16::MAX;

#[derive(Debug)]
pub enum LiveChatBinaryClientEvent {
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
    Rtc(RtcClientSignal),
}

pub fn decode_client_event(bytes: &[u8]) -> anyhow::Result<LiveChatBinaryClientEvent> {
    let mut reader = BinaryReader::new(bytes);
    let event_type = reader.read_u8()?;
    match event_type {
        CLIENT_RTC => {
            let signal = rtc::decode_rtc_client_signal(&mut reader)?;
            reader.finish()?;
            Ok(LiveChatBinaryClientEvent::Rtc(signal))
        }
        CLIENT_SEND_MESSAGE => {
            let client_message_id = reader.read_uuid()?.to_string();
            let body = reader.read_string()?;
            reader.finish()?;
            Ok(LiveChatBinaryClientEvent::SendMessage {
                client_message_id,
                body,
            })
        }
        CLIENT_TYPING_START => {
            reader.finish()?;
            Ok(LiveChatBinaryClientEvent::Typing { is_typing: true })
        }
        CLIENT_TYPING_STOP => {
            reader.finish()?;
            Ok(LiveChatBinaryClientEvent::Typing { is_typing: false })
        }
        CLIENT_PING => {
            let nonce = reader.read_u64()?.to_string();
            reader.finish()?;
            Ok(LiveChatBinaryClientEvent::Heartbeat { nonce })
        }
        _ => Err(anyhow::anyhow!(
            "Unknown live chat binary client event type"
        )),
    }
}

#[cfg(test)]
mod tests;
