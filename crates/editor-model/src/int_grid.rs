//! IntGridLayer — a layer inside a LevelSceneAsset that stores a grid of integer values.
//!
//! IntGrid layers are used by external source importers (LDtk, Tiled) to represent
//! semantic grid data where each cell holds a typed integer value rather than a tile reference.

use crate::ids::LayerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Opaque stable identifier for an IntGridLayer inside a LevelSceneAsset.
pub type IntGridLayerId = LayerId;

/// A coordinate in the IntGrid.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntGridCoord {
    /// X coordinate (column index).
    pub x: i32,
    /// Y coordinate (row index).
    pub y: i32,
}

impl IntGridCoord {
    /// Construct a new coordinate.
    pub fn new(x: i32, y: i32) -> Self {
        IntGridCoord { x, y }
    }
}

impl serde::Serialize for IntGridCoord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{},{}", self.x, self.y))
    }
}

impl<'de> serde::Deserialize<'de> for IntGridCoord {
    fn deserialize<D>(deserializer: D) -> Result<IntGridCoord, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::custom("expected 'x,y' format"));
        }
        let x = parts[0].parse().map_err(serde::de::Error::custom)?;
        let y = parts[1].parse().map_err(serde::de::Error::custom)?;
        Ok(IntGridCoord { x, y })
    }
}

/// Schema discriminator for IntGrid cell values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IntGridSchemaKind {
    /// Cells reference tile identifiers from a tileset.
    TileRef,
    /// Cells hold arbitrary integer values with optional string identifiers.
    #[default]
    Values,
}

/// A single cell in an IntGrid layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntGridCell {
    /// X coordinate in grid units.
    pub x: i32,
    /// Y coordinate in grid units.
    pub y: i32,
    /// The integer value stored at this cell.
    pub value: i32,
    /// Optional string identifier for the value (e.g., "solid", "water").
    /// Only present when `schema_kind == IntGridSchemaKind::Values`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
}

/// Sparse map of IntGrid coordinates to cells.
pub type IntGridMap = HashMap<IntGridCoord, IntGridCell>;

/// An IntGrid layer stores a sparse grid of integer values.
///
/// Unlike [`TileLayer`](crate::tile_layer::TileLayer) which references tiles from a tileset,
/// IntGrid layers store raw integer values that can represent collision rules,
/// terrain types, or any custom semantic data defined by the external source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntGridLayer {
    /// Unique identifier for this layer.
    pub id: IntGridLayerId,
    /// Human-readable layer name.
    pub name: String,
    /// Optional layer identifier from the external source (e.g., LDtk layer `identifier`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    /// How cell values are interpreted.
    #[serde(default)]
    pub schema_kind: IntGridSchemaKind,
    /// Sparse map of grid coordinates to cell values.
    #[serde(default)]
    pub values: IntGridMap,
    /// Z-ordering index.
    pub order: i32,
    /// Incremented each time the grid is modified.
    #[serde(default)]
    pub generation: u64,
    /// Width of the grid in cells.
    #[serde(default = "default_grid_width")]
    pub grid_width: u32,
    /// Height of the grid in cells.
    #[serde(default = "default_grid_height")]
    pub grid_height: u32,
    /// Optional metadata field for external source-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

fn default_grid_width() -> u32 {
    50
}

fn default_grid_height() -> u32 {
    50
}

impl IntGridLayer {
    /// Construct a new IntGridLayer with default grid dimensions (50×50) and Values schema.
    pub fn new(id: IntGridLayerId, name: String) -> Self {
        IntGridLayer {
            id,
            name,
            identifier: None,
            schema_kind: IntGridSchemaKind::Values,
            values: HashMap::new(),
            order: 0,
            generation: 0,
            grid_width: default_grid_width(),
            grid_height: default_grid_height(),
            metadata: None,
        }
    }

    /// Construct a new IntGridLayer with TileRef schema.
    pub fn with_tile_ref(id: IntGridLayerId, name: String) -> Self {
        IntGridLayer {
            id,
            name,
            identifier: None,
            schema_kind: IntGridSchemaKind::TileRef,
            values: HashMap::new(),
            order: 0,
            generation: 0,
            grid_width: default_grid_width(),
            grid_height: default_grid_height(),
            metadata: None,
        }
    }

    /// Set the layer identifier from the external source.
    pub fn with_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    /// Set the grid dimensions explicitly.
    pub fn with_dimensions(mut self, grid_width: u32, grid_height: u32) -> Self {
        self.grid_width = grid_width;
        self.grid_height = grid_height;
        self
    }

    /// Set the layer order.
    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// Set optional metadata.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set the schema kind.
    pub fn with_schema_kind(mut self, schema_kind: IntGridSchemaKind) -> Self {
        self.schema_kind = schema_kind;
        self
    }

