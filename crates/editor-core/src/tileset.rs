//! Tileset types for the Bevy 2D Editor level design tools.
//!
//! Tilesets are reusable grid-based image assets that provide tile graphics for
//! tile-based level design. A Tileset can be used across multiple TileLayers
//! and LevelSceneAssets.
//!
//! ## Architecture
//!
//! - `TilesetId`: Opaque stable identifier for a tileset asset
//! - `TilesetMetadata`: Catalog entry with image reference, tile dimensions, etc.
//! - `TilesetAsset`: The full tileset body (metadata + future tile data)
//! - `TileCoord`: Self-documenting coordinate struct (NOT a bare tuple)
//! - `TileRef`: Reference to a specific tile in a tileset
//! - `TileGrid`: Sparse HashMap grid of TileCoord → TileRef
//! - `AsepriteMetadata`: Parsed Aseprite JSON export data
//!
//! ## JSON Serialization Notes
//!
//! `TileCoord` uses a struct (not tuple) so JSON is `{"x":-1,"y":5}` instead
//! of `[-1, 5]` — self-documenting and unambiguous.
//!
//! `TileGrid` uses `HashMap` for O(1) runtime lookups. For deterministic
//! serialization (tests), convert to `BTreeMap` before comparing JSON.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// TilesetId — opaque String wrapper
// ─────────────────────────────────────────────────────────────────────────────

/// Opaque stable identifier for a Tileset.
/// Transparent so it serializes as a plain string, e.g. `"tileset_01abc..."`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TilesetId(pub String);

impl TilesetId {
    pub fn new(id: impl Into<String>) -> Self {
        TilesetId(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TileCoord — self-documenting struct (NOT bare tuple)
// ─────────────────────────────────────────────────────────────────────────────

/// A coordinate in the tile grid.
///
/// Uses a dedicated struct (not bare tuple) so JSON serializes as `{"x":-1,"y":5}`
/// which is self-documenting and unambiguous, rather than `[-1, 5]` which could
/// be confused with a generic array.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

impl TileCoord {
    pub fn new(x: i32, y: i32) -> Self {
        TileCoord { x, y }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TileRef — reference to a specific tile in a tileset
// ─────────────────────────────────────────────────────────────────────────────

/// Reference to a specific tile in a tileset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileRef {
    /// AssetReference to the Tileset this tile belongs to.
    pub tileset_id: String,
    /// Index into the tileset grid (row-major, 0-based).
    pub local_index: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// AsepriteMetadata — parsed from Aseprite JSON export
// ─────────────────────────────────────────────────────────────────────────────

/// Metadata parsed from Aseprite JSON export format.
/// Aseprite tags, slices, and frame durations are preserved for animation
/// and metadata purposes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsepriteMetadata {
    /// All frames in the spritesheet.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<AsepriteFrame>,
    /// Animation tags (walk, idle, attack, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<AsepriteTag>,
    /// Named slices (collision boxes, hit areas, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slices: Vec<AsepriteSlice>,
}

/// One frame in an Aseprite spritesheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsepriteFrame {
    /// Frame filename, e.g. "player_idle_0.png".
    pub name: String,
    /// Frame duration in milliseconds.
    pub duration: u32,
    /// Frame width in pixels.
    pub w: u32,
    /// Frame height in pixels.
    pub h: u32,
}

/// An animation tag defining a range of frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsepriteTag {
    /// Tag name, e.g. "walk", "idle", "attack".
    pub name: String,
    /// Starting frame index (inclusive, 0-based).
    pub from: u32,
    /// Ending frame index (inclusive, 0-based).
    pub to: u32,
}

/// A named slice defining a region in the spritesheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsepriteSlice {
    /// Slice name, e.g. "collision_box", "hit_area".
    pub name: String,
    /// X offset from spritesheet origin.
    pub x: i32,
    /// Y offset from spritesheet origin.
    pub y: i32,
    /// Slice width in pixels.
    pub w: u32,
    /// Slice height in pixels.
    pub h: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// TilesetMetadata — the catalog entry
// ─────────────────────────────────────────────────────────────────────────────

/// The catalog entry for a Tileset, stored in the tileset catalog index.
/// Contains everything needed to render the tileset and resolve tile references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetMetadata {
    /// Stable identifier for this tileset.
    pub id: TilesetId,
    /// Human-readable name, e.g. "Grass Tileset 16x16".
    pub name: String,
    /// AssetReference to the tileset image (e.g. "assets/tilesets/grass.png").
    pub image_ref: String,
    /// Tile width in pixels.
    pub tile_width: u32,
    /// Tile height in pixels.
    pub tile_height: u32,
    /// Number of columns in the tileset grid.
    pub columns: u32,
    /// Spacing between tiles in pixels.
    pub spacing: u32,
    /// Aseprite export metadata, if this tileset was exported from Aseprite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aseprite: Option<AsepriteMetadata>,
    /// ISO-8601 timestamp when this tileset was created.
    pub created_at: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// TilesetAsset — the body (stored in OPFS)
// ─────────────────────────────────────────────────────────────────────────────

/// The full tileset asset body stored in OPFS.
/// Mirrors the SceneAssetDocument pattern: metadata catalog entry + body file.
/// Tileset body doesn't store actual tile data — tiles live in TileLayers.
/// This allows a Tileset to be reused across multiple levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetAsset {
    /// The tileset metadata (catalog entry data).
    pub metadata: TilesetMetadata,
    /// Tileset body doesn't store tiles — tiles live in TileLayers.
    /// This field is present for future extension (e.g. tile collision data).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tile_data: Vec<u8>,
}

// ─────────────────────────────────────────────────────────────────────────────
// TileGrid — sparse grid of tiles
// ─────────────────────────────────────────────────────────────────────────────

/// Sparse grid mapping tile coordinates to tile references.
/// Only explicitly painted tiles are stored — empty cells use no memory.
/// Uses HashMap for O(1) runtime lookups.
///
/// For deterministic serialization in tests, convert to BTreeMap
/// (which has sorted keys) before comparing JSON bytes.
pub type TileGrid = HashMap<TileCoord, TileRef>;

// ─────────────────────────────────────────────────────────────────────────────
// TilesetManager — manages all tilesets in a project
// ─────────────────────────────────────────────────────────────────────────────

/// Manages all tilesets in a project, similar to SceneAssetCatalog.
/// Provides registration, lookup, and listing of tileset assets.
#[derive(Debug, Clone, Default)]
pub struct TilesetManager {
    entries: HashMap<TilesetId, TilesetMetadata>,
}

impl TilesetManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tileset in the manager.
    pub fn register(&mut self, metadata: TilesetMetadata) -> Option<TilesetMetadata> {
        self.entries.insert(metadata.id.clone(), metadata)
    }

