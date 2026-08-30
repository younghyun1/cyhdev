use diesel_async::{
    AsyncPgConnection,
    pooled_connection::bb8::{Pool, PooledConnection},
};

use super::super::error::BlogError;

#[derive(Clone)]
pub struct BlogRepository {
    pool: Pool<AsyncPgConnection>,
}

impl BlogRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }

    pub(super) async fn connection(
        &self,
    ) -> Result<PooledConnection<'_, AsyncPgConnection>, BlogError> {
        self.pool.get().await.map_err(BlogError::Pool)
    }
}
