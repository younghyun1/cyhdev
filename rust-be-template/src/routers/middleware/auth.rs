use std::sync::Arc;

use crate::{
    errors::code_error::{CodeError, HandlerResponse, code_err},
    features::accounts::{
        domain::session::{SESSION_COOKIE_NAME, Session},
        service::session_service::SessionService,
    },
    routers::middleware::is_logged_in::ResolvedSession,
};
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;

pub async fn auth_middleware(
    State(sessions): State<Arc<SessionService>>,
    cookie_jar: CookieJar,
    mut request: Request<Body>,
    next: Next,
) -> HandlerResponse<impl IntoResponse> {
    let session = match request.extensions().get::<ResolvedSession>() {
        Some(ResolvedSession::Active(session)) => session.clone(),
        Some(ResolvedSession::Absent) => {
            return Err(code_err(
                CodeError::UNAUTHORIZED_ACCESS,
                "Failed to retrieve session",
            ));
        }
        None => lookup_session(&sessions, &cookie_jar).await?,
    };

    if !session.get_is_email_verified() {
        return Err(code_err(
            CodeError::EMAIL_NOT_VERIFIED,
            "Email is not verified".to_string(),
        ));
    }

    request.extensions_mut().insert(session.get_user_id());
    request.extensions_mut().insert(session.get_role_type());

    let response = next.run(request).await;

    Ok(response)
}

/// Resolves the session directly when no outer middleware already did.
async fn lookup_session(
    sessions: &SessionService,
    cookie_jar: &CookieJar,
) -> HandlerResponse<Session> {
    let session_token = match cookie_jar.get(SESSION_COOKIE_NAME) {
        Some(session_cookie) => session_cookie.value(),
        None => {
            return Err(code_err(
                CodeError::UNAUTHORIZED_ACCESS,
                "Session cookie is missing".to_string(),
            ));
        }
    };
    match sessions.lookup(session_token).await {
        Some(session) => Ok(session),
        None => Err(code_err(
            CodeError::UNAUTHORIZED_ACCESS,
            "Failed to retrieve session",
        )),
    }
}
