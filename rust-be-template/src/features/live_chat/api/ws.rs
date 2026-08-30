use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tracing::{info, warn};
use uuid::Uuid;

use crate::features::live_chat::{
    domain::actor::ChatActor,
    service::{cache::LiveChatServerEvent, live_chat_service::LiveChatService},
};

mod persistence;
mod presence;
mod protocol;
mod rtc;
mod rtc_teardown;
mod message_handler;
mod actor_resolution;
mod registration;
mod upgrade;

use message_handler::handle_client_message;
use presence::cleanup_live_chat_connection;
use protocol::encode_event;
use rtc::RtcSession;
pub use upgrade::live_chat_ws_handler;

/// Bound on the per-connection outbound frame queue feeding the writer task.
const LIVE_CHAT_OUTBOUND_QUEUE: usize = 128;

const LIVE_CHAT_INITIAL_MESSAGES: usize = 50;
const LIVE_CHAT_MAX_MESSAGE_CHARS: usize = 300;
// Raised from 2 KiB to fit WebRTC SDP offers/answers, which exceed it. Chat
// message bodies remain bounded separately by `LIVE_CHAT_MAX_MESSAGE_CHARS`.
pub(super) const LIVE_CHAT_MAX_FRAME_BYTES: usize = 64 * 1024;
pub(super) const LIVE_CHAT_TYPING_TTL_SECONDS: i64 = 4;

#[derive(Clone, Copy)]
pub(super) enum LiveChatWireProtocol {
    Json,
    Binary,
}

pub(super) async fn handle_live_chat_socket(
    socket: WebSocket,
    service: Arc<LiveChatService>,
    actor: ChatActor,
    client_ip: std::net::IpAddr,
    wire_protocol: LiveChatWireProtocol,
    connection_id: Uuid,
    mut disconnect_rx: tokio::sync::watch::Receiver<bool>,
) {
    let broadcast_rx = service.cache.subscribe();

    let (mut sink, mut stream) = socket.split();

    // Send the initial Hello synchronously on the sink before the writer task
    // takes ownership, so it is guaranteed to be the first frame the client sees.
    let recent_messages = service.cache
        .get_recent_chat_messages(LIVE_CHAT_INITIAL_MESSAGES)
        .await;
    let mut hello = LiveChatServerEvent::Hello {
        actor: actor.clone(),
        recent_messages,
        connected_count: service.cache.connected_count(),
    };
    service.cache.anonymize_event_for_public(&mut hello);
    let hello_sent = match encode_event(&hello, wire_protocol) {
        Some(message) => sink.send(message).await.is_ok(),
        None => false,
    };
    if !hello_sent {
        cleanup_live_chat_connection(service, connection_id, &actor).await;
        return;
    }

    // The writer task owns the sink and drains two sources: this connection's
    // outbound queue (acks/errors) and the room broadcast channel. Keeping the
    // broadcast drain on its own task means a slow DB persist on the read side
    // can no longer stall delivery of other users' messages to this client.
    let (out_tx, mut out_rx) = tokio::sync::mpsc::channel::<Message>(LIVE_CHAT_OUTBOUND_QUEUE);
    let writer_service = Arc::clone(&service);
    let writer_actor = actor.clone();
    let writer = tokio::spawn(async move {
        let mut broadcast_rx = broadcast_rx;
        loop {
            tokio::select! {
                outbound = out_rx.recv() => {
                    match outbound {
                        Some(message) => {
                            if sink.send(message).await.is_err() {
                                break;
                            }
                        }
                        // Reader finished and dropped the sender.
                        None => break,
                    }
                }
                broadcast_event = broadcast_rx.recv() => {
                    match broadcast_event {
                        Ok(mut event) => {
                            writer_service.cache.anonymize_event_for_public(&mut event);
                            if let Some(message) = encode_event(&event, wire_protocol)
                                && sink.send(message).await.is_err()
                            {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!(skipped = skipped, connection_id = %connection_id, "Live chat broadcast receiver lagged; resyncing client");
                            let recent_messages = writer_service.cache
                                .get_recent_chat_messages(LIVE_CHAT_INITIAL_MESSAGES)
                                .await;
                            let mut resync = LiveChatServerEvent::Hello {
                                actor: writer_actor.clone(),
                                recent_messages,
                                connected_count: writer_service.cache.connected_count(),
                            };
                            writer_service.cache.anonymize_event_for_public(&mut resync);
                            if let Some(message) = encode_event(&resync, wire_protocol)
                                && sink.send(message).await.is_err()
                            {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            info!(connection_id = %connection_id, "Live chat broadcast channel closed");
                            break;
                        }
                    }
                }
            }
        }
        let _ = sink.close().await;
    });

    service.cache.broadcast(LiveChatServerEvent::Presence {
            connected_count: service.cache.connected_count(),
        });

    // Per-connection RTC signaling state, sharing the outbound queue with chat.
    let rtc_session = Arc::new(RtcSession::new(
        Arc::clone(&service),
        connection_id,
        actor.clone(),
        client_ip,
        out_tx.clone(),
        wire_protocol,
    ));

    // Reader loop: handle inbound frames, enqueueing any responses on out_tx.
    loop {
        let should_continue = tokio::select! {
            disconnect = disconnect_rx.changed() => {
                match disconnect {
                    Ok(()) => !*disconnect_rx.borrow_and_update(),
                    Err(_) => false,
                }
            }
            socket_message = stream.next() => {
                match socket_message {
                    Some(Ok(message)) => {
                        handle_client_message(
                            &out_tx,
                            Arc::clone(&service),
                            actor.clone(),
                            client_ip,
                            message,
                            wire_protocol,
                            &rtc_session,
                        )
                        .await
                    }
                    Some(Err(e)) => {
                        info!(error = ?e, connection_id = %connection_id, "Live chat WebSocket receive error");
                        false
                    }
                    None => false,
                }
            }
        };
        if !should_continue {
            break;
        }
    }

    // Tear down any active call for this connection before the socket closes.
    rtc_session.teardown().await;
    drop(rtc_session);

    // Drop our sender so the writer drains any still-buffered frames (e.g. a final
    // "banned"/error frame enqueued right before the reader broke) and exits on its
    // own once the queue empties. Bound the wait so a wedged client that has stopped
    // reading cannot delay teardown indefinitely; abort only if the drain stalls.
    drop(out_tx);
    let writer_abort = writer.abort_handle();
    if tokio::time::timeout(std::time::Duration::from_secs(2), writer)
        .await
        .is_err()
    {
        writer_abort.abort();
    }
    cleanup_live_chat_connection(service, connection_id, &actor).await;
}