    /// Paint a cell at `(x, y)`, replacing any existing value. Increments generation.
    pub fn paint_cell(&mut self, x: i32, y: i32, value: i32, identifier: Option<String>) {
        self.values.insert(
            IntGridCoord::new(x, y),
            IntGridCell {
                x,
                y,
                value,
                identifier,
            },
        );
        self.generation += 1;
    }

    /// Erase the cell at `(x, y)`. Returns the erased cell. Increments generation.
    pub fn erase_cell(&mut self, x: i32, y: i32) -> Option<IntGridCell> {
        let result = self.values.remove(&IntGridCoord::new(x, y));
        if result.is_some() {
            self.generation += 1;
        }
        result
    }

    /// Look up the cell at `(x, y)`.
    pub fn get_cell(&self, x: i32, y: i32) -> Option<&IntGridCell> {
        self.values.get(&IntGridCoord::new(x, y))
    }

    /// Total number of painted cells.
    pub fn cell_count(&self) -> usize {
        self.values.len()
    }

    /// True when no cells are painted.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_grid_coord_serde() {
        let coord = IntGridCoord::new(5, 10);
        let json = serde_json::to_string(&coord).unwrap();
        assert_eq!(json, "\"5,10\"");
        let parsed: IntGridCoord = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.x, 5);
        assert_eq!(parsed.y, 10);
    }

    #[test]
    fn test_int_grid_layer_paint_and_get() {
        let mut layer = IntGridLayer::new(
            IntGridLayerId::new("ig_collision".to_string()),
            "Collision".to_string(),
        );

        layer.paint_cell(3, 7, 1, Some("solid".to_string()));
        let cell = layer.get_cell(3, 7);
        assert!(cell.is_some());
        let cell = cell.unwrap();
        assert_eq!(cell.x, 3);
        assert_eq!(cell.y, 7);
        assert_eq!(cell.value, 1);
        assert_eq!(cell.identifier.as_deref(), Some("solid"));
    }

    #[test]
    fn test_int_grid_layer_erase() {
        let mut layer = IntGridLayer::new(
            IntGridLayerId::new("ig_test".to_string()),
            "Test Layer".to_string(),
        );

        assert!(layer.erase_cell(0, 0).is_none());
        layer.paint_cell(0, 0, 5, None);
        let erased = layer.erase_cell(0, 0);
        assert!(erased.is_some());
        assert_eq!(erased.unwrap().value, 5);
        assert!(layer.get_cell(0, 0).is_none());
    }

    #[test]
    fn test_int_grid_layer_tile_ref_schema() {
        let mut layer = IntGridLayer::with_tile_ref(
            IntGridLayerId::new("ig_tiles".to_string()),
            "Tile Refs".to_string(),
        );
        assert_eq!(layer.schema_kind, IntGridSchemaKind::TileRef);

        layer.paint_cell(0, 0, 3, None); // tile index 3
        let cell = layer.get_cell(0, 0).unwrap();
        assert_eq!(cell.value, 3);
        assert!(cell.identifier.is_none());
    }

    #[test]
    fn test_int_grid_layer_generation_bumped_on_paint() {
        let mut layer = IntGridLayer::new(
            IntGridLayerId::new("ig_gen".to_string()),
            "Gen Test".to_string(),
        );
        assert_eq!(layer.generation, 0);

        layer.paint_cell(0, 0, 1, None);
        assert_eq!(layer.generation, 1);

        layer.paint_cell(0, 0, 2, None); // overwriting bumps generation
        assert_eq!(layer.generation, 2);
    }

    #[test]
    fn test_int_grid_layer_is_empty() {
        let layer = IntGridLayer::new(
            IntGridLayerId::new("ig_empty".to_string()),
            "Empty Layer".to_string(),
        );
        assert!(layer.is_empty());
        assert_eq!(layer.cell_count(), 0);
    }

    #[test]
    fn test_int_grid_layer_round_trip() {
        let mut layer = IntGridLayer::new(
            IntGridLayerId::new("ig_roundtrip".to_string()),
            "Round Trip".to_string(),
        )
        .with_identifier("Collision")
        .with_order(2)
        .with_dimensions(100, 80);

        layer.paint_cell(0, 0, 1, Some("solid".to_string()));
        layer.paint_cell(1, 0, 0, None);
        layer.paint_cell(5, 10, 3, Some("water".to_string()));

        let json = serde_json::to_string(&layer).unwrap();
        let parsed: IntGridLayer = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id.as_str(), "ig_roundtrip");
        assert_eq!(parsed.name, "Round Trip");
        assert_eq!(parsed.identifier.as_deref(), Some("Collision"));
        assert_eq!(parsed.order, 2);
        assert_eq!(parsed.grid_width, 100);
        assert_eq!(parsed.grid_height, 80);
        assert_eq!(parsed.cell_count(), 3);

        let cell = parsed.get_cell(5, 10).unwrap();
        assert_eq!(cell.value, 3);
        assert_eq!(cell.identifier.as_deref(), Some("water"));
    }
}
