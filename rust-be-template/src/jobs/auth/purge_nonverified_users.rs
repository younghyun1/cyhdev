use std::sync::Arc;

use tracing::{error, info};

use crate::init::state::ServerState;

pub async fn purge_nonverified_users(state: Arc<ServerState>) {
    match state.account_service().purge_unverified_accounts().await {
        Ok(number_of_users_deleted) => {
            info!(
                number_of_users_deleted,
                "Deleted non-verified users with expired verification tokens"
            );
        }
        Err(error) => {
            error!(%error, "Failed to purge non-verified users");
        }
    }
}
