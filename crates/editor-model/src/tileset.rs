//! Tileset types for the Bevy 2D Editor level design tools.
//!
//! Tilesets are reusable grid-based image assets that provide tile graphics for
//! tile-based level design.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Opaque stable identifier for a Tileset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TilesetId(pub String);

impl TilesetId {
    /// Construct a new TilesetId from a string.
    pub fn new(id: impl Into<String>) -> Self {
        TilesetId(id.into())
    }

    /// Borrow the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A coordinate in the tile grid.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TileCoord {
    /// X coordinate (column index).
    pub x: i32,
    /// Y coordinate (row index).
    pub y: i32,
}

impl serde::Serialize for TileCoord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{},{}", self.x, self.y))
    }
}

impl<'de> serde::Deserialize<'de> for TileCoord {
    fn deserialize<D>(deserializer: D) -> Result<TileCoord, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::custom(format!(
                "invalid TileCoord string: expected 'x,y', got '{}'",
                s
            )));
        }
        let x = parts[0].parse().map_err(|_| {
            serde::de::Error::custom(format!("invalid x in TileCoord: '{}'", parts[0]))
        })?;
        let y = parts[1].parse().map_err(|_| {
            serde::de::Error::custom(format!("invalid y in TileCoord: '{}'", parts[1]))
        })?;
        Ok(TileCoord { x, y })
    }
}

impl TileCoord {
    /// Construct a new tile coordinate.
    pub fn new(x: i32, y: i32) -> Self {
        TileCoord { x, y }
    }
}

/// Reference to a specific tile in a tileset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileRef {
    /// ID of the tileset this tile belongs to.
    pub tileset_id: String,
    /// 0-based index of the tile within the tileset grid.
    pub local_index: u32,
}

/// Metadata parsed from Aseprite JSON export format.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsepriteMetadata {
    /// Animation frames from the Aseprite file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<AsepriteFrame>,
    /// Animation tags (e.g. "idle", "walk") from the Aseprite file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<AsepriteTag>,
    /// Named slices defining regions of interest.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slices: Vec<AsepriteSlice>,
}

/// One frame in an Aseprite animation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsepriteFrame {
    /// Frame name as exported by Aseprite.
    pub name: String,
    /// Frame duration in milliseconds.
    pub duration: u32,
    /// Frame width in pixels.
    pub w: u32,
    /// Frame height in pixels.
    pub h: u32,
}

/// A named animation tag defined in Aseprite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsepriteTag {
    /// Display name of the tag.
    pub name: String,
    /// First frame index (inclusive, 0-based).
    pub from: u32,
    /// Last frame index (inclusive, 0-based).
    pub to: u32,
}

/// A named region of interest defined in Aseprite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsepriteSlice {
    /// Display name of the slice.
    pub name: String,
    /// X coordinate of the slice origin in the sprite.
    pub x: i32,
    /// Y coordinate of the slice origin in the sprite.
    pub y: i32,
    /// Width of the slice in pixels.
    pub w: u32,
    /// Height of the slice in pixels.
    pub h: u32,
}

/// The catalog entry for a Tileset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetMetadata {
    /// Stable identifier for this tileset.
    pub id: TilesetId,
    /// Human-readable name.
    pub name: String,
    /// Logical path to the source image asset.
    pub image_ref: String,
    /// Width of each tile in pixels.
    pub tile_width: u32,
    /// Height of each tile in pixels.
    pub tile_height: u32,
    /// Number of columns in the source image grid.
    pub columns: u32,
    /// Spacing between tiles in pixels.
    pub spacing: u32,
    /// Parsed Aseprite animation metadata, if exported from Aseprite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aseprite: Option<AsepriteMetadata>,
    /// ISO-8601 creation timestamp.
    pub created_at: String,
}

/// The full tileset asset body stored in OPFS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetAsset {
    /// Tileset catalog metadata.
    pub metadata: TilesetMetadata,
    /// Raw pixel data (optional, may be empty if data is image-only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tile_data: Vec<u8>,
}

/// Sparse grid mapping tile coordinates to tile references.
pub type TileGrid = HashMap<TileCoord, TileRef>;

/// Manages all tilesets in a project.
#[derive(Debug, Clone, Default)]
pub struct TilesetManager {
    entries: HashMap<TilesetId, TilesetMetadata>,
}

impl TilesetManager {
    /// Construct a new empty manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tileset. Returns the previous metadata if one existed for this ID.
    pub fn register(&mut self, metadata: TilesetMetadata) -> Option<TilesetMetadata> {
        self.entries.insert(metadata.id.clone(), metadata)
    }

    /// Unregister a tileset by ID. Returns the removed metadata if found.
    pub fn unregister(&mut self, id: &TilesetId) -> Option<TilesetMetadata> {
        self.entries.remove(id)
    }

    /// Look up a tileset by ID.
    pub fn get(&self, id: &TilesetId) -> Option<&TilesetMetadata> {
        self.entries.get(id)
    }

    /// Return all registered tileset metadata entries.
    pub fn list_all(&self) -> Vec<&TilesetMetadata> {
        self.entries.values().collect()
    }

    /// Return the number of registered tilesets.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if no tilesets are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_coord_serialization_roundtrip() {
        let coord = TileCoord::new(-1, 5);
        let json = serde_json::to_string(&coord).unwrap();
        assert_eq!(json, "\"-1,5\"");
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
    fn test_tile_ref_serialization_roundtrip() {
        let tile_ref = TileRef {
            tileset_id: "tileset_grass_16".to_string(),
            local_index: 42,
        };
        let json = serde_json::to_string(&tile_ref).unwrap();
        let roundtrip: TileRef = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip, tile_ref);
    }

    #[test]
    fn test_sparse_grid_many_empty_cells() {
        let mut grid = TileGrid::new();
        grid.insert(
            TileCoord::new(0, 0),
            TileRef {
                tileset_id: "ts_01".to_string(),
                local_index: 0,
            },
        );
        grid.insert(
            TileCoord::new(50, 50),
            TileRef {
                tileset_id: "ts_01".to_string(),
                local_index: 1,
            },
        );
        assert_eq!(grid.len(), 2);
        assert!(grid.get(&TileCoord::new(1, 0)).is_none());
    }
}
