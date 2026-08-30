use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::{
    errors::code_error::HandlerResponse,
    features::accounts::{
        domain::{
            role::RoleType,
            session::{SESSION_COOKIE_NAME, Session},
        },
        service::session_service::SessionService,
    },
};

#[derive(Clone)]
pub enum AuthStatus {
    LoggedIn(Uuid),
    LoggedOut,
}

#[derive(Clone)]
pub struct AuthSession {
    pub user_id: Uuid,
    pub role_type: RoleType,
    pub user_name: String,
    pub user_country: i32,
}

impl From<&Session> for AuthSession {
    fn from(session: &Session) -> Self {
        Self {
            user_id: session.get_user_id(),
            role_type: session.get_role_type(),
            user_name: session.get_user_name().to_string(),
            user_country: session.get_user_country(),
        }
    }
}

pub async fn is_logged_in_middleware(
    State(sessions): State<Arc<SessionService>>,
    cookie_jar: CookieJar,
    mut request: Request<Body>,
    next: Next,
) -> HandlerResponse<impl IntoResponse> {
    let mut auth_session: Option<AuthSession> = None;
    let auth_status = if let Some(session_cookie) = cookie_jar.get(SESSION_COOKIE_NAME) {
        match sessions.lookup(session_cookie.value()).await {
            Some(session) => {
                auth_session = Some(AuthSession::from(&session));
                AuthStatus::LoggedIn(session.get_user_id())
            }
            None => AuthStatus::LoggedOut,
        }
    } else {
        AuthStatus::LoggedOut
    };

    request.extensions_mut().insert(auth_status);
    request.extensions_mut().insert(auth_session.clone());

    let mut response = next.run(request).await;
    if let Some(auth_session) = auth_session {
        response.extensions_mut().insert(auth_session);
    }

    Ok(response)
}
