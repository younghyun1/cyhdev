use std::sync::Arc;

use anyhow::{Result, Context};
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ServerState {
    pub count: Arc<RwLock<u128>>,
    pub db_url: String,
    pub writer_client: Arc<Pool<AsyncPgConnection>>,
}

pub async fn get_state() -> Result<ServerState> {
    let writer_config =
        AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(format!(
            "postgres://postgres:{}@localhost:5432/postgres",
            std::env::var("DB_PASSWORD").unwrap()
        ));
    Ok(ServerState {
        count: Arc::new(RwLock::new(0)),
        db_url: format!(
            "postgres://postgres:{}@localhost:5432/postgres",
            std::env::var("DB_PASSWORD").unwrap()
        ),
        writer_client: Arc::new(
            Pool::builder(writer_config)
                .build()
                .context("Failed to initialize writer_client.")?,
        ),
    })
}
