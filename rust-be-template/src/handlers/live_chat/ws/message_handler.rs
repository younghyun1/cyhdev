use std::{net::IpAddr, sync::Arc};

use axum::extract::ws::Message;
use chrono::{Duration as ChronoDuration, Utc};

use super::{
    LIVE_CHAT_MAX_MESSAGE_CHARS, LIVE_CHAT_TYPING_TTL_SECONDS, LiveChatWireProtocol,
    persistence::{persist_live_chat_ban, persist_message},
    presence::{broadcast_typing_set, handle_typing},
    protocol::{
        DecodedLiveChatClientEvent, OutboundSender, decode_binary_client_event,
        decode_json_client_event, send_event,
    },
    rtc::RtcSession,
};
use crate::{
    domain::live_chat::cache::{ChatActor, LiveChatServerEvent},
    init::state::ServerState,
};

pub(super) async fn handle_client_message(
    out: &OutboundSender,
    state: Arc<ServerState>,
    actor: ChatActor,
    client_ip: IpAddr,
    message: Message,
    wire_protocol: LiveChatWireProtocol,
    rtc_session: &Arc<RtcSession>,
) -> bool {
    if matches!(message, Message::Close(_)) {
        return false;
    }

    let client_event = match wire_protocol {
        LiveChatWireProtocol::Json => match decode_json_client_event(out, &message).await {
            Some(event) => event,
            None => return true,
        },
        LiveChatWireProtocol::Binary => match decode_binary_client_event(out, message).await {
            Some(event) => event,
            None => return true,
        },
    };

    match client_event {
        DecodedLiveChatClientEvent::SendMessage {
            client_message_id,
            body,
        } => {
            return handle_send_message(
                out,
                state,
                actor,
                client_ip,
                client_message_id,
                body,
                wire_protocol,
            )
            .await;
        }
        DecodedLiveChatClientEvent::Typing { is_typing } => {
            handle_typing(state, actor, is_typing).await;
        }
        DecodedLiveChatClientEvent::Heartbeat { nonce } => {
            send_event(
                out,
                &LiveChatServerEvent::HeartbeatAck { nonce },
                wire_protocol,
            )
            .await;
        }
        DecodedLiveChatClientEvent::Rtc(signal) => rtc_session.dispatch(signal).await,
    }

    true
}

async fn handle_send_message(
    out: &OutboundSender,
    state: Arc<ServerState>,
    actor: ChatActor,
    client_ip: IpAddr,
    client_message_id: String,
    body: String,
    wire_protocol: LiveChatWireProtocol,
) -> bool {
    let now = Utc::now();
    if state.is_live_chat_actor_banned(actor.user_id, client_ip).await {
        send_error(out, "banned", "Live chat access denied.", wire_protocol).await;
        return false;
    }

    if state
        .live_chat_cache
        .record_message_attempt(actor.user_id, client_ip, now)
        .await
    {
        if let Some(ban) = persist_live_chat_ban(state.clone(), &actor, client_ip).await {
            let _ = state.live_chat_cache.cache_ban(ban).await;
        }
        send_error(
            out,
            "banned",
            "Live chat access denied for abnormal messaging patterns.",
            wire_protocol,
        )
        .await;
        return false;
    }

    let body = body.trim().to_string();
    if body.is_empty() {
        send_error(out, "empty_message", "Message cannot be empty.", wire_protocol).await;
        return true;
    }
    if body.chars().count() > LIVE_CHAT_MAX_MESSAGE_CHARS {
        let message = format!("Message must be {LIVE_CHAT_MAX_MESSAGE_CHARS} characters or fewer.");
        send_error(out, "message_too_large", &message, wire_protocol).await;
        return true;
    }

    let persisted = match persist_message(state.clone(), &actor, body).await {
        Some(message) => message,
        None => {
            send_error(out, "persist_failed", "Message could not be saved.", wire_protocol).await;
            return true;
        }
    };
    state
        .live_chat_cache
        .append_persisted_chat_message(persisted.clone())
        .await;
    if state.live_chat_cache.clear_typing(&actor.actor_key).await {
        let expires_at = Utc::now() + ChronoDuration::seconds(LIVE_CHAT_TYPING_TTL_SECONDS);
        broadcast_typing_set(state.clone(), expires_at).await;
    }

    send_event(
        out,
        &LiveChatServerEvent::MessageAck {
            client_message_id,
            message: persisted.clone(),
        },
        wire_protocol,
    )
    .await;
    state
        .live_chat_cache
        .broadcast(LiveChatServerEvent::Message { message: persisted });
    true
}

async fn send_error(
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
