//! TileLayer — a layer inside a LevelSceneAsset that stores a grid of tiles.
//!
//! A TileLayer belongs to exactly one LevelSceneAsset and owns its sparse grid
//! of tiles. The layer references a Tileset for tile graphics.

use super::tileset::{TileCoord, TileRef, TilesetId};
use crate::scene_asset::LayerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// TileLayerId — alias for LayerId (LAYER_ID_UNIFICATION full)
// ─────────────────────────────────────────────────────────────────────────────

/// Opaque stable identifier for a TileLayer inside a LevelSceneAsset.
///
/// LAYER_ID_UNIFICATION full: `TileLayerId` is now a type alias for `LayerId`.
/// All three layer-id types (`LayerId`, `TileLayerId`, `AutoLayerId`) wrap the
/// same `String` and serialize identically (`#[serde(transparent)]`). Keeping
/// the type alias preserves source compatibility with all call sites that name
/// `TileLayerId` explicitly (auto_layer rules, paint/erase commands, tests).
pub type TileLayerId = LayerId;

// ─────────────────────────────────────────────────────────────────────────────
// TileLayer
// ─────────────────────────────────────────────────────────────────────────────

/// A layer inside a LevelSceneAsset that stores a sparse grid of tiles.
///
/// A TileLayer is identified by a stable `TileLayerId` and references a
/// `TilesetId` for tile graphics. The `grid` field is a sparse HashMap:
/// only explicitly painted tiles consume memory.
///
/// # Layer Order
///
/// The `order` field controls rendering order (lower = rendered first).
/// This allows foreground/background layering in the viewport.
///
/// # Generation Counter
///
/// The `generation` field increments every time the layer is mutated
/// (`paint_tile` or `erase_tile`). AutoLayers track this counter to detect
/// when their cached output is stale and needs regeneration.
///
/// # Example
///
/// ```
/// use editor_core::tileset::{TileCoord, TileRef, TilesetId};
/// use editor_core::tile_layer::{TileLayer, TileLayerId};
///
/// let tileset_id = TilesetId::new("tileset_grass_16".to_string());
/// let mut layer = TileLayer::new(
///     TileLayerId::new("layer_ground".to_string()),
///     "Ground".to_string(),
///     tileset_id,
/// );
///
/// // Paint a tile at (5, 10)
/// layer.paint_tile(
///     TileCoord::new(5, 10),
///     TileRef { tileset_id: "tileset_grass_16".to_string(), local_index: 0 },
/// );
///
/// // Check it exists
/// assert!(layer.get_tile(&TileCoord::new(5, 10)).is_some());
///
/// // Erase it
/// let erased = layer.erase_tile(&TileCoord::new(5, 10));
/// assert!(erased.is_some());
/// assert!(layer.get_tile(&TileCoord::new(5, 10)).is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileLayer {
    /// Stable identifier for this layer.
    pub id: TileLayerId,
    /// Human-readable name shown in the layer list.
    pub name: String,
    /// The tileset this layer paints from.
    pub tileset_id: TilesetId,
    /// Sparse grid: only painted tiles are stored.
    /// HashMap gives O(1) lookup/insert/delete.
    #[serde(default)]
    pub grid: TileGrid,
    /// Layer order for rendering (lower = rendered first).
    /// Multiple layers can share the same order for grouped rendering.
    pub order: i32,
    /// Generation counter. Incremented on every `paint_tile` or `erase_tile`.
    /// Used by AutoLayers to detect staleness via `source_generation`.
    #[serde(default)]
    pub generation: u64,
    /// Grid width in tiles (horizontal extent). `#[serde(default)]` so old
    /// serialized layers without this field still parse (default: 50, the
    /// historical UI constant). New layers set this explicitly at creation.
    #[serde(default = "default_grid_width")]
    pub grid_width: u32,
    /// Grid height in tiles (vertical extent). `#[serde(default)]` so old
    /// serialized layers without this field still parse (default: 50).
    #[serde(default = "default_grid_height")]
    pub grid_height: u32,
}

/// Default grid width for layers serialized before `grid_width` was added.
fn default_grid_width() -> u32 {
    50
}

/// Default grid height for layers serialized before `grid_height` was added.
fn default_grid_height() -> u32 {
    50
}

/// Re-export TileGrid from tileset module for convenience.
pub use super::tileset::TileGrid as TileGrid;

impl TileLayer {
    /// Create a new TileLayer with an empty grid, default order (0), and generation (0).
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

    /// Create a new TileLayer with explicit grid dimensions.
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

    /// Paint a tile at the given coordinate, overwriting any existing tile.
    ///
    /// This is an upsert operation — if a tile already exists at `coord`,
    /// it is replaced with the new `tile_ref`. Bumps `generation` to mark
    /// the layer as modified (used by AutoLayers for staleness detection).
    pub fn paint_tile(&mut self, coord: TileCoord, tile_ref: TileRef) {
        self.grid.insert(coord, tile_ref);
        self.generation += 1;
    }

