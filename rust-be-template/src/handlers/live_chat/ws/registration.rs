//! Pending-connection registration and RAM session revalidation.

use std::sync::Arc;

use axum_extra::extract::CookieJar;
use chrono::Utc;
use tracing::warn;
use uuid::Uuid;

use crate::{
    domain::live_chat::cache::{
        ChatActor, ChatConnectionState, DEFAULT_LIVE_CHAT_ROOM, LIVE_CHAT_MAX_CONNECTIONS,
    },
    features::accounts::domain::session::SESSION_COOKIE_NAME,
    init::state::ServerState,
};

pub(super) enum LiveChatRegistrationError {
    Disabled,
    Capacity,
    ExpiredSession,
}

pub(super) struct RegisteredLiveChatConnection {
    pub(super) connection_id: Uuid,
    pub(super) disconnect_rx: tokio::sync::watch::Receiver<bool>,
}

pub(super) async fn register_connection(
    state: &Arc<ServerState>,
    cookie_jar: &CookieJar,
    actor: &ChatActor,
) -> Result<RegisteredLiveChatConnection, LiveChatRegistrationError> {
    let connection_id = Uuid::now_v7();
    let (disconnect_tx, disconnect_rx) = tokio::sync::watch::channel(false);
    if !state
        .live_chat_cache
        .register_connection(
            connection_id,
            ChatConnectionState {
                actor: actor.clone(),
                authority_user_id: actor.user_id,
                disconnect_tx,
                room_key: DEFAULT_LIVE_CHAT_ROOM.to_owned(),
                connected_at: Utc::now(),
            },
        )
        .await
    {
        let disabled = match actor.user_id {
            Some(user_id) => state.live_chat_cache.is_connected_user_disabled(user_id),
            None => false,
        };
        warn!(
            user_id = ?actor.user_id,
            max_connections = LIVE_CHAT_MAX_CONNECTIONS,
            disabled,
            "Rejected live chat connection"
        );
        return if disabled {
            Err(LiveChatRegistrationError::Disabled)
        } else {
            Err(LiveChatRegistrationError::Capacity)
        };
    }

    if !session_is_current(state, cookie_jar, actor.user_id).await {
        state
            .live_chat_cache
            .unregister_connection(connection_id)
            .await;
        return Err(LiveChatRegistrationError::ExpiredSession);
    }

    Ok(RegisteredLiveChatConnection {
        connection_id,
        disconnect_rx,
    })
}

async fn session_is_current(
    state: &ServerState,
    cookie_jar: &CookieJar,
    user_id: Option<Uuid>,
) -> bool {
    let Some(user_id) = user_id else {
        return true;
    };
    let Some(session_cookie) = cookie_jar.get(SESSION_COOKIE_NAME) else {
        return false;
    };
    match state.session_service.lookup(session_cookie.value()).await {
        Some(session) => session.get_user_id() == user_id,
        None => false,
    }
}
