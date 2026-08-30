//! PostgreSQL access point for account persistence.

use diesel_async::{
    AsyncPgConnection,
    pooled_connection::bb8::{Pool, PooledConnection},
};

use crate::features::accounts::error::AccountError;

/// Owns the database pool used by account repository operations.
#[derive(Clone)]
pub struct AccountRepository {
    pool: Pool<AsyncPgConnection>,
}

impl AccountRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }

    pub(super) async fn connection(
        &self,
    ) -> Result<PooledConnection<'_, AsyncPgConnection>, AccountError> {
        self.pool.get().await.map_err(AccountError::Pool)
    }
}
