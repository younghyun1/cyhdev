use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;
use tracing::info;

use super::server_state_model::ServerState;
use crate::{server_init::server_state_model::ShuffleBag, DB_ADDR, DB_NAME, DB_PORT, DB_USERNAME};

pub async fn init_state(server_start_time: DateTime<Utc>, pw: String) -> Result<ServerState> {
    let start = tokio::time::Instant::now();
    let mut cfg: Config = Config::new();
    cfg.url = Some(format!(
        "postgres://{}:{}@{}:{}/{}",
        DB_USERNAME,
        pw,
        DB_ADDR.to_string(),
        DB_PORT,
        DB_NAME
    ));
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    info!(
        "deadpool-postgres config inititalized in {:?}",
        start.elapsed()
    );

    let start = tokio::time::Instant::now();
    let pool = Arc::new(match cfg.create_pool(Some(Runtime::Tokio1), NoTls) {
        Ok(pool) => pool,
        Err(e) => {
            return Err(anyhow!("Could not create pool with configuration: {:?}", e));
        }
    });
    info!("Connection pool established in {:?}", start.elapsed());

    Ok(ServerState {
        pool,
        server_start_time,
        quote_shuffle_bag: ShuffleBag::new(),
    })
}
