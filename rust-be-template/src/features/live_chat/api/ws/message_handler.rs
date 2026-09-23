use std::sync::Arc;

use axum::extract::ws::Message;
use chrono::Utc;
use tokio::time::Instant;
use tracing::info;
use uuid::Uuid;

use super::{
    ConnectionContext, LIVE_CHAT_MAX_MESSAGE_CHARS,
    event_budget::{BudgetDecision, ClientEventBudget, PING_FRAME_COST, event_cost},
    persistence::{persist_live_chat_ban, persist_message},
    protocol::{DecodedLiveChatClientEvent, decode_client_frame, send_error, send_event},
};
use crate::features::live_chat::service::cache::{LiveChatServerEvent, MessageRateDecision};

/// Malformed data frames tolerated per connection. Browsers never send one,
/// so a few cover a protocol mismatch while a junk stream is cut off quickly.
const MAX_MALFORMED_FRAMES: u8 = 4;

/// What the reader loop should do after one inbound frame.
pub(super) enum FrameOutcome {
    Continue,
    Close,
}

/// Per-connection admission state consulted before any frame is dispatched.
pub(super) struct FrameGuard {
    budget: ClientEventBudget,
    malformed_frames: u8,
}

enum Admission {
    Admitted,
    Dropped,
    Close,
}

impl FrameGuard {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            budget: ClientEventBudget::new(now),
            malformed_frames: 0,
        }
    }

    async fn admit(&mut self, context: &ConnectionContext, cost: u32) -> Admission {
        match self.budget.spend(cost, Instant::now()) {
            BudgetDecision::Admit => Admission::Admitted,
            BudgetDecision::Reject { notify } => {
                if notify {
                    send_error(
                        &context.out,
                        "rate_limited",
                        "Too many live chat events. Please slow down.",
                        context.wire_protocol,
                    )
                    .await;
                }
                Admission::Dropped
            }
            BudgetDecision::Close => {
                info!(connection_id = %context.connection_id, "Closing live chat connection that kept exceeding its event budget");
                Admission::Close
            }
        }
    }

    fn record_malformed(&mut self, oversized: bool, connection_id: Uuid) -> FrameOutcome {
        self.malformed_frames = self.malformed_frames.saturating_add(1);
        if oversized || self.malformed_frames >= MAX_MALFORMED_FRAMES {
            info!(%connection_id, oversized, malformed_frames = self.malformed_frames, "Closing live chat connection after malformed frames");
            return FrameOutcome::Close;
        }
        FrameOutcome::Continue
    }
}

pub(super) async fn handle_client_frame(
    context: &ConnectionContext,
    guard: &mut FrameGuard,
    message: Message,
) -> FrameOutcome {
    match &message {
        Message::Close(_) => return FrameOutcome::Close,
        // Liveness only: the reader already moved the idle deadline.
        Message::Pong(_) => return FrameOutcome::Continue,
        // tungstenite queues the Pong reply itself; only meter the request.
        Message::Ping(_) => {
            return match guard.admit(context, PING_FRAME_COST).await {
                Admission::Close => FrameOutcome::Close,
                Admission::Admitted | Admission::Dropped => FrameOutcome::Continue,
            };
        }
        Message::Text(_) | Message::Binary(_) => {}
    }
    if let Some(user_id) = context.actor.user_id
        && context.service.cache.is_connected_user_disabled(user_id)
    {
        send_error(
            &context.out,
            "account_deleted",
            "This account can no longer use live chat.",
            context.wire_protocol,
        )
        .await;
        return FrameOutcome::Close;
    }

    let event = match decode_client_frame(message, context.wire_protocol) {
        Ok(event) => event,
        Err(rejection) => {
            send_error(
                &context.out,
                rejection.code,
                rejection.message,
                context.wire_protocol,
            )
            .await;
            return guard.record_malformed(rejection.oversized, context.connection_id);
        }
    };
    match guard.admit(context, event_cost(&event)).await {
        Admission::Admitted => {}
        Admission::Dropped => return FrameOutcome::Continue,
        Admission::Close => return FrameOutcome::Close,
    }

    match event {
        DecodedLiveChatClientEvent::SendMessage {
            client_message_id,
            body,
        } => return handle_send_message(context, client_message_id, body).await,
        DecodedLiveChatClientEvent::Typing { is_typing } => {
            context
                .service
                .record_typing(&context.actor, is_typing)
                .await;
        }
        DecodedLiveChatClientEvent::Heartbeat { nonce } => {
            send_event(
                &context.out,
                &LiveChatServerEvent::HeartbeatAck { nonce },
                context.wire_protocol,
            )
            .await;
        }
        DecodedLiveChatClientEvent::Rtc(signal) => context.rtc.dispatch(signal).await,
    }
    FrameOutcome::Continue
}

