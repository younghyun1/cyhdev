use ansi_to_html::convert;
use chrono::{DateTime, Utc};
use tokio::{process::Command, sync::{Mutex, RwLock}};

use crate::errors::code_error::CodeError;

pub const FASTFETCH_CACHE_MAX_BYTES: usize = 256 * 1024;
const UPDATE_INTERVAL: chrono::Duration = chrono::Duration::minutes(1);

pub struct FastFetchCache {
    value: RwLock<String>,
    last_fetched: RwLock<DateTime<Utc>>,
    update_gate: Mutex<()>,
}

impl FastFetchCache {
    pub async fn init() -> Self {
        let cache = Self::new();
        cache.initialize().await;
        cache
    }

    pub fn new() -> Self {
        Self {
            value: RwLock::new(String::new()),
            last_fetched: RwLock::new(DateTime::<Utc>::MIN_UTC),
            update_gate: Mutex::new(()),
        }
    }

    pub async fn initialize(&self) {
        if let Err(error) = self.refresh().await {
            tracing::error!(error = ?error, "Initial fastfetch population failed");
        }
    }

    pub async fn value(&self) -> Result<String, CodeError> {
        if Utc::now() - *self.last_fetched.read().await > UPDATE_INTERVAL {
            self.refresh().await?;
        }
        Ok(self.value.read().await.clone())
    }

    pub async fn get_last_fetched_time(&self) -> DateTime<Utc> {
        *self.last_fetched.read().await
    }

    pub async fn get_fastfetch_string(&self) -> String {
        self.value.read().await.clone()
    }

    pub async fn update_fastfetch_string(&self) -> Result<(), CodeError> {
        self.refresh().await
    }

    async fn refresh(&self) -> Result<(), CodeError> {
        let _update = self.update_gate.lock().await;
        if Utc::now() - *self.last_fetched.read().await <= UPDATE_INTERVAL {
            return Ok(());
        }
        let output = Command::new("fastfetch")
            .arg("--pipe")
            .arg("false")
            .arg("--logo-position")
            .arg("top")
            .env("TERM", "xterm-256color")
            .output()
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "Failed to run fastfetch");
                CodeError::COULD_NOT_RUN_FASTFETCH
            })?;
        let ansi = String::from_utf8_lossy(&output.stdout);
        let mut html = convert(&ansi).map_err(|error| {
            tracing::error!(error = %error, "Failed to convert fastfetch output");
            CodeError::COULD_NOT_RUN_FASTFETCH
        })?;
        if html.len() > FASTFETCH_CACHE_MAX_BYTES {
            let mut boundary = FASTFETCH_CACHE_MAX_BYTES;
            while !html.is_char_boundary(boundary) {
                boundary = boundary.saturating_sub(1);
            }
            html.truncate(boundary);
        }
        *self.value.write().await = html;
        *self.last_fetched.write().await = Utc::now();
        Ok(())
    }
}

impl Default for FastFetchCache {
    fn default() -> Self {
        Self::new()
    }
}
