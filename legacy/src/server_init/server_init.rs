use crate::handlers::api::submit_message::submit_handler;
use crate::handlers::{
    health_check::healthcheck_handler, health_check_aws::healthcheck_aws_handler,
};
use crate::server_init::state_init::{get_state, ServerState};
use anyhow::{Context, Result};
use axum::routing::post;
use axum::{routing::get, Router};
use colored::Colorize;
use tower_http::services::ServeDir;
use std::net::SocketAddr;
use std::time::Instant;

#[inline(always)]
pub async fn server_initializer() -> Result<String> {
    let start = Instant::now();
    let app_name_version: String = String::from("cyhdev-0.1.0");

    let hosting_address: SocketAddr = "[::]:10000"
        .parse()
        .context("Failed to parse hosting address")?;

    let static_files_service = ServeDir::new("./front");

    let state: ServerState = get_state().await?;

    let app: Router = Router::new()
        .nest_service("/", static_files_service)
        .route("/health", get(healthcheck_handler))
        .route("/health_aws", get(healthcheck_aws_handler))
        .route("/api/submit", post(submit_handler))
        .with_state(state);

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind(hosting_address)
        .await
        .context("Failed to bind TCP listener")?;

    let duration: std::time::Duration = start.elapsed();

    println!(
        "{}",
        format!(
            "{} started successfully on {} in {:?}.",
            app_name_version, hosting_address, duration
        )
        .green()
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok("Server exiting.".to_string())
}
