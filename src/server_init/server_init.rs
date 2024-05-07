use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

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

    let healthcheck_router: axum::Router = axum::Router::new()
        .route("/healthcheck/healthcheck", get(healthcheck_handler)) // simple healthcheck
        .with_state(Arc::clone(&state));

    Ok(())
}
