use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::bb8::PooledConnection;
use std::sync::Arc;
use super::ServerState;
use crate::features::accounts::service::{
    account_service::AccountService, auth_abuse::AuthAbuseService,
    session_service::SessionService,
};
use crate::init::state::{DeploymentEnvironment, ServerStateBuilder};

impl ServerState {
    pub fn builder() -> ServerStateBuilder {
        ServerStateBuilder::default()
    }

    pub fn get_app_name_version(&self) -> String {
        self.app_name_version.clone()
    }

    pub fn get_uptime(&self) -> tokio::time::Duration {
        self.server_start_time.elapsed()
    }

    pub async fn get_conn(&self) -> anyhow::Result<PooledConnection<'_, AsyncPgConnection>> {
        Ok(self.pool.get().await?)
    }

    pub fn account_service(&self) -> Arc<AccountService> {
        Arc::clone(&self.account_service)
    }

    pub fn auth_abuse_service(&self) -> Arc<AuthAbuseService> {
        Arc::clone(&self.auth_abuse_service)
    }

    pub fn session_service(&self) -> Arc<SessionService> {
        Arc::clone(&self.session_service)
    }

    pub fn get_responses_handled(&self) -> u64 {
        std::sync::atomic::AtomicU64::load(
            &self.responses_handled,
            std::sync::atomic::Ordering::SeqCst,
        )
    }

    pub fn add_responses_handled(&self) {
        self.responses_handled
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get_deployment_environment(&self) -> DeploymentEnvironment {
        self.deployment_environment
    }

    pub fn get_request_client(&self) -> &reqwest::Client {
        &self.request_client
    }
}
