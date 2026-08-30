//! PostgreSQL access point for forum persistence.

use diesel_async::{
    AsyncPgConnection,
    pooled_connection::bb8::{Pool, PooledConnection},
};

use crate::features::forum::error::ForumError;

#[derive(Clone)]
pub struct ForumRepository {
    pool: Pool<AsyncPgConnection>,
}

impl ForumRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self { Self { pool } }

    pub(super) async fn connection(
        &self,
    ) -> Result<PooledConnection<'_, AsyncPgConnection>, ForumError> {
        self.pool.get().await.map_err(ForumError::Pool)
    }
}
