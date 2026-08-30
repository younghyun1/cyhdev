//! RAM-authoritative actor presentation for live-chat connections.

use std::{net::IpAddr, sync::Arc};

use crate::{
    domain::live_chat::cache::ChatActor,
    init::state::ServerState,
    routers::middleware::is_logged_in::{AuthSession, AuthStatus},
};

pub(super) async fn resolve_actor(
    state: Arc<ServerState>,
    auth_status: AuthStatus,
    auth_session: Option<AuthSession>,
    client_ip: IpAddr,
) -> ChatActor {
    match auth_status {
        AuthStatus::LoggedIn(user_id) => {
            let (display_name, country_flag, user_profile_picture_url) = match auth_session {
                Some(session) if session.user_id == user_id => {
                    let country_flag = state
                        .country_flag_for_country_code(session.user_country)
                        .await;
                    let user_profile_picture_url =
                        state.latest_user_profile_picture_url(user_id).await;
                    (session.user_name, country_flag, user_profile_picture_url)
                }
                _ => (format!("user@{user_id}"), None, None),
            };
            ChatActor::user(
                user_id,
                display_name,
                country_flag,
                user_profile_picture_url,
            )
        }
        AuthStatus::LoggedOut => {
            let country_flag = state.country_flag_for_ip(client_ip).await;
            ChatActor::guest(client_ip, country_flag)
        }
    }
}
