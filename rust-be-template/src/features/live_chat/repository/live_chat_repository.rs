use diesel_async::{AsyncPgConnection, pooled_connection::bb8::{Pool, PooledConnection}};

use super::super::error::LiveChatError;

#[derive(Clone)]
pub struct LiveChatRepository {
    pool: Pool<AsyncPgConnection>,
}

impl LiveChatRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self { Self { pool } }

    pub(super) async fn connection(
        &self,
    ) -> Result<PooledConnection<'_, AsyncPgConnection>, LiveChatError> {
        self.pool.get().await.map_err(LiveChatError::Pool)
    }
}
