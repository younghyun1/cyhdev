use anyhow::Result;
use chrono::{DateTime, Utc};

use super::server_state_model::ServerState;

pub async fn get_state(server_start_time: DateTime<Utc>, pw: String) -> Result<ServerState> {
    
}
