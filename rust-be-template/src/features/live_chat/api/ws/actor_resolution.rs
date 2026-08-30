//! RAM-authoritative actor presentation for live-chat connections.

use std::{net::IpAddr, sync::Arc};

use crate::{
    features::live_chat::domain::actor::ChatActor,
    features::live_chat::service::live_chat_service::LiveChatService,
    routers::middleware::is_logged_in::{AuthSession, AuthStatus},
};

pub(super) async fn resolve_actor(
    service: Arc<LiveChatService>,
    auth_status: AuthStatus,
    auth_session: Option<AuthSession>,
    client_ip: IpAddr,
) -> ChatActor {
    match auth_status {
        AuthStatus::LoggedIn(user_id) => match auth_session {
            Some(session) if session.user_id == user_id => {
                match service
                    .user_actor(user_id, session.user_name, session.user_country)
                    .await
                {
                    Ok(actor) => actor,
                    Err(_) => ChatActor::user(user_id, format!("user@{user_id}"), None, None),
                }
            }
            _ => ChatActor::user(user_id, format!("user@{user_id}"), None, None),
        },
        AuthStatus::LoggedOut => service.guest_actor(client_ip).await,
    }
}
