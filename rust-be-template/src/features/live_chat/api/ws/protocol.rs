use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::ws::{Message, Utf8Bytes},
};
use tracing::{debug, error, warn};

use crate::{
    dto::requests::live_chat::live_chat_client_event::LiveChatClientEvent,
    features::live_chat::{
        api::binary_codec::{LiveChatBinaryClientEvent, decode_client_event, encode_server_event},
        domain::rtc::RtcClientSignal,
        service::cache::{LiveChatBroadcast, LiveChatCache, LiveChatServerEvent},
    },
};

use super::{LIVE_CHAT_MAX_FRAME_BYTES, LiveChatWireProtocol, log_throttle::LogThrottle};

/// Outbound frame channel to the per-connection writer task. Handlers enqueue
/// already-or-soon-encoded frames here instead of writing to the socket directly,
/// so a slow DB persist on the read side never stalls the broadcast drain.
pub(super) type OutboundSender = tokio::sync::mpsc::Sender<Message>;

/// One warning per interval summarizes malformed frames across all
/// connections; individual frames log at debug. A per-frame warning let any
/// client turn junk frames into log volume.
static MALFORMED_FRAME_LOG: LogThrottle = LogThrottle::new();
const MALFORMED_FRAME_LOG_INTERVAL_MILLIS: u64 = 10_000;

