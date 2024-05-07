use axum::{http::StatusCode, response::IntoResponse};


pub async fn healthcheck_aws_handler() -> impl IntoResponse {
    let version_str = "cyhdev-0.1.0";
    (
        StatusCode::OK,
        format!("{} reports a healthy 200!", version_str),
    )
}
