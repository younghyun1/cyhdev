use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Result};
use axum::{
    http::StatusCode,
    routing::{get, get_service},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use chrono::{DateTime, Utc};
use tower_http::services::{ServeDir, ServeFile};
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

    // Serves front.
    let front_router = Router::new()
        .route(
            "/", // Serve index.html directly at the root
            get_service(ServeFile::new("/home/cyh/cyhdev/front/index.html")).handle_error(
                |e| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Could not serve file: {}", e),
                    )
                },
            ),
        )
        .nest_service(
            "/front", // Serve all files under /assets path
            get_service(ServeDir::new("/home/cyh/cyhdev/front")).handle_error(|e| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Could not serve directory: {}", e),
                )
            }),
        );

    // 서버 관리용.
    // For server maintenance handlers.
    let healthcheck_router: axum::Router = axum::Router::new()
        .route("/api/healthcheck", get(healthcheck_handler)) // simple healthcheck
        .route("/api/systemcheck", get(systemcheck_handler))
        .with_state(Arc::clone(&state)); // system diagnosis

    // Final app.
    let app: axum::Router = axum::Router::new()
        .fallback(get(fallback_handler).with_state(Arc::clone(&state)))
        .merge(front_router)
        .merge(healthcheck_router);

    // TLS config.
    let config = match RustlsConfig::from_pem_file(
        PathBuf::from("/home/cyh/cyhdev/src/server_init/keys/cert.pem"),
        PathBuf::from("/home/cyh/cyhdev/src/server_init/keys/priv.pem"),
    )
    .await
    {
        Ok(cfg) => cfg,
        Err(e) => {
            return Err(anyhow!("Could not configure Rustls: {:?}", e));
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
    // Serve app here.
    match axum_server::bind_rustls(HOST_ADDR, config)
        .serve(app.into_make_service())
        .await
    {
        Ok(_) => {
            info!("Server terminating.")
        }
        Err(e) => {
            return Err(anyhow!("Server terminating with error: {:?}", e));
        }
    };

    Ok(())
}
