use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use crate::{
    errors::code_error::{CodeError, HandlerResponse, code_err},
    features::accounts::{
        domain::session::{SESSION_COOKIE_NAME, Session},
        service::session_service::SessionService,
    },
};

pub async fn auth_middleware(
    State(sessions): State<Arc<SessionService>>,
    cookie_jar: CookieJar,
    mut request: Request<Body>,
    next: Next,
) -> HandlerResponse<impl IntoResponse> {
    let session_token = match cookie_jar.get(SESSION_COOKIE_NAME) {
        Some(session_cookie) => session_cookie.value(),
        None => {
            return Err(code_err(
                CodeError::UNAUTHORIZED_ACCESS,
                "Session cookie is missing".to_string(),
            ));
        }
    };

    let session: Session = match sessions.lookup(session_token).await {
        Some(session) => session,
        None => {
            return Err(code_err(
                CodeError::UNAUTHORIZED_ACCESS,
                "Failed to retrieve session",
            ));
        }
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
