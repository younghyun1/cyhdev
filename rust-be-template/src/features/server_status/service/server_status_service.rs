//! Process-owned runtime/build/host status service.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use crate::features::server_status::{
    domain::system_info::SystemInfo,
    repository::server_status_repository::ServerStatusRepository,
    service::{fastfetch::FastFetchCache, system_info_state::SystemInfoState},
};

pub struct RuntimeStatus {
    pub uptime: tokio::time::Duration,
    pub responses_handled: u64,
    pub users_logged_in: usize,
}

pub struct ServerStateStatus {
    pub database_version: String,
    pub database_latency: std::time::Duration,
    pub runtime: RuntimeStatus,
}

pub struct HostStats {
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_free: u64,
}

impl HostStats {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(20);
        bytes.extend_from_slice(&self.cpu_usage.to_be_bytes());
        bytes.extend_from_slice(&self.memory_total.to_be_bytes());
        bytes.extend_from_slice(&self.memory_free.to_be_bytes());
        bytes
    }
}

pub struct ServerStatusService {
    repository: Arc<ServerStatusRepository>,
    system: SystemInfoState,
    fastfetch: FastFetchCache,
    app_name_version: Arc<str>,
    started_at: tokio::time::Instant,
    responses_handled: AtomicU64,
}

impl ServerStatusService {
    pub fn new(
        repository: Arc<ServerStatusRepository>,
        app_name_version: impl Into<Arc<str>>,
        started_at: tokio::time::Instant,
    ) -> Self {
        Self {
            repository,
            system: SystemInfoState::new(),
            fastfetch: FastFetchCache::new(),
            app_name_version: app_name_version.into(),
            started_at,
            responses_handled: AtomicU64::new(0),
        }
    }

    pub fn app_name_version(&self) -> &str {
        self.app_name_version.as_ref()
    }

    pub fn record_response(&self) {
        self.responses_handled.fetch_add(1, Ordering::Relaxed);
    }

    pub fn runtime(&self, users_logged_in: usize) -> RuntimeStatus {
        RuntimeStatus {
            uptime: self.started_at.elapsed(),
            responses_handled: self.responses_handled.load(Ordering::Relaxed),
            users_logged_in,
        }
    }

    pub async fn initialize(&self) {
        self.fastfetch.initialize().await;
    }

    pub async fn state(&self, runtime: RuntimeStatus) -> anyhow::Result<ServerStateStatus> {
        let (database_version, database_latency) = self.repository.database_version().await?;
        Ok(ServerStateStatus {
            database_version,
            database_latency,
            runtime,
        })
    }

    pub async fn fastfetch(&self) -> Result<String, crate::errors::code_error::CodeError> {
        self.fastfetch.value().await
    }

    pub async fn update_system_stats(&self) {
        self.system.update().await;
    }

    pub async fn host_stats(&self) -> HostStats {
        let SystemInfo {
            cpu_usage,
            memory_usage,
        } = self.system.latest().await;
        let memory_total = self.system.total_memory();
        HostStats {
            cpu_usage: cpu_usage as f32,
            memory_total,
            memory_free: memory_total.saturating_sub(memory_usage),
        }
    }
}
