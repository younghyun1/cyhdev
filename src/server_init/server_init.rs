use std::{net::SocketAddr, sync::Arc};

use anyhow::{anyhow, Result};
use axum::routing::get;
use chrono::{DateTime, Utc};
use tracing::info;

use crate::{
    handlers::healthcheck_handlers::{
        fallback::fallback_handler, healthcheck::healthcheck_handler,
        systemcheck::systemcheck_handler,
    },
    APP_NAME_VERSION, HOST_ADDR,
};

use super::{server_state_model::ServerState, state_init::init_state};

#[inline(always)]
pub async fn server_initializer(
    server_start: tokio::time::Instant,
    server_start_time: DateTime<Utc>,
    pw: String,
) -> Result<()> {
    // 요청 처리함수들에게 넘겨줄 state data를 여기서 만듬.
    // Constructing state data to pass to the request handlers here.
    let state: Arc<ServerState> = match init_state(server_start_time, pw).await {
        Ok(state) => Arc::new(state),
        Err(e) => return Err(anyhow!("Could not create ServerState: {:?}", e)),
    };

    // 서버 관리용.
    // For server maintenance handlers.
    let healthcheck_router: axum::Router = axum::Router::new()
        .route("/healthcheck/healthcheck", get(healthcheck_handler)) // simple healthcheck
        .route("/healthcheck/systemcheck", get(systemcheck_handler))
        .with_state(Arc::clone(&state)); // system diagnosis

    let app: axum::Router = axum::Router::new()
        .merge(healthcheck_router)
        .fallback(get(fallback_handler).with_state(Arc::clone(&state)));

    // Tokio TCP listener에 IP를 연결해주고 오류처리.
    // Bind IP address to the Tokio TCP listener here.
    let listener: tokio::net::TcpListener = match tokio::net::TcpListener::bind(HOST_ADDR).await {
        Ok(listener) => listener,
        Err(e) => {
            return Err(anyhow!("Could not initialize TcpListener: {:?}", e));
        }
    };

    // 나중에 오류처리로 넘길 것.
    // Handle error later.
    info!(
        "{}",
        format!(
            "{} started successfully on {} in {:?}.",
            APP_NAME_VERSION,
            HOST_ADDR,
            server_start.elapsed()
        )
    );

    // 여기서 앱을 Axum으로 서빙.
    // Serve app with Axum here.
    match axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Axum could not serve app: {:?}", e));
        }
    };

    Ok(())
}