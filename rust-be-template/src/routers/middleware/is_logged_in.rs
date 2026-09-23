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

/// Session lookup result published by the outer middleware for the inner `auth_middleware`.
///
/// Only server middleware inserts request extensions, so a browser cannot forge this value.
/// Reusing it saves a second hash derivation and map lookup on every authenticated request.
#[derive(Clone)]
pub enum ResolvedSession {
    Active(Session),
    Absent,
}

pub async fn is_logged_in_middleware(
    State(sessions): State<Arc<SessionService>>,
    cookie_jar: CookieJar,
    mut request: Request<Body>,
    next: Next,
) -> HandlerResponse<impl IntoResponse> {
    let session = match cookie_jar.get(SESSION_COOKIE_NAME) {
        Some(session_cookie) => sessions.lookup(session_cookie.value()).await,
        None => None,
    };
    let auth_session = session.as_ref().map(AuthSession::from);
    let auth_status = match &session {
        Some(session) => AuthStatus::LoggedIn(session.get_user_id()),
        None => AuthStatus::LoggedOut,
    };
    let resolved = match session {
        Some(session) => ResolvedSession::Active(session),
        None => ResolvedSession::Absent,
    };

    request.extensions_mut().insert(auth_status);
    request.extensions_mut().insert(auth_session.clone());
    request.extensions_mut().insert(resolved);

    let mut response = next.run(request).await;
    if let Some(auth_session) = auth_session {
        response.extensions_mut().insert(auth_session);
    }

    Ok(response)
}
