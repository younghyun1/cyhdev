//! Continuous seed tiles with bounded CPU work and separately refreshed permissions.

use super::{
    seed_profile::{PERMISSION_MS, Preset, Profile},
    seed_tile_cache::{Key, TileCache},
    world_query::WorldQueryService,
};
use crate::features::minecraft::{
    domain::{
        prediction::PredictionPreset,
        seed_tile::{SIDE, SeedTile, SeedTileQuery},
    },
    error::MapError,
};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::{Mutex, Semaphore};

pub(super) struct ActiveProfile {
    pub profile: Profile,
    pub epoch: String,
}

pub struct SeedTileService {
    world: Arc<WorldQueryService>,
    predictor: Option<minecraft_seed::Predictor>,
    profile: Arc<Mutex<Option<Arc<ActiveProfile>>>>,
    refresh: Arc<Mutex<Option<super::seed_profile_refresh::Receiver>>>,
    cache: TileCache,
    admission: Semaphore,
    flights: Mutex<HashMap<Key, Weak<Mutex<()>>>>,
    #[cfg(test)]
    generated: std::sync::atomic::AtomicUsize,
}

impl SeedTileService {
    pub fn new(world: Arc<WorldQueryService>) -> anyhow::Result<Self> {
        let predictor = if world.path.is_some() {
            Some(minecraft_seed::Predictor::new()?)
        } else {
            None
        };
        Ok(Self {
            world,
            predictor,
            profile: Arc::new(Mutex::new(None)),
            refresh: Arc::new(Mutex::new(None)),
            cache: TileCache::new(),
            admission: Semaphore::new(64),
            flights: Mutex::new(HashMap::new()),
            #[cfg(test)]
            generated: std::sync::atomic::AtomicUsize::new(0),
        })
    }

    /// Only the profile refresh touches Paper; warmed tiles never enter the observation gate.
    pub async fn tile(&self, query: SeedTileQuery) -> Result<SeedTile, MapError> {
        let (min_x, min_z, step) = query.geometry().ok_or(MapError::Invalid)?;
        if query.world != "minecraft:overworld" {
            return Err(MapError::Unavailable);
        }
        let _admission = self.admission.try_acquire().map_err(|_| MapError::Busy)?;
        let initial = self.profile(&query.world).await?;
        let key = Key::new(&initial.profile, &query);
        let raw = match self.cache.get(&key).await {
            Some(tile) => tile,
            None => {
                let flight = {
                    let mut flights = self.flights.lock().await;
                    flights.retain(|_, flight| flight.strong_count() > 0);
                    match flights.get(&key).and_then(Weak::upgrade) {
                        Some(flight) => flight,
                        None => {
                            let flight = Arc::new(Mutex::new(()));
                            flights.insert(key.clone(), Arc::downgrade(&flight));
                            flight
                        }
                    }
                };
                let _flight = flight.lock().await;
                match self.cache.get(&key).await {
                    Some(tile) => tile,
                    None => {
                        #[cfg(test)]
                        self.generated
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let tile = Arc::new(
                            super::seed_tile_generate::generate(
                                self.predictor.as_ref().ok_or(MapError::Disabled)?,
                                &initial.profile,
                                &query,
                                min_x,
                                min_z,
                                step,
                            )
                            .await?,
                        );
                        self.cache.insert(key, Arc::clone(&tile)).await;
                        tile
                    }
                }
            }
        };
        // A slow generation cannot extend stale permissions or publish another generator's tile.
        let latest = self.profile(&query.world).await?;
        if !initial.profile.same(&latest.profile) {
            return Err(MapError::Unavailable);
        }
        let mut indices = raw.indices.clone();
        for (index, value) in indices.iter_mut().enumerate() {
            let x = i64::from(min_x) + (index % SIDE) as i64 * i64::from(step);
            let z = i64::from(min_z) + (index / SIDE) as i64 * i64::from(step);
            if !latest.profile.permits(x, z, step) {
                *value = None;
            }
        }
        // Hidden cells must not leak their biome names through unused palette entries.
        let mut palette = Vec::new();
        let mut remap = vec![None; raw.palette.len()];
        for value in indices.iter_mut().flatten() {
            let index = usize::from(*value);
            let Some(slot) = remap.get_mut(index) else {
                return Err(MapError::Unavailable);
            };
            let entry = match *slot {
                Some(entry) => entry,
                None => {
                    let entry = palette.len() as u16;
                    let Some(biome) = raw.palette.get(index) else {
                        return Err(MapError::Unavailable);
                    };
                    palette.push(biome.clone());
                    *slot = Some(entry);
                    entry
                }
            };
            *value = entry;
        }
        Ok(SeedTile {
            query,
            min_x,
            min_z,
            step,
            preset: if latest.profile.preset == Preset::LargeBiomes {
                PredictionPreset::LargeBiomes
            } else {
                PredictionPreset::Default
            },
            profile_epoch: latest.epoch.clone(),
            sampled_at_ms: latest.profile.sampled_at_ms,
            expires_at_ms: latest.profile.sampled_at_ms + PERMISSION_MS,
            palette,
            indices,
        })
    }

    async fn profile(&self, world: &str) -> Result<Arc<ActiveProfile>, MapError> {
        if self.world.path.is_none() {
            return Err(MapError::Disabled);
        }
        {
            let cached = self.profile.lock().await;
            if let Some(active) = cached.as_ref()
                && active.profile.world == world
                && active.profile.fresh(now_ms()?)
            {
                return Ok(Arc::clone(active));
            }
        }
        let mut receiver = {
            let mut refresh = self.refresh.lock().await;
            match refresh.as_ref() {
                Some(receiver) => receiver.clone(),
                None => {
                    let (sender, receiver) = tokio::sync::watch::channel(None);
                    *refresh = Some(receiver.clone());
                    let world = Arc::clone(&self.world);
                    let cached = Arc::clone(&self.profile);
                    let refreshing = Arc::clone(&self.refresh);
                    tokio::spawn(async move {
                        let result = super::seed_profile_refresh::refresh(&world, &cached).await;
                        let _ = sender.send(Some(result));
                        *refreshing.lock().await = None;
                    });
                    receiver
                }
            }
        };
        loop {
            if let Some(result) = receiver.borrow().clone() {
                return result.map_err(Into::into);
            }
            receiver
                .changed()
                .await
                .map_err(|_| MapError::Unavailable)?;
        }
    }
}

pub(super) fn now_ms() -> Result<i64, MapError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| MapError::Unavailable)?;
    i64::try_from(elapsed.as_millis()).map_err(|_| MapError::Unavailable)
}

#[cfg(test)]
#[path = "seed_tile_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "seed_tile_concurrency_tests.rs"]
mod concurrency_tests;
