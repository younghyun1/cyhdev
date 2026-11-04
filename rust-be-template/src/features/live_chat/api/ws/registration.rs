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
        accounts::service::{session_end::SessionEnd, session_service::SessionService},
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
    /// Ends a signed-in connection with its session; guests have none.
    pub(super) session_end: Option<SessionEnd>,
}

/// How a new connection relates to the session that authorized it.
enum SessionBinding {
    Guest,
    Current(SessionEnd),
    Stale,
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

    let session_end = match bind_to_session(sessions, cookie_jar, actor.user_id).await {
        SessionBinding::Guest => None,
        SessionBinding::Current(session_end) => Some(session_end),
        SessionBinding::Stale => {
            service.cache.unregister_connection(connection_id).await;
            return Err(LiveChatRegistrationError::ExpiredSession);
        }
    };

    Ok(RegisteredLiveChatConnection {
        connection_id,
        disconnect_rx,
        session_end,
    })
}

async fn bind_to_session(
    sessions: &SessionService,
    cookie_jar: &CookieJar,
    user_id: Option<Uuid>,
) -> SessionBinding {
    let Some(user_id) = user_id else {
        return SessionBinding::Guest;
    };
    let Some(session_cookie) = cookie_jar.get(SESSION_COOKIE_NAME) else {
        return SessionBinding::Stale;
    };
    match sessions.lookup_with_end(session_cookie.value()).await {
        Some((session, session_end)) if session.get_user_id() == user_id => {
            SessionBinding::Current(session_end)
        }
        Some(_) | None => SessionBinding::Stale,
    }
}
