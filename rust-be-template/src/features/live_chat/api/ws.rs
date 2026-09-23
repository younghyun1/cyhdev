use std::{net::IpAddr, sync::Arc, time::Duration};

use axum::extract::ws::WebSocket;
use futures_util::{StreamExt, stream::SplitStream};
use tracing::info;
use uuid::Uuid;

use crate::features::live_chat::{
    domain::actor::ChatActor,
    service::{cache::LiveChatServerEvent, live_chat_service::LiveChatService},
};

mod actor_resolution;
mod event_budget;
mod log_throttle;
mod message_handler;
mod persistence;
mod presence;
mod protocol;
mod registration;
mod rtc;
mod rtc_teardown;
mod upgrade;
#[cfg(test)]
mod upgrade_tests;
mod writer;

use message_handler::{FrameGuard, FrameOutcome, handle_client_frame};
use presence::cleanup_live_chat_connection;
use protocol::{OutboundSender, encode_event};
use rtc::RtcSession;
pub use upgrade::live_chat_ws_handler;
use writer::{WriterInputs, send_with_timeout, spawn_writer};

/// Bound on the per-connection outbound frame queue feeding the writer task.
const LIVE_CHAT_OUTBOUND_QUEUE: usize = 128;

const LIVE_CHAT_INITIAL_MESSAGES: usize = 50;
const LIVE_CHAT_MAX_MESSAGE_CHARS: usize = 300;
// Raised from 2 KiB to fit WebRTC SDP offers/answers, which exceed it. Chat
// message bodies remain bounded separately by `LIVE_CHAT_MAX_MESSAGE_CHARS`.
// Also the tungstenite message and frame limit set at upgrade.
pub(super) const LIVE_CHAT_MAX_FRAME_BYTES: usize = 64 * 1024;
/// Server Ping cadence; browsers answer at the protocol level without script.
const LIVE_CHAT_PING_INTERVAL: Duration = Duration::from_secs(20);
/// A connection with no inbound frame, Pong included, for this long is dead
/// or deliberately idle and is closed. Three missed pings.
const LIVE_CHAT_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
/// Longest a single outbound frame may take to reach the socket.
const LIVE_CHAT_SEND_TIMEOUT: Duration = Duration::from_secs(10);
/// Bound on draining queued frames and the close handshake at teardown.
const LIVE_CHAT_CLOSE_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy)]
pub(super) enum LiveChatWireProtocol {
    Json,
    Binary,
}

/// Everything a frame handler needs about its connection. Private to this
/// module; the child handler modules read its fields directly.
struct ConnectionContext {
    out: OutboundSender,
    service: Arc<LiveChatService>,
    actor: ChatActor,
    client_ip: IpAddr,
    wire_protocol: LiveChatWireProtocol,
    connection_id: Uuid,
    rtc: Arc<RtcSession>,
}

pub(super) async fn handle_live_chat_socket(
    socket: WebSocket,
    service: Arc<LiveChatService>,
    actor: ChatActor,
    client_ip: IpAddr,
    wire_protocol: LiveChatWireProtocol,
    connection_id: Uuid,
    disconnect_rx: tokio::sync::watch::Receiver<bool>,
) {
    let broadcasts = service.cache.subscribe();
    let (mut sink, stream) = socket.split();

    // Send the initial Hello before the writer task takes ownership of the
    // sink, so it is guaranteed to be the first frame the client sees.
    let recent_messages = service
        .cache
        .get_recent_chat_messages(LIVE_CHAT_INITIAL_MESSAGES)
        .await;
    let mut hello = LiveChatServerEvent::Hello {
        actor: actor.clone(),
        recent_messages,
        connected_count: service.cache.connected_count(),
    };
    service.cache.anonymize_event_for_public(&mut hello);
    let hello_sent = match encode_event(&hello, wire_protocol) {
        Some(message) => send_with_timeout(&mut sink, message, connection_id).await,
        None => false,
    };
    if !hello_sent {
        cleanup_live_chat_connection(service, connection_id, &actor).await;
        return;
    }

    // The writer owns the sink; a slow database persist on the read side never
    // stalls delivery of other users' broadcasts to this client.
    let (out_tx, out_rx) = tokio::sync::mpsc::channel(LIVE_CHAT_OUTBOUND_QUEUE);
    let writer = spawn_writer(WriterInputs {
        sink,
        outbound: out_rx,
        broadcasts,
        service: Arc::clone(&service),
        actor: actor.clone(),
        wire_protocol,
        connection_id,
    });

    service.cache.broadcast(LiveChatServerEvent::Presence {
        connected_count: service.cache.connected_count(),
    });

    let rtc_session = Arc::new(RtcSession::new(
        Arc::clone(&service),
        connection_id,
        actor.clone(),
        client_ip,
        out_tx.clone(),
        wire_protocol,
    ));
    let context = ConnectionContext {
        out: out_tx,
        service: Arc::clone(&service),
        actor: actor.clone(),
        client_ip,
        wire_protocol,
        connection_id,
        rtc: Arc::clone(&rtc_session),
    };
    read_frames(&context, stream, disconnect_rx).await;

    // Tear down any active call for this connection before the socket closes.
    rtc_session.teardown().await;
    drop(rtc_session);

    // Dropping the context releases the last sender, so the writer drains any
    // final frame (a "banned" error, say) and exits. Bound the wait so a peer
    // that stopped reading cannot delay teardown; abort only if it stalls.
    drop(context);
    let writer_abort = writer.abort_handle();
    if tokio::time::timeout(LIVE_CHAT_CLOSE_TIMEOUT, writer)
        .await
        .is_err()
    {
        writer_abort.abort();
    }
    cleanup_live_chat_connection(service, connection_id, &actor).await;
}

/// Read until the peer closes, the account is revoked, the writer gives up on
/// a slow consumer, a frame handler closes the connection, or nothing arrives
/// within the idle timeout.
async fn read_frames(
    context: &ConnectionContext,
    mut stream: SplitStream<WebSocket>,
    mut disconnect_rx: tokio::sync::watch::Receiver<bool>,
) {
    let mut guard = FrameGuard::new(tokio::time::Instant::now());
    let idle = tokio::time::sleep(LIVE_CHAT_IDLE_TIMEOUT);
    tokio::pin!(idle);
    loop {
        tokio::select! {
            disconnect = disconnect_rx.changed() => {
                let revoked = match disconnect {
                    Ok(()) => *disconnect_rx.borrow_and_update(),
                    Err(_) => true,
                };
                if revoked {
                    return;
                }
            }
            () = context.out.closed() => return,
            () = &mut idle => {
                info!(connection_id = %context.connection_id, "Closing idle live chat connection");
                return;
            }
            inbound = stream.next() => match inbound {
                Some(Ok(message)) => {
                    idle.as_mut().reset(tokio::time::Instant::now() + LIVE_CHAT_IDLE_TIMEOUT);
                    match handle_client_frame(context, &mut guard, message).await {
                        FrameOutcome::Continue => {}
                        FrameOutcome::Close => return,
                    }
                }
                Some(Err(error_value)) => {
                    // Includes frames above the configured size limit.
                    info!(error = %error_value, connection_id = %context.connection_id, "Live chat WebSocket receive error");
                    return;
                }
                None => return,
            },
        }
    }
}
