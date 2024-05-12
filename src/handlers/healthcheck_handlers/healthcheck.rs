use axum::http::StatusCode;

use crate::APP_NAME_VERSION;

// GET '/healthcheck/healthcheck'
// 서버 가동 여부 확인 가능.
// Simple healthcheck.
pub async fn healthcheck_handler() -> (StatusCode, std::string::String) {
    (
        StatusCode::OK,
        format!("{} good to go!", APP_NAME_VERSION),
    )
}
