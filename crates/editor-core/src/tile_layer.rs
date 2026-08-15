//! TileLayer — a layer inside a LevelSceneAsset that stores a grid of tiles.
//!
//! PR2 refactoring: pure types moved to editor_model::tile_layer.
//! This module is now a thin re-export wrapper.

pub use editor_model::tile_layer::TileLayer;

// TileLayerId is now editor_model::ids::LayerId.
pub type TileLayerId = editor_model::ids::LayerId;
