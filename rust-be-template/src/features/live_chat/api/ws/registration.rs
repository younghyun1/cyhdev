//! Pending-connection registration and RAM session revalidation.

use std::{net::IpAddr, sync::Arc};

use axum_extra::extract::CookieJar;
use chrono::Utc;
use tracing::warn;
use uuid::Uuid;

use crate::{
    features::accounts::domain::session::SESSION_COOKIE_NAME,
    features::live_chat::{
        domain::{actor::ChatActor, ip_prefix::LiveChatIpPrefix, message::DEFAULT_LIVE_CHAT_ROOM},
        service::cache::{
            ChatConnectionState, ConnectionAdmission, LIVE_CHAT_MAX_CONNECTIONS,
            LIVE_CHAT_MAX_CONNECTIONS_PER_ADDRESS,
        },
    },
    features::{
        accounts::service::session_service::SessionService,
        live_chat::service::live_chat_service::LiveChatService,
    },
};

pub(super) enum LiveChatRegistrationError {
    Disabled,
    Capacity,
    AddressLimit,
    ExpiredSession,
}

pub(super) struct RegisteredLiveChatConnection {
    pub(super) connection_id: Uuid,
    pub(super) disconnect_rx: tokio::sync::watch::Receiver<bool>,
}

pub(super) async fn register_connection(
    service: &Arc<LiveChatService>,
    sessions: &SessionService,
    cookie_jar: &CookieJar,
    actor: &ChatActor,
    client_ip: IpAddr,
) -> Result<RegisteredLiveChatConnection, LiveChatRegistrationError> {
    let connection_id = Uuid::now_v7();
    let (disconnect_tx, disconnect_rx) = tokio::sync::watch::channel(false);
    let admission = service
        .cache
        .register_connection(
            connection_id,
            ChatConnectionState {
                actor: actor.clone(),
                authority_user_id: actor.user_id,
                client_prefix: LiveChatIpPrefix::of(client_ip),
                disconnect_tx,
                room_key: DEFAULT_LIVE_CHAT_ROOM.to_owned(),
                connected_at: Utc::now(),
            },
        )
        .await;
    let rejection = match admission {
        ConnectionAdmission::Admitted => None,
        ConnectionAdmission::Disabled => Some(LiveChatRegistrationError::Disabled),
        ConnectionAdmission::Full => Some(LiveChatRegistrationError::Capacity),
        ConnectionAdmission::AddressLimit => Some(LiveChatRegistrationError::AddressLimit),
    };
    if let Some(rejection) = rejection {
        warn!(
            user_id = ?actor.user_id,
            ?admission,
            max_connections = LIVE_CHAT_MAX_CONNECTIONS,
            max_connections_per_address = LIVE_CHAT_MAX_CONNECTIONS_PER_ADDRESS,
            "Rejected live chat connection"
        );
        return Err(rejection);
    }

    if !session_is_current(sessions, cookie_jar, actor.user_id).await {
        service.cache.unregister_connection(connection_id).await;
        return Err(LiveChatRegistrationError::ExpiredSession);
    }

    Ok(RegisteredLiveChatConnection {
        connection_id,
        disconnect_rx,
    })
}

async fn session_is_current(
    sessions: &SessionService,
    cookie_jar: &CookieJar,
    user_id: Option<Uuid>,
) -> bool {
    let Some(user_id) = user_id else {
        return true;
    };
    let Some(session_cookie) = cookie_jar.get(SESSION_COOKIE_NAME) else {
        return false;
    };
    match sessions.lookup(session_cookie.value()).await {
        Some(session) => session.get_user_id() == user_id,
        None => false,
    }
}
