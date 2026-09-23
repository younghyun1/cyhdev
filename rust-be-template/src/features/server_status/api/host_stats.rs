//! Public WebSocket that streams one 20-byte host sample per second.
//!
//! Sockets are admitted through a global and per-client cap before the upgrade. The
//! client never needs to send data, so inbound frames are limited to 1 KiB and serve
//! only as liveness: the server pings every 30 seconds (browsers answer with a pong)
//! and closes a socket that has been silent for 75 seconds. Each send has a five-second
//! deadline, so a client that stops reading cannot pin the task.

use std::{future::Future, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    body::Bytes,
    extract::{
        ConnectInfo, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use tokio::time::{Instant, MissedTickBehavior};

use crate::{
    init::state::ServerState,
    util::{connection_limit::ConnectionRejection, extract::client_ip::extract_client_ip},
};

const MAX_INBOUND_BYTES: usize = 1024;
const READ_BUFFER_BYTES: usize = 4 * 1024;
const MAX_WRITE_BUFFER_BYTES: usize = 64 * 1024;

/// Per-socket timing; production values are [`SocketTiming::PRODUCTION`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct SocketTiming {
    pub(crate) sample_interval: Duration,
    pub(crate) ping_interval: Duration,
    /// Longer than two ping intervals, so one lost pong does not close a healthy socket.
    pub(crate) idle_timeout: Duration,
    pub(crate) write_timeout: Duration,
}

impl SocketTiming {
    pub(crate) const PRODUCTION: Self = Self {
        sample_interval: Duration::from_secs(1),
        ping_interval: Duration::from_secs(30),
        idle_timeout: Duration::from_secs(75),
        write_timeout: Duration::from_secs(5),
    };
}

pub async fn ws_host_stats_handler(
    State(state): State<Arc<ServerState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    websocket: WebSocketUpgrade,
) -> Response {
    let client_ip = extract_client_ip(&headers, peer).unwrap_or_else(|| peer.ip());
    let service = state.server_status_service();
    let permit = match service.admit_host_stats_socket(client_ip) {
        Ok(permit) => permit,
        Err(ConnectionRejection::ClientLimit) => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                "Too many host statistics streams.",
            )
                .into_response();
        }
        Err(ConnectionRejection::GlobalLimit) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Host statistics stream unavailable.",
            )
                .into_response();
        }
    };
    bounded_upgrade(websocket).on_upgrade(move |socket| async move {
        // The permit lives for the whole session and is released on every exit path.
        let _permit = permit;
        stream_samples(socket, SocketTiming::PRODUCTION, || {
            let service = Arc::clone(&service);
            async move { Bytes::from(service.host_stats().await.to_bytes()) }
        })
        .await;
    })
}

/// Applies the inbound size limits and small buffers before the handshake completes.
pub(crate) fn bounded_upgrade(websocket: WebSocketUpgrade) -> WebSocketUpgrade {
    websocket
        .read_buffer_size(READ_BUFFER_BYTES)
        // Samples are 20 bytes; write each immediately instead of batching.
        .write_buffer_size(0)
        .max_write_buffer_size(MAX_WRITE_BUFFER_BYTES)
        .max_message_size(MAX_INBOUND_BYTES)
        .max_frame_size(MAX_INBOUND_BYTES)
}

/// Sends a sample per interval until the client closes, errs, stops reading, or goes silent.
pub(crate) async fn stream_samples<F, Fut>(
    mut socket: WebSocket,
    timing: SocketTiming,
    mut sample: F,
) where
    F: FnMut() -> Fut,
    Fut: Future<Output = Bytes>,
{
    let mut samples = tokio::time::interval(timing.sample_interval);
    samples.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last_inbound = Instant::now();
    let mut last_ping = Instant::now();
    loop {
        tokio::select! {
            _ = samples.tick() => {
                if last_inbound.elapsed() >= timing.idle_timeout {
                    tracing::debug!("Host stats WebSocket closed after inbound silence");
                    return;
                }
                if !send(&mut socket, Message::Binary(sample().await), timing).await {
                    return;
                }
                if last_ping.elapsed() >= timing.ping_interval {
                    if !send(&mut socket, Message::Ping(Bytes::new()), timing).await {
                        return;
                    }
                    last_ping = Instant::now();
                }
            }
            inbound = socket.recv() => match inbound {
                Some(Ok(Message::Close(_))) | None => return,
                Some(Ok(_)) => last_inbound = Instant::now(),
                Some(Err(error)) => {
                    tracing::debug!(error = %error, "Host stats WebSocket read failed");
                    return;
                }
            },
        }
    }
}

async fn send(socket: &mut WebSocket, message: Message, timing: SocketTiming) -> bool {
    match tokio::time::timeout(timing.write_timeout, socket.send(message)).await {
        Ok(Ok(())) => true,
        Ok(Err(error)) => {
            tracing::debug!(error = %error, "Host stats WebSocket disconnected");
            false
        }
        Err(_) => {
            tracing::debug!("Host stats WebSocket write timed out");
            false
        }
    }
}

#[cfg(test)]
#[path = "host_stats_tests.rs"]
mod tests;
