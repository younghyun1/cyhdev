use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::{DateTime, Utc};
use serde_derive::Serialize;
use utoipa::ToSchema;

use crate::{
    build_info::{BUILD_TIME_UTC, LIB_VERSION_MAP, RUSTC_VERSION},
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    init::state::ServerState,
    util::{time::duration_formatter::format_duration, time::now::tokio_now},
};

#[derive(Serialize, ToSchema)]
pub struct RootHandlerResponse {
    timestamp: DateTime<Utc>,
    server_uptime: String,
    responses_handled: u64,
    users_logged_in: usize,
    db_version: String,
    db_latency: String,
}

#[utoipa::path(get, path = "/api/healthcheck/state", tag = "server", responses(
    (status = 200, description = "Server state information", body = RootHandlerResponse),
    (status = 500, description = "Internal server error", body = CodeErrorResp)
))]
pub async fn root_handler(
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let status = state
        .server_status_service()
        .state(
            state
                .server_status_service()
                .runtime(state.session_service().len()),
        )
        .await
        .map_err(|error| code_err(CodeError::DB_QUERY_ERROR, error))?;
    Ok(http_resp(
        RootHandlerResponse {
            timestamp: Utc::now(),
            server_uptime: format_duration(status.runtime.uptime),
            responses_handled: status.runtime.responses_handled,
            users_logged_in: status.runtime.users_logged_in,
            db_version: status.database_version,
            db_latency: format!("{:?}", status.database_latency),
        },
        (),
        start,
    ))
}

#[derive(Serialize, ToSchema)]
pub struct ServerHealthcheckResponse {
    pub build_time: &'static str,
    pub axum_version: String,
    pub rust_version: &'static str,
}

#[utoipa::path(get, path = "/api/healthcheck/server", tag = "server", responses(
    (status = 200, description = "Server is healthy", body = ServerHealthcheckResponse)
))]
pub async fn healthcheck() -> impl IntoResponse {
    let axum_version = LIB_VERSION_MAP
        .get("axum")
        .map(|library| [library.get_name(), library.get_version()].concat())
        .unwrap_or_else(|| "Unknown".to_owned());
    (
        StatusCode::OK,
        Json(ServerHealthcheckResponse {
            build_time: BUILD_TIME_UTC,
            axum_version,
            rust_version: RUSTC_VERSION,
        }),
    )
}

#[utoipa::path(get, path = "/api/healthcheck/fastfetch", tag = "server", responses(
    (status = 200, description = "Host fastfetch information", body = String, content_type = "application/json"),
    (status = 500, description = "Internal server error", body = CodeErrorResp)
))]
pub async fn get_host_fastfetch(
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let value = state
        .server_status_service()
        .fastfetch()
        .await
        .map_err(|error| code_err(error, "Could not update fastfetch string"))?;
    Ok(http_resp(value, (), start))
}
