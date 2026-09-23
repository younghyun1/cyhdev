//! Database status queries for the server-status feature.

use diesel::{QueryableByName, sql_query};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::bb8::Pool};

#[derive(QueryableByName)]
struct VersionRow {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    version_num: i32,
}

pub struct ServerStatusRepository {
    pool: Pool<AsyncPgConnection>,
}

impl ServerStatusRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }

    /// Returns `server_version_num` (for example 180001) and the query round trip.
    ///
    /// The numeric setting avoids exposing the packager's build string, which the
    /// textual `server_version` includes.
    pub async fn database_version_num(&self) -> anyhow::Result<(i32, std::time::Duration)> {
        let mut connection = self.pool.get().await?;
        let start = tokio::time::Instant::now();
        let row = sql_query("SELECT current_setting('server_version_num')::integer AS version_num")
            .get_result::<VersionRow>(&mut connection)
            .await?;
        Ok((row.version_num, start.elapsed()))
    }
}
