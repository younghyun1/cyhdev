//! A single retained refresh finishes even when its triggering viewport tile is cancelled.

use super::{
    seed_profile::Profile,
    seed_tile::{ActiveProfile, now_ms},
    world_query::{WorldQueryService, exchange},
};
use crate::features::minecraft::error::MapError;
use std::{sync::Arc, time::Duration};
use tokio::sync::{Mutex, watch};

#[derive(Clone, Copy)]
pub(super) enum Failure {
    Disabled,
    Busy,
    Unavailable,
}

impl From<Failure> for MapError {
    fn from(value: Failure) -> Self {
        match value {
            Failure::Disabled => Self::Disabled,
            Failure::Busy => Self::Busy,
            Failure::Unavailable => Self::Unavailable,
        }
    }
}

pub(super) type Receiver = watch::Receiver<Option<Result<Arc<ActiveProfile>, Failure>>>;

pub(super) async fn refresh(
    world: &WorldQueryService,
    cached: &Mutex<Option<Arc<ActiveProfile>>>,
) -> Result<Arc<ActiveProfile>, Failure> {
    let path = world.path.as_ref().ok_or(Failure::Disabled)?;
    let mut gate = world.gate.try_lock().map_err(|_| Failure::Busy)?;
    let now = tokio::time::Instant::now();
    if now < *gate {
        return Err(Failure::Busy);
    }
    *gate = now + Duration::from_secs(21);
    let result = tokio::time::timeout(
        Duration::from_secs(16),
        exchange(
            path,
            b"{\"kind\":\"seed_profile\",\"world\":\"minecraft:overworld\"}\n",
        ),
    )
    .await;
    *gate = tokio::time::Instant::now() + Duration::from_secs(1);
    let bytes = match result {
        Ok(Ok(bytes)) => bytes,
        _ => return Err(Failure::Unavailable),
    };
    let profile = Profile::parse(
        &bytes,
        "minecraft:overworld",
        now_ms().map_err(|_| Failure::Unavailable)?,
    )
    .map_err(|_| Failure::Unavailable)?;
    let mut cached = cached.lock().await;
    let epoch = match cached.as_ref() {
        Some(previous) if previous.profile.same(&profile) => previous.epoch.clone(),
        _ => uuid::Uuid::new_v4().to_string(),
    };
    let active = Arc::new(ActiveProfile { profile, epoch });
    *cached = Some(Arc::clone(&active));
    Ok(active)
}
