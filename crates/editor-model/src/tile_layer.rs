//! TileLayer — a layer inside a LevelSceneAsset that stores a grid of tiles.

use crate::ids::LayerId;
use crate::tileset::{TileCoord, TileGrid, TileRef, TilesetId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Opaque stable identifier for a TileLayer inside a LevelSceneAsset.
///
/// LAYER_ID_UNIFICATION full: `TileLayerId` is now a type alias for `LayerId`.
pub type TileLayerId = LayerId;

/// A layer inside a LevelSceneAsset that stores a sparse grid of tiles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileLayer {
    /// Unique identifier for this layer.
    pub id: TileLayerId,
    /// Human-readable name.
    pub name: String,
    /// Tileset providing tile graphics for this layer.
    pub tileset_id: TilesetId,
    /// Sparse map of painted tile coordinates to tile references.
    #[serde(default)]
    pub grid: TileGrid,
    /// Z-ordering index.
    pub order: i32,
    /// Incremented each time the grid is modified.
    #[serde(default)]
    pub generation: u64,
    /// Width of the grid in tiles.
    #[serde(default = "default_grid_width")]
    pub grid_width: u32,
    /// Height of the grid in tiles.
    #[serde(default = "default_grid_height")]
    pub grid_height: u32,
}

fn default_grid_width() -> u32 {
    50
}

fn default_grid_height() -> u32 {
    50
}

impl TileLayer {
    /// Construct a new TileLayer with default grid dimensions (50×50).
    pub fn new(id: TileLayerId, name: String, tileset_id: TilesetId) -> Self {
        TileLayer {
            id,
            name,
            tileset_id,
            grid: HashMap::new(),
            order: 0,
            generation: 0,
            grid_width: default_grid_width(),
            grid_height: default_grid_height(),
        }
    }

    /// Create a new layer with explicit grid dimensions.
    pub fn with_dimensions(
        id: TileLayerId,
        name: String,
        tileset_id: TilesetId,
        grid_width: u32,
        grid_height: u32,
    ) -> Self {
        TileLayer {
            id,
            name,
            tileset_id,
            grid: HashMap::new(),
            order: 0,
            generation: 0,
            grid_width,
            grid_height,
        }
    }

    /// Paint a tile at `coord`, replacing any existing tile. Increments generation.
    pub fn paint_tile(&mut self, coord: TileCoord, tile_ref: TileRef) {
        self.grid.insert(coord, tile_ref);
        self.generation += 1;
    }

    /// Erase the tile at `coord`. Returns the erased tile reference. Increments generation.
    pub fn erase_tile(&mut self, coord: &TileCoord) -> Option<TileRef> {
        let result = self.grid.remove(coord);
        if result.is_some() {
            self.generation += 1;
        }
        result
    }

    /// Look up the tile at `coord`.
    pub fn get_tile(&self, coord: &TileCoord) -> Option<&TileRef> {
        self.grid.get(coord)
    }

    /// Total number of painted tiles.
    pub fn tile_count(&self) -> usize {
        self.grid.len()
    }

    /// True when no tiles are painted.
    pub fn is_empty(&self) -> bool {
        self.grid.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_layer_paint_and_get() {
        let tileset_id = TilesetId::new("ts_grass".to_string());
        let mut layer = TileLayer::new(
            TileLayerId::new("layer_1".to_string()),
            "Grass Layer".to_string(),
            tileset_id.clone(),
        );

        let coord = TileCoord::new(3, 7);
        let tile_ref = TileRef {
            tileset_id: "ts_grass".to_string(),
            local_index: 12,
        };

        layer.paint_tile(coord.clone(), tile_ref.clone());
        assert_eq!(layer.get_tile(&coord), Some(&tile_ref));
    }

    #[test]
    fn test_tile_layer_erase() {
        let tileset_id = TilesetId::new("ts_grass".to_string());
        let mut layer = TileLayer::new(
            TileLayerId::new("layer_1".to_string()),
            "Grass Layer".to_string(),
            tileset_id.clone(),
        );

        let coord = TileCoord::new(3, 7);
        let tile_ref = TileRef {
            tileset_id: "ts_grass".to_string(),
            local_index: 12,
        };

        assert!(layer.erase_tile(&coord).is_none());
        layer.paint_tile(coord.clone(), tile_ref.clone());
        let erased = layer.erase_tile(&coord);
        assert_eq!(erased, Some(tile_ref));
        assert!(layer.get_tile(&coord).is_none());
    }

    #[test]
    fn test_sparse_grid_many_empty_cells() {
        let tileset_id = TilesetId::new("ts_grass".to_string());
        let mut layer = TileLayer::new(
            TileLayerId::new("layer_1".to_string()),
            "Test Layer".to_string(),
            tileset_id,
        );

        layer.paint_tile(
            TileCoord::new(0, 0),
            TileRef {
                tileset_id: "ts_grass".to_string(),
                local_index: 0,
            },
        );
        layer.paint_tile(
            TileCoord::new(100, 200),
            TileRef {
                tileset_id: "ts_grass".to_string(),
                local_index: 1,
            },
        );

        assert_eq!(layer.tile_count(), 2);
        assert!(!layer.is_empty());
        assert!(layer.get_tile(&TileCoord::new(1, 0)).is_none());
    }

    #[test]
    fn test_tile_layer_is_empty() {
        let tileset_id = TilesetId::new("ts_grass".to_string());
        let layer = TileLayer::new(
            TileLayerId::new("layer_1".to_string()),
            "Empty Layer".to_string(),
            tileset_id,
        );

        assert!(layer.is_empty());
        assert_eq!(layer.tile_count(), 0);
    }

    #[test]
    fn test_tile_layer_generation_starts_at_zero() {
        let tileset_id = TilesetId::new("ts_test".to_string());
        let layer = TileLayer::new(
            TileLayerId::new("layer_gen".to_string()),
            "Gen Test".to_string(),
            tileset_id,
        );
        assert_eq!(layer.generation, 0);
    }

    #[test]
    fn test_tile_layer_generation_bumped_on_paint() {
        let tileset_id = TilesetId::new("ts_test".to_string());
        let mut layer = TileLayer::new(
            TileLayerId::new("layer_gen".to_string()),
            "Gen Test".to_string(),
            tileset_id,
        );
        assert_eq!(layer.generation, 0);

        layer.paint_tile(
            TileCoord::new(0, 0),
            TileRef {
                tileset_id: "ts_test".to_string(),
                local_index: 0,
            },
        );
        assert_eq!(layer.generation, 1);

        layer.paint_tile(
            TileCoord::new(0, 0),
            TileRef {
                tileset_id: "ts_test".to_string(),
                local_index: 1,
            },
        );
        assert_eq!(layer.generation, 2);
    }
}
