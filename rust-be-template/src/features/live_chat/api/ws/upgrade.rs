use std::{net::SocketAddr, sync::Arc};

use axum::{
    Extension,
    extract::{ConnectInfo, State, ws::WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use tracing::warn;

use crate::{
    features::live_chat::api::binary_codec::LIVE_CHAT_BINARY_PROTOCOL,
    init::state::ServerState,
    routers::middleware::is_logged_in::{AuthSession, AuthStatus},
    util::extract::client_ip::extract_client_ip,
};

use super::{
    LiveChatWireProtocol, actor_resolution::resolve_actor, handle_live_chat_socket,
    registration::{LiveChatRegistrationError, register_connection},
};

pub async fn live_chat_ws_handler(
    Extension(auth_status): Extension<AuthStatus>,
    Extension(auth_session): Extension<Option<AuthSession>>,
    State(state): State<Arc<ServerState>>,
    cookie_jar: CookieJar,
    ConnectInfo(socket_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let client_ip = extract_client_ip(&headers, socket_addr).unwrap_or_else(|| socket_addr.ip());
    let service = state.live_chat_service();
    let sessions = state.session_service();
    let actor = resolve_actor(Arc::clone(&service), auth_status, auth_session, client_ip).await;
    if service.is_actor_banned(actor.user_id, client_ip).await {
        return (StatusCode::FORBIDDEN, "Live chat access denied.").into_response();
    }
    let registered = match register_connection(&service, sessions.as_ref(), &cookie_jar, &actor).await {
        Ok(registered) => registered,
        Err(LiveChatRegistrationError::Disabled) => return (StatusCode::FORBIDDEN, "Live chat connection unavailable.").into_response(),
        Err(LiveChatRegistrationError::Capacity) => return (StatusCode::SERVICE_UNAVAILABLE, "Live chat connection unavailable.").into_response(),
        Err(LiveChatRegistrationError::ExpiredSession) => return (StatusCode::UNAUTHORIZED, "Live chat session expired.").into_response(),
    };
    let connection_id = registered.connection_id;
    let ws = ws.protocols([LIVE_CHAT_BINARY_PROTOCOL]);
    let wire_protocol = match ws.selected_protocol().and_then(|value| value.to_str().ok()) {
        Some(LIVE_CHAT_BINARY_PROTOCOL) => LiveChatWireProtocol::Binary,
        _ => LiveChatWireProtocol::Json,
    };
    let failed_service = Arc::clone(&service);
    ws.on_failed_upgrade(move |error| {
        warn!(error = %error, %connection_id, "Live chat WebSocket upgrade failed");
        let _cleanup = tokio::spawn(async move {
            failed_service.cache.unregister_connection(connection_id).await;
        });
    }).on_upgrade(move |socket| handle_live_chat_socket(
        socket, service, actor, client_ip, wire_protocol, connection_id, registered.disconnect_rx,
    ))
}
