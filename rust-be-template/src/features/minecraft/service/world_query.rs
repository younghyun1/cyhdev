//! Single-flight, uncached terrain reads over a same-user Unix socket.

use super::world_wire;
use crate::features::minecraft::{
    domain::{map_data::MapData, map_query::MapQuery},
    error::MapError,
};
use std::{path::PathBuf, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::Mutex,
    time::Instant,
};

const MAX_REPLY: usize = 2 * 1024 * 1024;
pub struct WorldQueryService {
    path: Option<PathBuf>,
    gate: Mutex<Instant>,
}

impl WorldQueryService {
    pub fn from_environment() -> anyhow::Result<Self> {
        Self::from_path(std::env::var_os("MINECRAFT_WORLD_SOCKET").map(PathBuf::from))
    }

    fn from_path(path: Option<PathBuf>) -> anyhow::Result<Self> {
        if let Some(path) = &path {
            // macOS sockaddr_un is smaller than Linux's; retain one portable bound.
            anyhow::ensure!(
                path.is_absolute()
                    && path.file_name().is_some()
                    && path.as_os_str().as_encoded_bytes().len() < 104
                    && !path.as_os_str().as_encoded_bytes().contains(&0),
                "MINECRAFT_WORLD_SOCKET must be an absolute Unix socket path shorter than 104 bytes"
            );
        }
        Ok(Self {
            path,
            gate: Mutex::new(Instant::now()),
        })
    }

    /// Holding the only slot across I/O prevents browsers from building a server work queue.
    pub async fn query(&self, query: MapQuery) -> Result<MapData, MapError> {
        if !query.valid() {
            return Err(MapError::Invalid);
        }
        let path = self.path.as_ref().ok_or(MapError::Disabled)?;
        let mut gate = self.gate.try_lock().map_err(|_| MapError::Busy)?;
        if Instant::now() < *gate {
            return Err(MapError::Busy);
        }
        // Browser cancellation can leave Paper finishing its read; reserve the full deadline
        // until an acknowledged response safely shortens the cooldown.
        *gate = Instant::now() + Duration::from_secs(21);
        let result = tokio::time::timeout(Duration::from_secs(20), async {
            let request = world_wire::request(&query)?;
            anyhow::ensure!(request.len() <= 4096, "World request too large");
            let mut stream = UnixStream::connect(path).await?;
            stream.write_all(&request).await?;
            let mut bytes = Vec::new();
            stream
                .take((MAX_REPLY + 1) as u64)
                .read_to_end(&mut bytes)
                .await?;
            anyhow::ensure!(bytes.len() <= MAX_REPLY, "World response too large");
            world_wire::parse(&bytes, &query)
        })
        .await;
        *gate = Instant::now() + Duration::from_secs(1);
        match result {
            Ok(Ok(data)) => Ok(data),
            _ => {
                tracing::warn!("Minecraft world query unavailable");
                Err(MapError::Unavailable)
            }
        }
    }
}

#[cfg(test)]
#[path = "world_query_tests.rs"]
mod tests;
