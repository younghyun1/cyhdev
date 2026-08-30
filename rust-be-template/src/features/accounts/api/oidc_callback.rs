//! State-bound provider callback with same-origin two-phase link completion.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderValue, header},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{
    errors::code_error::{CodeError, HandlerResponse, code_err},
    features::accounts::{
        api::login::{session_cookie, session_token_from_cookie},
        domain::oidc::OidcFlowMode,
        service::oidc::provider::OidcCallbackOutcome,
    },
    init::state::ServerState,
};

#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct OidcCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/auth/oidc/callback",
    tag = "auth",
    params(
        ("code" = Option<String>, Query, description = "One-time provider authorization code"),
        ("state" = Option<String>, Query, description = "One-time CSRF state"),
        ("error" = Option<String>, Query, description = "Provider denial code")
    ),
    responses(
        (status = 303, description = "Redirect to the exact public application origin"),
        (status = 404, description = "OIDC is not configured")
    )
)]
pub async fn oidc_callback(
    cookie_jar: CookieJar,
    State(state): State<Arc<ServerState>>,
    Query(query): Query<OidcCallbackQuery>,
) -> HandlerResponse<Response> {
    let public_origin = state.public_app_origin();
    let oidc = state.oidc_service();
    if !oidc.is_enabled() {
        return Err(code_err(CodeError::OIDC_DISABLED, "OIDC is disabled"));
    }

    if query.error.is_some() {
        let mode = match query.state.as_deref() {
            Some(state_token) => oidc.cancel_authorization(state_token).await,
            None => None,
        };
        return Ok(failure_redirect(public_origin.as_str(), mode));
    }
    let (state_token, code) = match (query.state.as_deref(), query.code.as_deref()) {
        (Some(state_token), Some(code)) => (state_token, code),
        _ => return Ok(failure_redirect(public_origin.as_str(), None)),
    };

    let outcome = match oidc.finish_authorization(state_token, code).await {
        Ok(outcome) => outcome,
        Err(error) => {
            tracing::warn!(
                event = "oidc_callback_rejected",
                error = %error,
                "OpenID Connect callback rejected"
            );
            return Ok(failure_redirect(public_origin.as_str(), None));
        }
    };
    match outcome {
        OidcCallbackOutcome::Login(identity) => {
            let receipt = match state
                .account_service()
                .oidc_login(&identity, session_token_from_cookie(&cookie_jar))
                .await
            {
                Ok(receipt) => receipt,
                Err(error) => {
                    tracing::warn!(
                        event = "oidc_login_rejected",
                        error = %error,
                        "OpenID Connect login rejected"
                    );
                    return Ok(failure_redirect(public_origin.as_str(), None));
                }
            };
            redirect_response(
                &format!("{}/login#oidc=success", public_origin.as_str()),
                Some(session_cookie(&receipt.session_token)),
            )
        }
        OidcCallbackOutcome::LinkReady { completion_token } => redirect_response(
            &format!(
                "{}/edit-profile#oidc_link_token={}",
                public_origin.as_str(),
                completion_token.expose()
            ),
            None,
        ),
    }
}

fn failure_redirect(public_origin: &str, mode: Option<OidcFlowMode>) -> Response {
    let path = match mode {
        Some(OidcFlowMode::Link { .. }) => "edit-profile",
        Some(OidcFlowMode::Login) | None => "login",
    };
    hardened_redirect(&format!("{public_origin}/{path}#oidc=failed"))
}

fn redirect_response(
    location: &str,
    cookie: Option<axum_extra::extract::cookie::Cookie<'static>>,
) -> HandlerResponse<Response> {
    let mut response = hardened_redirect(location);
    if let Some(cookie) = cookie {
        let header_value = HeaderValue::from_str(&cookie.to_string()).map_err(|error| {
            code_err(
                CodeError::SESSION_CREATION_FAILED,
                format!("could not encode session cookie: {error}"),
            )
        })?;
        response
            .headers_mut()
            .append(header::SET_COOKIE, header_value);
    }
    Ok(response)
}

fn hardened_redirect(location: &str) -> Response {
    let mut response = Redirect::to(location).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, max-age=0"),
    );
    response.headers_mut().insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_completion_credential_is_placed_only_in_fragment() {
        let response =
            hardened_redirect("https://app.example.test/edit-profile#oidc_link_token=secret");
        let location = response.headers().get(header::LOCATION);
        assert_eq!(
            location.and_then(|value| value.to_str().ok()),
            Some("https://app.example.test/edit-profile#oidc_link_token=secret")
        );
        assert_eq!(
            response.headers().get(header::REFERRER_POLICY),
            Some(&HeaderValue::from_static("no-referrer"))
        );
    }
}
