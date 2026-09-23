//! Per-connection writer: the only task that touches the socket sink.

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use futures_util::{SinkExt, stream::SplitSink};
use tokio::{sync::broadcast, task::JoinHandle, time::MissedTickBehavior};
use tracing::{info, warn};
use uuid::Uuid;

use crate::features::live_chat::{
    domain::actor::ChatActor,
    service::{
        cache::{LiveChatBroadcast, LiveChatServerEvent},
        live_chat_service::LiveChatService,
    },
};

use super::{
    LIVE_CHAT_CLOSE_TIMEOUT, LIVE_CHAT_INITIAL_MESSAGES, LIVE_CHAT_PING_INTERVAL,
    LIVE_CHAT_SEND_TIMEOUT, LiveChatWireProtocol,
    protocol::{encode_broadcast, encode_event},
};

pub(super) type SocketSink = SplitSink<WebSocket, Message>;

/// Write one frame, bounded by [`LIVE_CHAT_SEND_TIMEOUT`]. A peer that stops
/// reading fills its TCP window and would otherwise park this task forever,
/// holding the connection slot and every queue that feeds it.
pub(super) async fn send_with_timeout(
    sink: &mut SocketSink,
    message: Message,
    connection_id: Uuid,
) -> bool {
    match tokio::time::timeout(LIVE_CHAT_SEND_TIMEOUT, sink.send(message)).await {
        Ok(Ok(())) => true,
        Ok(Err(_)) => false,
        Err(_) => {
            info!(%connection_id, timeout = ?LIVE_CHAT_SEND_TIMEOUT, "Live chat send timed out; closing slow consumer");
            false
        }
    }
}

pub(super) struct WriterInputs {
    pub(super) sink: SocketSink,
    pub(super) outbound: tokio::sync::mpsc::Receiver<Message>,
    pub(super) broadcasts: broadcast::Receiver<Arc<LiveChatBroadcast>>,
    pub(super) service: Arc<LiveChatService>,
    pub(super) actor: ChatActor,
    pub(super) wire_protocol: LiveChatWireProtocol,
    pub(super) connection_id: Uuid,
}

/// Drain this connection's outbound queue and the room broadcast channel, and
/// send keepalive pings. Exiting drops the outbound receiver, which the reader
/// observes and treats as the end of the connection.
pub(super) fn spawn_writer(inputs: WriterInputs) -> JoinHandle<()> {
    tokio::spawn(async move {
        let WriterInputs {
            mut sink,
            mut outbound,
            mut broadcasts,
            service,
            actor,
            wire_protocol,
            connection_id,
        } = inputs;
        let mut ping = tokio::time::interval_at(
            tokio::time::Instant::now() + LIVE_CHAT_PING_INTERVAL,
            LIVE_CHAT_PING_INTERVAL,
        );
        ping.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            let message = tokio::select! {
                queued = outbound.recv() => match queued {
                    Some(message) => message,
                    // Reader finished and dropped the sender.
                    None => break,
                },
                received = broadcasts.recv() => match received {
                    Ok(broadcast) => {
                        match encode_broadcast(&broadcast, &service.cache, wire_protocol) {
                            Some(message) => message,
                            None => continue,
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!(skipped, %connection_id, "Live chat broadcast receiver lagged; resyncing client");
                        match resync_frame(&service, &actor, wire_protocol).await {
                            Some(message) => message,
                            None => continue,
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!(%connection_id, "Live chat broadcast channel closed");
                        break;
                    }
                },
                _ = ping.tick() => Message::Ping(Bytes::new()),
            };
            if !send_with_timeout(&mut sink, message, connection_id).await {
                break;
            }
        }
        let _ = tokio::time::timeout(LIVE_CHAT_CLOSE_TIMEOUT, sink.close()).await;
    })
}

/// A lagged receiver missed broadcasts; a fresh Hello replaces the client's view.
async fn resync_frame(
    service: &LiveChatService,
    actor: &ChatActor,
    wire_protocol: LiveChatWireProtocol,
) -> Option<Message> {
    let recent_messages = service
        .cache
        .get_recent_chat_messages(LIVE_CHAT_INITIAL_MESSAGES)
        .await;
    let mut resync = LiveChatServerEvent::Hello {
        actor: actor.clone(),
        recent_messages,
        connected_count: service.cache.connected_count(),
    };
    service.cache.anonymize_event_for_public(&mut resync);
    encode_event(&resync, wire_protocol)
}
