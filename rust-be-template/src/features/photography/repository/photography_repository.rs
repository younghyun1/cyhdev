use super::super::error::PhotographyError;
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::bb8::{Pool, PooledConnection},
};

#[derive(Clone)]
pub struct PhotographyRepository {
    pool: Pool<AsyncPgConnection>,
}

impl PhotographyRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }
    pub(super) async fn connection(
        &self,
    ) -> Result<PooledConnection<'_, AsyncPgConnection>, PhotographyError> {
        self.pool.get().await.map_err(PhotographyError::Pool)
    }
}