    /// Erase a tile at the given coordinate.
    ///
    /// Returns the erased `TileRef` if a tile existed at that coordinate,
    /// or `None` if the coordinate was already empty. Bumps `generation` to
    /// mark the layer as modified (used by AutoLayers for staleness detection).
    pub fn erase_tile(&mut self, coord: &TileCoord) -> Option<TileRef> {
        let result = self.grid.remove(coord);
        if result.is_some() {
            self.generation += 1;
        }
        result
    }

    /// Get a tile reference at the given coordinate.
    ///
    /// Returns `Some(&TileRef)` if a tile exists at that coordinate,
    /// or `None` if the coordinate is empty.
    pub fn get_tile(&self, coord: &TileCoord) -> Option<&TileRef> {
        self.grid.get(coord)
    }

    /// Number of painted tiles in this layer.
    ///
    /// This is NOT the grid size — it's the count of non-empty cells.
    pub fn tile_count(&self) -> usize {
        self.grid.len()
    }

    /// True if no tiles are painted (all cells empty).
    pub fn is_empty(&self) -> bool {
        self.grid.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // TileLayer paint/erase/get tests
    // ─────────────────────────────────────────────────────────────────────────

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

        // Initially empty
        assert!(layer.erase_tile(&coord).is_none());

        // Paint then erase
        layer.paint_tile(coord.clone(), tile_ref.clone());
        let erased = layer.erase_tile(&coord);
        assert_eq!(erased, Some(tile_ref));
        assert!(layer.get_tile(&coord).is_none());
    }

    #[test]
    fn test_tile_layer_overwrite_tile() {
        let tileset_id = TilesetId::new("ts_grass".to_string());
        let mut layer = TileLayer::new(
            TileLayerId::new("layer_1".to_string()),
            "Grass Layer".to_string(),
            tileset_id.clone(),
        );

        let coord = TileCoord::new(0, 0);

        // First tile
        let tile_a = TileRef {
            tileset_id: "ts_grass".to_string(),
            local_index: 0,
        };
        layer.paint_tile(coord.clone(), tile_a);

        // Overwrite with second tile
        let tile_b = TileRef {
            tileset_id: "ts_grass".to_string(),
            local_index: 1,
        };
        layer.paint_tile(coord.clone(), tile_b.clone());

        // Get returns second tile
        assert_eq!(layer.get_tile(&coord), Some(&tile_b));

        // Erase returns second tile (not first)
        assert_eq!(layer.erase_tile(&coord), Some(tile_b));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // TileLayer sparse grid tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_sparse_grid_many_empty_cells() {
        let tileset_id = TilesetId::new("ts_grass".to_string());
        let mut layer = TileLayer::new(
            TileLayerId::new("layer_1".to_string()),
            "Test Layer".to_string(),
            tileset_id,
        );

        // Paint only 3 tiles in a large grid
        layer.paint_tile(
            TileCoord::new(0, 0),
            TileRef { tileset_id: "ts_grass".to_string(), local_index: 0 },
        );
        layer.paint_tile(
            TileCoord::new(100, 200),
            TileRef { tileset_id: "ts_grass".to_string(), local_index: 1 },
        );
        layer.paint_tile(
            TileCoord::new(-50, -30),
            TileRef { tileset_id: "ts_grass".to_string(), local_index: 2 },
        );

        // Layer reports 3 tiles
        assert_eq!(layer.tile_count(), 3);
        assert!(!layer.is_empty());

        // All other coordinates are empty
        assert!(layer.get_tile(&TileCoord::new(1, 0)).is_none());
        assert!(layer.get_tile(&TileCoord::new(50, 50)).is_none());
        assert!(layer.get_tile(&TileCoord::new(100, 199)).is_none());
        assert!(layer.get_tile(&TileCoord::new(-49, -30)).is_none());
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

    // ─────────────────────────────────────────────────────────────────────────
    // TileLayerId tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_tile_layer_id() {
        let id = TileLayerId::new("layer_abc123".to_string());
        assert_eq!(id.as_str(), "layer_abc123");
    }

    #[test]
    fn test_tile_layer_id_serialization() {
        let id = TileLayerId::new("layer_test".to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""layer_test""#);
        let roundtrip: TileLayerId = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.as_str(), "layer_test");
    }

