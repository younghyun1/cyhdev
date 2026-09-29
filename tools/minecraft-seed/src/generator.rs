//! Initialize only the biome router, without a server, chunks or world storage.

use pumpkin_data::{
    chunk::OVERWORLD_BIOME_SOURCE,
    noise_router::{LARGE_BIOMES_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER},
};
use pumpkin_world::generation::{
    GlobalRandomConfig,
    noise::router::{
        density_volume::DensityVolume,
        multi_noise_sampler::MultiNoiseSampler,
        proto_noise_router::{ProtoMultiNoiseRouter, ProtoNoiseRouters},
    },
};

use crate::{Cell, Error, PredictionRequest, PredictionResponse};

/// Exact source and generated-data revision used by this executable.
pub const GENERATOR_REVISION: &str = "pumpkin-4426d1113a211e6018a2db416e33b6b8a7802614-java26.3";

/// Construct a seed-specific router using vanilla default or large-biomes noise.
pub(crate) fn router(seed: i64, large_biomes: bool) -> ProtoMultiNoiseRouter {
    let base = if large_biomes {
        &LARGE_BIOMES_BASE_NOISE_ROUTER.multi_noise
    } else {
        &OVERWORLD_BASE_NOISE_ROUTER.multi_noise
    };
    let random = GlobalRandomConfig::new(seed as u64, false);
    ProtoMultiNoiseRouter {
        full_component_stack: ProtoNoiseRouters::generate_proto_stack(
            base.full_component_stack,
            &random,
        ),
        temperature: base.temperature,
        vegetation: base.vegetation,
        continents: base.continents,
        erosion: base.erosion,
        depth: base.depth,
        ridges: base.ridges,
    }
}

/// Predict a bounded fixed-height grid, including the true large-biomes preset.
pub fn predict(request: &PredictionRequest) -> Result<PredictionResponse, Error> {
    request.validate()?;
    let router = router(request.seed, request.large_biomes);
    let mut sampler = MultiNoiseSampler::generate(&router);
    sample(request, &mut sampler, || false)
}

/// Reuse seeded state on its owning CPU thread; cancellation is checked between rows.
pub(crate) fn sample(
    request: &PredictionRequest,
    sampler: &mut MultiNoiseSampler<'_>,
    cancelled: impl Fn() -> bool,
) -> Result<PredictionResponse, Error> {
    request.validate()?;
    if cancelled() {
        return Err(Error::Cancelled);
    }

    // Aligned quart grids can use Pumpkin's volume path. Other grids must round
    // each coordinate separately, since a block step need not be a quart step.
    if request.step.is_multiple_of(4)
        && request.min_x % 4 == 0
        && request.min_z % 4 == 0
        && let Ok(step) = i32::try_from(request.step)
    {
        sampler.fill_volume(DensityVolume::new(
            request.width as usize,
            1,
            request.height as usize,
            request.min_x,
            (request.y >> 2) << 2,
            request.min_z,
            step,
            4,
            step,
        ));
    }

    let mut cells = Vec::with_capacity((request.width * request.height) as usize);
    for row in 0..request.height {
        if cancelled() {
            return Err(Error::Cancelled);
        }
        for column in 0..request.width {
            let x = i64::from(request.min_x) + i64::from(column) * i64::from(request.step);
            let z = i64::from(request.min_z) + i64::from(row) * i64::from(request.step);
            let x = i32::try_from(x).map_err(|_| Error::Bounds)?;
            let z = i32::try_from(z).map_err(|_| Error::Bounds)?;
            let point = sampler.sample(x >> 2, request.y >> 2, z >> 2);
            // Vanilla preserves the prior leaf on equal-distance ties. A map
            // sample must stay stable when the viewport or request order changes.
            let biome = OVERWORLD_BIOME_SOURCE.get(&point.convert_to_list(), &mut None);
            cells.push(Cell {
                x,
                z,
                biome: format!("minecraft:{}", biome.registry_id),
            });
        }
    }
    Ok(PredictionResponse {
        generator_revision: GENERATOR_REVISION.to_owned(),
        large_biomes: request.large_biomes,
        cells,
    })
}

#[cfg(test)]
#[path = "generator_tests.rs"]
mod tests;
