//! Public account transport endpoint.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::{
    dto::responses::{
        response_data::http_resp, user::public_user_info_response::PublicUserInfoResponse,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::api::account_error::{AccountMutation, map_account_error},
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    get,
    path = "/api/users/{user_name}",
    tag = "user",
    params(("user_name" = String, Path, description = "Public username")),
    responses(
        (status = 200, description = "Public user information", body = PublicUserInfoResponse),
        (status = 400, description = "Invalid username", body = CodeErrorResp),
        (status = 404, description = "User not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn get_user_info(
    State(state): State<Arc<ServerState>>,
    Path(user_name): Path<String>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let account = state
        .account_service()
        .public_account(&user_name)
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?;
    let user_country_flag = state.country_flag_for_country_code(account.country).await;

    Ok(http_resp(
        PublicUserInfoResponse {
            user_id: account.user_id,
            user_name: account.user_name,
            user_created_at: account.created_at,
            user_country_flag,
            user_profile_picture_url: account.profile_picture_url,
        },
        (),
        start,
    ))
}
