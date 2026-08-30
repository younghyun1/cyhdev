use axum::{Extension, extract::State, response::IntoResponse};
use std::sync::Arc;

use crate::{
    dto::responses::{forum::topics::ForumCapabilitiesResponse, response_data::http_resp},
    errors::code_error::HandlerResponse,
    features::forum::api::error::map_forum_error,
    init::state::ServerState,
    routers::middleware::is_logged_in::AuthStatus,
    util::time::now::tokio_now,
};

#[utoipa::path(get, path = "/api/forum/capabilities", tag = "forum", responses((status = 200, body = ForumCapabilitiesResponse)))]
pub async fn forum_capabilities(
    Extension(auth): Extension<AuthStatus>,
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let user_id = match auth {
        AuthStatus::LoggedIn(user_id) => Some(user_id),
        AuthStatus::LoggedOut => None,
    };
    let capability = state
        .forum_service()
        .capabilities(user_id)
        .await
        .map_err(map_forum_error)?;
    Ok(http_resp(
        ForumCapabilitiesResponse {
            authenticated: capability.authenticated,
            can_post: capability.can_post,
            can_moderate: capability.can_moderate,
        },
        (),
        start,
    ))
}