    #[test]
    fn test_layerid_full_unification_preserves_api() {
        // LAYER_ID_UNIFICATION full: TileLayerId is now a type alias of LayerId.
        // These tests lock down the API surface that every call site relies on:
        // - construction via ::new(s), From<&str>, From<String>
        // - .as_str() accessor
        // - serde::transparent representation (plain string)
        // - equality + hashing across the alias boundary
        use crate::scene_asset::LayerId;
        use std::collections::HashSet;

        // All three constructor styles must work on the alias.
        let a = TileLayerId::new("lyr_01");
        let b: TileLayerId = LayerId::from("lyr_01");
        let c: TileLayerId = String::from("lyr_01").into();
        let d: TileLayerId = "lyr_01".into();

        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(c, d);

        // Accessor preserved.
        assert_eq!(a.as_str(), "lyr_01");

        // Serde representation is a plain string (no discriminant added).
        let json = serde_json::to_string(&a).unwrap();
        assert_eq!(json, r#""lyr_01""#);
        let back: TileLayerId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, a);

        // Hashable — usable in collections (regression for HashMap<LayerId>).
        let mut set: HashSet<TileLayerId> = HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&b));

        // Type alias means TileLayerId == LayerId at the type level.
        // (Both deserialize from the same JSON shape; no manual conversion needed.)
        let from_layer: LayerId = serde_json::from_str(r#""lyr_01""#).unwrap();
        assert_eq!(from_layer, a);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // TileLayer serialization tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_tile_layer_serialization_roundtrip() {
        let tileset_id = TilesetId::new("ts_grass".to_string());
        let mut layer = TileLayer::new(
            TileLayerId::new("layer_grass".to_string()),
            "Grass Layer".to_string(),
            tileset_id,
        );
        layer.order = 1;
        layer.paint_tile(
            TileCoord::new(5, 10),
            TileRef { tileset_id: "ts_grass".to_string(), local_index: 7 },
        );

        let json = serde_json::to_string(&layer).unwrap();
        let roundtrip: TileLayer = serde_json::from_str(&json).unwrap();

        assert_eq!(roundtrip.id.as_str(), "layer_grass");
        assert_eq!(roundtrip.name, "Grass Layer");
        assert_eq!(roundtrip.tileset_id.as_str(), "ts_grass");
        assert_eq!(roundtrip.order, 1);
        assert_eq!(roundtrip.tile_count(), 1);
    }

    #[test]
    fn test_tile_layer_empty_grid_deserializes() {
        // Deserialize a TileLayer with no grid field (old save format)
        // Should default to empty HashMap
        let json = r#"{
            "id": "layer_01",
            "name": "Test Layer",
            "tileset_id": "ts_01",
            "order": 0
        }"#;
        let layer: TileLayer = serde_json::from_str(json).unwrap();
        assert!(layer.is_empty());
        assert_eq!(layer.tile_count(), 0);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // TileLayer generation counter tests
    // ─────────────────────────────────────────────────────────────────────────

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
            TileRef { tileset_id: "ts_test".to_string(), local_index: 0 },
        );
        assert_eq!(layer.generation, 1);

        // Paint again at same coord — still bumps
        layer.paint_tile(
            TileCoord::new(0, 0),
            TileRef { tileset_id: "ts_test".to_string(), local_index: 1 },
        );
        assert_eq!(layer.generation, 2);
    }

    #[test]
    fn test_tile_layer_generation_bumped_on_erase() {
        let tileset_id = TilesetId::new("ts_test".to_string());
        let mut layer = TileLayer::new(
            TileLayerId::new("layer_gen".to_string()),
            "Gen Test".to_string(),
            tileset_id,
        );
        layer.paint_tile(
            TileCoord::new(5, 5),
            TileRef { tileset_id: "ts_test".to_string(), local_index: 0 },
        );
        assert_eq!(layer.generation, 1);

        // Erase existing tile — bumps
        let erased = layer.erase_tile(&TileCoord::new(5, 5));
        assert!(erased.is_some());
        assert_eq!(layer.generation, 2);

        // Erase non-existent — no bump
        let erased_none = layer.erase_tile(&TileCoord::new(5, 5));
        assert!(erased_none.is_none());
        assert_eq!(layer.generation, 2);
    }

    #[test]
    fn test_tile_layer_generation_deserializes_default() {
        // Deserialize a TileLayer with no generation field (old save format)
        // Should default to 0
        let json = r#"{
            "id": "layer_01",
            "name": "Test Layer",
            "tileset_id": "ts_01",
            "order": 0
        }"#;
        let layer: TileLayer = serde_json::from_str(json).unwrap();
        assert_eq!(layer.generation, 0);
    }

    #[test]
    fn test_tile_layer_generation_deserializes_from_json() {
        // Verify that a TileLayer with explicit generation field deserializes correctly.
        // Note: serde_json cannot round-trip HashMap<TileCoord, _> through JSON
        // (JSON requires string keys), so we test only the deserialization path.
        let json = r#"{
            "id": "layer_gen",
            "name": "Gen Roundtrip",
            "tileset_id": "ts_test",
            "order": 0,
            "generation": 5,
            "grid": {}
        }"#;
        let layer: TileLayer = serde_json::from_str(json).unwrap();
        assert_eq!(layer.generation, 5);
        assert_eq!(layer.name, "Gen Roundtrip");
        assert!(layer.is_empty());
    }
}
