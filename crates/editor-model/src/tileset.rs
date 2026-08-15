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
    pub fn new(id: impl Into<String>) -> Self {
        TilesetId(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A coordinate in the tile grid.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub x: i32,
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
    pub fn new(x: i32, y: i32) -> Self {
        TileCoord { x, y }
    }
}

/// Reference to a specific tile in a tileset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileRef {
    pub tileset_id: String,
    pub local_index: u32,
}

/// Metadata parsed from Aseprite JSON export format.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsepriteMetadata {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<AsepriteFrame>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<AsepriteTag>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slices: Vec<AsepriteSlice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsepriteFrame {
    pub name: String,
    pub duration: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsepriteTag {
    pub name: String,
    pub from: u32,
    pub to: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsepriteSlice {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// The catalog entry for a Tileset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetMetadata {
    pub id: TilesetId,
    pub name: String,
    pub image_ref: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u32,
    pub spacing: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aseprite: Option<AsepriteMetadata>,
    pub created_at: String,
}

/// The full tileset asset body stored in OPFS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetAsset {
    pub metadata: TilesetMetadata,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, metadata: TilesetMetadata) -> Option<TilesetMetadata> {
        self.entries.insert(metadata.id.clone(), metadata)
    }

    pub fn unregister(&mut self, id: &TilesetId) -> Option<TilesetMetadata> {
        self.entries.remove(id)
    }

    pub fn get(&self, id: &TilesetId) -> Option<&TilesetMetadata> {
        self.entries.get(id)
    }

    pub fn list_all(&self) -> Vec<&TilesetMetadata> {
        self.entries.values().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

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
