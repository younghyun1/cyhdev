//! Database status queries for the server-status feature.

use diesel::{QueryableByName, sql_query};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::bb8::Pool};

#[derive(QueryableByName)]
struct VersionRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    version: String,
}

pub struct ServerStatusRepository {
    pool: Pool<AsyncPgConnection>,
}

impl ServerStatusRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }

    pub async fn database_version(&self) -> anyhow::Result<(String, std::time::Duration)> {
        let mut connection = self.pool.get().await?;
        let start = tokio::time::Instant::now();
        let row = sql_query("SELECT current_setting('server_version') AS version")
            .get_result::<VersionRow>(&mut connection)
            .await?;
        Ok((row.version, start.elapsed()))
    }
}
