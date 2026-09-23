use std::process::Stdio;

use ansi_to_html::convert;
use chrono::{DateTime, Utc};
use tokio::{
    process::Command,
    sync::{Mutex, RwLock},
};

use crate::errors::code_error::CodeError;

pub const FASTFETCH_CACHE_MAX_BYTES: usize = 256 * 1024;
const UPDATE_INTERVAL: chrono::Duration = chrono::Duration::minutes(1);
/// The output is public, so the child gets a hard deadline and is killed if it overruns.
const FASTFETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
/// Modules shown on the public statistics page. An explicit `--structure` replaces both
/// fastfetch's default list and any module list in a config file, so host identifiers
/// never run: `Title` (`user@hostname`), `LocalIp`, `PublicIp`, `Users` (login
/// sources), `DNS`, `Wifi`, and desktop-session modules are deliberately absent.
pub const FASTFETCH_PUBLIC_MODULES: &str =
    "OS:Host:Kernel:Uptime:Packages:Shell:CPU:GPU:Memory:Swap:Disk:Locale:Break:Colors";

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
        let child = Command::new("fastfetch")
            .args(fastfetch_arguments())
            .env("TERM", "xterm-256color")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .output();
        // Dropping the timed-out future drops the child, which kill_on_drop terminates.
        let output = match tokio::time::timeout(FASTFETCH_TIMEOUT, child).await {
            Ok(Ok(output)) => output,
            Ok(Err(error)) => {
                tracing::error!(error = %error, "Failed to run fastfetch");
                return Err(CodeError::COULD_NOT_RUN_FASTFETCH);
            }
            Err(_) => {
                tracing::error!(
                    timeout_ms = FASTFETCH_TIMEOUT.as_millis(),
                    "fastfetch timed out and was killed"
                );
                return Err(CodeError::COULD_NOT_RUN_FASTFETCH);
            }
        };
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

fn fastfetch_arguments() -> [&'static str; 6] {
    [
        "--pipe",
        "false",
        "--logo-position",
        "top",
        "--structure",
        FASTFETCH_PUBLIC_MODULES,
    ]
}

#[cfg(test)]
mod tests {
    use super::{FASTFETCH_PUBLIC_MODULES, fastfetch_arguments};

    #[test]
    fn public_structure_excludes_host_identifying_modules() {
        let modules = FASTFETCH_PUBLIC_MODULES.split(':').collect::<Vec<_>>();
        for identifying in [
            "Title",
            "LocalIp",
            "PublicIp",
            "Users",
            "DNS",
            "Wifi",
            "Terminal",
            "Separator",
        ] {
            assert!(!modules.contains(&identifying), "{identifying}");
        }
        let arguments = fastfetch_arguments();
        assert!(
            arguments
                .windows(2)
                .any(|pair| pair == ["--structure", FASTFETCH_PUBLIC_MODULES])
        );
    }
}