    /// Unregister a tileset by ID. Returns the removed entry if it existed.
    pub fn unregister(&mut self, id: &TilesetId) -> Option<TilesetMetadata> {
        self.entries.remove(id)
    }

    /// Get tileset metadata by ID.
    pub fn get(&self, id: &TilesetId) -> Option<&TilesetMetadata> {
        self.entries.get(id)
    }

    /// List all tilesets.
    pub fn list_all(&self) -> Vec<&TilesetMetadata> {
        self.entries.values().collect()
    }

    /// Number of tilesets currently registered.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if no tilesets are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // TileCoord tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_tile_coord_serialization_roundtrip() {
        // TileCoord uses a struct so JSON is self-documenting: {"x":-1,"y":5}
        // NOT an ambiguous array: [-1, 5]
        let coord = TileCoord::new(-1, 5);
        let json = serde_json::to_string(&coord).unwrap();
        assert_eq!(json, r#"{"x":-1,"y":5}"#);

        let roundtrip: TileCoord = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip, coord);
    }

    #[test]
    fn test_tile_coord_equality() {
        let a = TileCoord::new(0, 0);
        let b = TileCoord::new(0, 0);
        let c = TileCoord::new(1, 0);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_tile_coord_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(TileCoord::new(0, 0));
        set.insert(TileCoord::new(1, 2));
        assert_eq!(set.len(), 2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // TileRef tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_tile_ref_serialization_roundtrip() {
        let tile_ref = TileRef {
            tileset_id: "tileset_grass_16".to_string(),
            local_index: 42,
        };
        let json = serde_json::to_string(&tile_ref).unwrap();
        let roundtrip: TileRef = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip, tile_ref);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // TilesetMetadata tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_tileset_metadata_serialization_roundtrip() {
        let metadata = TilesetMetadata {
            id: TilesetId::new("tileset_grass_16".to_string()),
            name: "Grass Tileset 16x16".to_string(),
            image_ref: "assets/tilesets/grass.png".to_string(),
            tile_width: 16,
            tile_height: 16,
            columns: 16,
            spacing: 0,
            aseprite: None,
            created_at: "2024-01-15T10:30:00Z".to_string(),
        };
        let json = serde_json::to_string(&metadata).unwrap();
        let roundtrip: TilesetMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.id, metadata.id);
        assert_eq!(roundtrip.name, metadata.name);
        assert_eq!(roundtrip.tile_width, metadata.tile_width);
    }

    #[test]
    fn test_tileset_metadata_with_aseprite() {
        let metadata = TilesetMetadata {
            id: TilesetId::new("tileset_player".to_string()),
            name: "Player Tileset".to_string(),
            image_ref: "assets/tilesets/player.png".to_string(),
            tile_width: 32,
            tile_height: 32,
            columns: 8,
            spacing: 1,
            aseprite: Some(AsepriteMetadata {
                frames: vec![
                    AsepriteFrame {
                        name: "player_idle_0.png".to_string(),
                        duration: 100,
                        w: 32,
                        h: 32,
                    },
                    AsepriteFrame {
                        name: "player_idle_1.png".to_string(),
                        duration: 100,
                        w: 32,
                        h: 32,
                    },
                ],
                tags: vec![AsepriteTag {
                    name: "idle".to_string(),
                    from: 0,
                    to: 1,
                }],
                slices: vec![],
            }),
            created_at: "2024-01-15T10:30:00Z".to_string(),
        };
        let json = serde_json::to_string(&metadata).unwrap();
        let roundtrip: TilesetMetadata = serde_json::from_str(&json).unwrap();
        assert!(roundtrip.aseprite.is_some());
        let aseprite = roundtrip.aseprite.unwrap();
        assert_eq!(aseprite.frames.len(), 2);
        assert_eq!(aseprite.tags[0].name, "idle");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // TilesetManager tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_tileset_manager_register_get() {
        let mut manager = TilesetManager::new();
        let metadata = TilesetMetadata {
            id: TilesetId::new("ts_01".to_string()),
            name: "Test Tileset".to_string(),
            image_ref: "assets/test.png".to_string(),
            tile_width: 16,
            tile_height: 16,
            columns: 8,
            spacing: 0,
            aseprite: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        manager.register(metadata.clone());
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.get(&TilesetId::new("ts_01".to_string())).unwrap().name, "Test Tileset");
    }

    #[test]
    fn test_tileset_manager_unregister() {
        let mut manager = TilesetManager::new();
        let metadata = TilesetMetadata {
            id: TilesetId::new("ts_02".to_string()),
            name: "Test Tileset 2".to_string(),
            image_ref: "assets/test2.png".to_string(),
            tile_width: 16,
            tile_height: 16,
            columns: 8,
            spacing: 0,
            aseprite: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        manager.register(metadata.clone());
        let removed = manager.unregister(&TilesetId::new("ts_02".to_string()));
        assert!(removed.is_some());
        assert!(manager.is_empty());
    }

    #[test]
    fn test_tileset_manager_list_all() {
        let mut manager = TilesetManager::new();
        for i in 0..3 {
            let metadata = TilesetMetadata {
                id: TilesetId::new(format!("ts_{}", i)),
                name: format!("Tileset {}", i),
                image_ref: format!("assets/tileset_{}.png", i),
                tile_width: 16,
                tile_height: 16,
                columns: 8,
                spacing: 0,
                aseprite: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
            };
            manager.register(metadata);
        }
        let all = manager.list_all();
        assert_eq!(all.len(), 3);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // TileGrid sparse grid tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_sparse_grid_many_empty_cells() {
        // Sparse grid: only stored tiles use memory
        let mut grid = TileGrid::new();

        // Only 3 tiles in a 100x100 grid
        grid.insert(TileCoord::new(0, 0), TileRef {
            tileset_id: "ts_01".to_string(),
            local_index: 0,
        });
        grid.insert(TileCoord::new(50, 50), TileRef {
            tileset_id: "ts_01".to_string(),
            local_index: 1,
        });
        grid.insert(TileCoord::new(99, 99), TileRef {
            tileset_id: "ts_01".to_string(),
            local_index: 2,
        });

        assert_eq!(grid.len(), 3);

        // All other cells are empty (no memory used)
        assert!(grid.get(&TileCoord::new(1, 0)).is_none());
        assert!(grid.get(&TileCoord::new(0, 1)).is_none());
        assert!(grid.get(&TileCoord::new(25, 25)).is_none());
    }

    #[test]
    fn test_sparse_grid_insert_get_remove() {
        let mut grid = TileGrid::new();
        let coord = TileCoord::new(5, 10);
        let tile_ref = TileRef {
            tileset_id: "ts_test".to_string(),
            local_index: 7,
        };

        // Insert
        grid.insert(coord.clone(), tile_ref.clone());
        assert_eq!(grid.get(&coord), Some(&tile_ref));

        // Update
        let new_ref = TileRef {
            tileset_id: "ts_test".to_string(),
            local_index: 99,
        };
        grid.insert(coord.clone(), new_ref.clone());
        assert_eq!(grid.get(&coord), Some(&new_ref));

        // Remove
        let removed = grid.remove(&coord);
        assert_eq!(removed, Some(new_ref));
        assert!(grid.get(&coord).is_none());
    }
}
