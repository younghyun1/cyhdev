use std::sync::Arc;

use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;

pub struct ServerState {
    pub pool: Arc<Pool>,
    pub server_start_time: DateTime<Utc>,
}
