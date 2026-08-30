use std::sync::Arc;

use axum::{body::Bytes, extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}}, response::Response};

use crate::init::state::ServerState;

pub async fn ws_host_stats_handler(
    State(state): State<Arc<ServerState>>,
    websocket: WebSocketUpgrade,
) -> Response {
    websocket.on_upgrade(move |socket| host_stats_socket(socket, state))
}

async fn host_stats_socket(mut socket: WebSocket, state: Arc<ServerState>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        interval.tick().await;
        let stats = state.server_status_service().host_stats().await;
        if let Err(error) = socket
            .send(Message::Binary(Bytes::from(stats.to_bytes())))
            .await
        {
            tracing::debug!(error = %error, "Host stats WebSocket disconnected");
            return;
        }
    }
}