pub(super) enum DecodedLiveChatClientEvent {
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

impl From<LiveChatClientEvent> for DecodedLiveChatClientEvent {
    fn from(event: LiveChatClientEvent) -> Self {
        match event {
            LiveChatClientEvent::SendMessage {
                client_message_id,
                body,
            } => Self::SendMessage {
                client_message_id,
                body,
            },
            LiveChatClientEvent::Typing { is_typing } => Self::Typing { is_typing },
            LiveChatClientEvent::Heartbeat { nonce } => Self::Heartbeat { nonce },
            LiveChatClientEvent::Rtc(signal) => Self::Rtc(signal),
        }
    }
}

impl From<LiveChatBinaryClientEvent> for DecodedLiveChatClientEvent {
    fn from(event: LiveChatBinaryClientEvent) -> Self {
        match event {
            LiveChatBinaryClientEvent::SendMessage {
                client_message_id,
                body,
            } => Self::SendMessage {
                client_message_id,
                body,
            },
            LiveChatBinaryClientEvent::Typing { is_typing } => Self::Typing { is_typing },
            LiveChatBinaryClientEvent::Heartbeat { nonce } => Self::Heartbeat { nonce },
            LiveChatBinaryClientEvent::Rtc(signal) => Self::Rtc(signal),
        }
    }
}

/// A data frame the protocol cannot accept, with the error sent to the client.
pub(super) struct FrameRejection {
    pub(super) code: &'static str,
    pub(super) message: &'static str,
    /// Oversized frames close the connection immediately; tungstenite already
    /// enforces the limit, so reaching this check means a misconfiguration.
    pub(super) oversized: bool,
}

impl FrameRejection {
    const fn malformed(code: &'static str, message: &'static str) -> Self {
        Self {
            code,
            message,
            oversized: false,
        }
    }
}

/// Decode one data frame. Control frames are handled by the caller.
pub(super) fn decode_client_frame(
    message: Message,
    wire_protocol: LiveChatWireProtocol,
) -> Result<DecodedLiveChatClientEvent, FrameRejection> {
    let payload_len = match &message {
        Message::Text(text) => text.len(),
        Message::Binary(bytes) => bytes.len(),
        Message::Ping(_) | Message::Pong(_) | Message::Close(_) => 0,
    };
    if payload_len > LIVE_CHAT_MAX_FRAME_BYTES {
        return Err(FrameRejection {
            code: "frame_too_large",
            message: "Live chat event payload is too large.",
            oversized: true,
        });
    }
    match (wire_protocol, message) {
        (LiveChatWireProtocol::Json, Message::Text(text)) => {
            match serde_json::from_str::<LiveChatClientEvent>(text.as_str()) {
                Ok(event) => Ok(event.into()),
                Err(error_value) => {
                    log_malformed_frame("invalid_json", &error_value);
                    Err(FrameRejection::malformed(
                        "invalid_json",
                        "Invalid live chat event payload",
                    ))
                }
            }
        }
        (LiveChatWireProtocol::Binary, Message::Binary(bytes)) => {
            match decode_client_event(&bytes) {
                Ok(event) => Ok(event.into()),
                Err(error_value) => {
                    log_malformed_frame("invalid_binary", &error_value);
                    Err(FrameRejection::malformed(
                        "invalid_binary",
                        "Invalid live chat binary event payload",
                    ))
                }
            }
        }
        (LiveChatWireProtocol::Json, _) => {
            log_malformed_frame("invalid_frame", &"non-text frame in JSON mode");
            Err(FrameRejection::malformed(
                "invalid_frame",
                "Expected UTF-8 text frame",
            ))
        }
        (LiveChatWireProtocol::Binary, _) => {
            log_malformed_frame("invalid_frame", &"non-binary frame in binary mode");
            Err(FrameRejection::malformed(
                "invalid_frame",
                "Expected binary live chat frame",
            ))
        }
    }
}

fn log_malformed_frame(kind: &'static str, error_value: &dyn std::fmt::Display) {
    let now_millis = u64::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or(0);
    match MALFORMED_FRAME_LOG.admit(now_millis, MALFORMED_FRAME_LOG_INTERVAL_MILLIS) {
        Some(suppressed) => {
            warn!(kind, error = %error_value, suppressed, "Rejected malformed live chat frame")
        }
        None => debug!(kind, error = %error_value, "Rejected malformed live chat frame"),
    }
}

/// Serialize a server event into a WebSocket frame for the negotiated wire
/// protocol. Returns `None` (and logs) on a serialization failure.
pub(super) fn encode_event(
    event: &LiveChatServerEvent,
    wire_protocol: LiveChatWireProtocol,
) -> Option<Message> {
    match wire_protocol {
        LiveChatWireProtocol::Binary => {
            encode_binary(event).map(|payload| Message::Binary(Bytes::from(payload)))
        }
        LiveChatWireProtocol::Json => match serde_json::to_string(event) {
            Ok(payload) => Some(Message::Text(payload.into())),
            Err(error_value) => {
                error!(error = ?error_value, "Failed to serialize live chat server event");
                None
            }
        },
    }
}

/// Frame for a room broadcast, encoded once per protocol and shared across
/// every recipient. Anonymization runs inside that single encode.
pub(super) fn encode_broadcast(
    broadcast: &LiveChatBroadcast,
    cache: &LiveChatCache,
    wire_protocol: LiveChatWireProtocol,
) -> Option<Message> {
    let public_event = |event: &LiveChatServerEvent| {
        let mut event = event.clone();
        cache.anonymize_event_for_public(&mut event);
        event
    };
    match wire_protocol {
        LiveChatWireProtocol::Binary => broadcast
            .binary_frame(|event| encode_binary(&public_event(event)))
            .map(|frame| Message::Binary(Bytes::from_owner(SharedFrame(frame)))),
        LiveChatWireProtocol::Json => {
            let frame =
                broadcast.json_frame(|event| match serde_json::to_vec(&public_event(event)) {
                    Ok(payload) => Some(payload),
                    Err(error_value) => {
                        error!(error = ?error_value, "Failed to serialize live chat broadcast");
                        None
                    }
                })?;
            // serde_json output is UTF-8; the check validates without copying.
            match Utf8Bytes::try_from(Bytes::from_owner(SharedFrame(frame))) {
                Ok(text) => Some(Message::Text(text)),
                Err(error_value) => {
                    error!(error = %error_value, "Live chat broadcast JSON was not UTF-8");
                    None
                }
            }
        }
    }
}

fn encode_binary(event: &LiveChatServerEvent) -> Option<Vec<u8>> {
    match encode_server_event(event) {
        Ok(payload) => Some(payload),
        Err(error_value) => {
            error!(error = ?error_value, "Failed to serialize binary live chat server event");
            None
        }
    }
}

/// Reference-counted frame bytes handed to the socket without copying.
struct SharedFrame(Arc<[u8]>);

impl AsRef<[u8]> for SharedFrame {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Enqueue a server event for the writer task. A closed channel means the
/// connection is already tearing down, so the frame is dropped silently.
pub(super) async fn send_event(
    out: &OutboundSender,
    event: &LiveChatServerEvent,
    wire_protocol: LiveChatWireProtocol,
) {
    if let Some(message) = encode_event(event, wire_protocol) {
        let _ = out.send(message).await;
    }
}

pub(super) async fn send_error(
    out: &OutboundSender,
    code: &str,
    message: &str,
    wire_protocol: LiveChatWireProtocol,
) {
    send_event(
        out,
        &LiveChatServerEvent::Error {
            code: code.to_string(),
            message: message.to_string(),
        },
        wire_protocol,
    )
    .await;
}