async fn handle_send_message(
    context: &ConnectionContext,
    client_message_id: String,
    body: String,
) -> FrameOutcome {
    let ConnectionContext {
        out,
        service,
        actor,
        client_ip,
        wire_protocol,
        ..
    } = context;
    let (client_ip, wire_protocol) = (*client_ip, *wire_protocol);
    if service.is_actor_banned(actor.user_id, client_ip).await {
        send_error(out, "banned", "Live chat access denied.", wire_protocol).await;
        return FrameOutcome::Close;
    }

    match service
        .cache
        .record_message_attempt(actor.user_id, client_ip, Utc::now())
        .await
    {
        MessageRateDecision::Allowed => {}
        MessageRateDecision::Abnormal => {
            if let Some(ban) = persist_live_chat_ban(Arc::clone(service), actor, client_ip).await {
                let _ = service.cache.cache_ban(ban).await;
            }
            send_error(
                out,
                "banned",
                "Live chat access denied for abnormal messaging patterns.",
                wire_protocol,
            )
            .await;
            return FrameOutcome::Close;
        }
        MessageRateDecision::Saturated => {
            send_error(
                out,
                "rate_limited",
                "Live chat is busy. Please try again shortly.",
                wire_protocol,
            )
            .await;
            return FrameOutcome::Continue;
        }
    }

    let body = body.trim().to_string();
    if body.is_empty() {
        send_error(
            out,
            "empty_message",
            "Message cannot be empty.",
            wire_protocol,
        )
        .await;
        return FrameOutcome::Continue;
    }
    if body.chars().count() > LIVE_CHAT_MAX_MESSAGE_CHARS {
        let message = format!("Message must be {LIVE_CHAT_MAX_MESSAGE_CHARS} characters or fewer.");
        send_error(out, "message_too_large", &message, wire_protocol).await;
        return FrameOutcome::Continue;
    }

    let persisted = match persist_message(Arc::clone(service), actor, body).await {
        Some(message) => message,
        None => {
            send_error(
                out,
                "persist_failed",
                "Message could not be saved.",
                wire_protocol,
            )
            .await;
            return FrameOutcome::Continue;
        }
    };
    service
        .cache
        .append_persisted_chat_message(persisted.clone())
        .await;
    service.clear_actor_typing(&actor.actor_key).await;

    send_event(
        out,
        &LiveChatServerEvent::MessageAck {
            client_message_id,
            message: persisted.clone(),
        },
        wire_protocol,
    )
    .await;
    service
        .cache
        .broadcast(LiveChatServerEvent::Message { message: persisted });
    FrameOutcome::Continue
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_malformed_frames_and_any_oversized_frame_close() {
        let mut guard = FrameGuard::new(Instant::now());
        let id = Uuid::now_v7();
        for _ in 1..MAX_MALFORMED_FRAMES {
            assert!(matches!(
                guard.record_malformed(false, id),
                FrameOutcome::Continue
            ));
        }
        assert!(matches!(
            guard.record_malformed(false, id),
            FrameOutcome::Close
        ));
        let mut fresh = FrameGuard::new(Instant::now());
        assert!(matches!(
            fresh.record_malformed(true, id),
            FrameOutcome::Close
        ));
    }
}
