use std::collections::VecDeque;

use tokio::sync::RwLock;

use crate::features::server_status::{
    domain::system_info::SystemInfo,
    repository::platform::{cpu_usage, memory_usage, total_memory},
};

const SYSTEM_HISTORY_MAX_SAMPLES: usize = 3_600;

pub struct SystemInfoState {
    history: RwLock<VecDeque<SystemInfo>>,
    total_memory: u64,
}

impl Default for SystemInfoState {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemInfoState {
    pub fn new() -> Self {
        Self {
            history: RwLock::new(VecDeque::with_capacity(SYSTEM_HISTORY_MAX_SAMPLES)),
            total_memory: total_memory(),
        }
    }

    pub async fn update(&self) {
        let cpu_usage = cpu_usage().await;
        let memory_usage = match tokio::task::spawn_blocking(memory_usage).await {
            Ok(usage) => usage,
            Err(error) => {
                tracing::error!(error = %error, "Host memory sampling task failed");
                0
            }
        };
        let mut history = self.history.write().await;
        if history.len() == SYSTEM_HISTORY_MAX_SAMPLES {
            history.pop_front();
        }
        history.push_back(SystemInfo {
            cpu_usage,
            memory_usage,
        });
    }

    pub async fn latest(&self) -> SystemInfo {
        self.history
            .read()
            .await
            .back()
            .copied()
            .unwrap_or(SystemInfo {
                cpu_usage: 0.0,
                memory_usage: 0,
            })
    }

    pub const fn total_memory(&self) -> u64 {
        self.total_memory
    }
}
