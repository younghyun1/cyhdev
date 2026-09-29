//! Stable, zoom-dependent addressing for bounded biome tiles.

use super::map_query::identifier;

pub const SIDE: usize = 64;
pub const CELLS: usize = SIDE * SIDE;
pub const WORLD_EDGE: i64 = 30_000_000;

pub struct SeedTile {
    pub query: SeedTileQuery,
    pub min_x: i32,
    pub min_z: i32,
    pub step: u32,
    pub preset: super::prediction::PredictionPreset,
    pub profile_epoch: String,
    pub sampled_at_ms: i64,
    pub expires_at_ms: i64,
    pub palette: Vec<String>,
    pub indices: Vec<Option<u16>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SeedTileQuery {
    pub world: String,
    pub tile_x: i32,
    pub tile_z: i32,
    pub level: u8,
    pub y: i32,
}

impl SeedTileQuery {
    /// Keep coordinate arithmetic widened until the complete tile is validated.
    pub fn geometry(&self) -> Option<(i32, i32, u32)> {
        if self.level > 12 || !identifier(&self.world) || !(-64..=319).contains(&self.y) {
            return None;
        }
        let step = 4_u32 << self.level;
        let span = i64::from(step) * SIDE as i64;
        let x = i64::from(self.tile_x) * span;
        let z = i64::from(self.tile_z) * span;
        if [x, z]
            .into_iter()
            .any(|origin| origin >= WORLD_EDGE || origin + span <= -WORLD_EDGE)
        {
            return None;
        }
        Some((i32::try_from(x).ok()?, i32::try_from(z).ok()?, step))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_tiles_and_zoom_levels_have_stable_origins() {
        let mut query = SeedTileQuery {
            world: "minecraft:overworld".into(),
            tile_x: -1,
            tile_z: 1,
            level: 0,
            y: 64,
        };
        assert_eq!(query.geometry(), Some((-256, 256, 4)));
        query.level = 12;
        assert_eq!(query.geometry(), Some((-1_048_576, 1_048_576, 16_384)));
        query.level = 13;
        assert!(query.geometry().is_none());
        query.level = 0;
        query.tile_x = i32::MAX;
        assert!(query.geometry().is_none());
        query.tile_x = 117187;
        assert!(query.geometry().is_some());
    }
}
