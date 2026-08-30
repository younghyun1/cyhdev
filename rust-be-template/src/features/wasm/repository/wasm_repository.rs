//! PostgreSQL entry point for WebAssembly persistence.

use diesel_async::{
    AsyncPgConnection,
    pooled_connection::bb8::{Pool, PooledConnection},
};

use super::super::error::WasmError;

#[derive(Clone)]
pub struct WasmRepository {
    pool: Pool<AsyncPgConnection>,
}

impl WasmRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }

    pub(super) async fn connection(
        &self,
    ) -> Result<PooledConnection<'_, AsyncPgConnection>, WasmError> {
        self.pool.get().await.map_err(WasmError::Pool)
    }
}
