//! Authenticated full-profile update with current-password confirmation.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Extension, Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::{
    dto::{
        requests::auth::update_profile_request::UpdateProfileRequest,
        responses::{
            auth::update_profile_response::UpdateProfileResponse, response_data::http_resp,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::{
        api::{
            account_error::{AccountMutation, map_account_error},
            auth_abuse::request_client_ip,
            login::session_token_from_cookie,
            password_confirmation::Confirmation,
        },
        domain::account::ProfileUpdateCommand,
        error::AccountError,
    },
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    patch,
    path = "/api/auth/profile",
    tag = "auth",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Editable profile fields updated", body = UpdateProfileResponse),
        (status = 400, description = "Invalid profile field or password confirmation", body = CodeErrorResp),
        (status = 401, description = "Authentication required", body = CodeErrorResp),
        (status = 409, description = "Profile identity conflict", body = CodeErrorResp),
        (status = 429, description = "Password confirmation budget exhausted", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn update_profile(
    Extension(user_id): Extension<Uuid>,
    ConnectInfo(socket_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    cookie_jar: CookieJar,
    State(state): State<Arc<ServerState>>,
    Json(request): Json<UpdateProfileRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let confirmation = Confirmation {
        user_id,
        client_ip: request_client_ip(&headers, socket_addr),
    };
    confirmation.admit(&state).await?;
    let result = state
        .account_service()
        .update_profile(
            user_id,
            &request.current_password,
            ProfileUpdateCommand {
                user_name: request.user_name.clone(),
                country: request.user_country,
                language: request.user_language,
                subdivision: request.user_subdivision,
            },
        )
        .await;
    let profile = confirmation
        .settle(
            &state,
            session_token_from_cookie(&cookie_jar),
            result,
            map_profile_update_error,
        )
        .await?;
    Ok(http_resp(
        UpdateProfileResponse {
            user_id: profile.user_id,
            user_name: profile.user_name,
            user_country: profile.country,
            user_language: profile.language,
            user_subdivision: profile.subdivision,
        },
        (),
        start,
    ))
}

fn map_profile_update_error(error: AccountError) -> CodeErrorResp {
    map_account_error(error, AccountMutation::Update)
}
