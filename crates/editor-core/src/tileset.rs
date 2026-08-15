//! Tileset types for the Bevy 2D Editor level design tools.
//!
//! PR2 refactoring: pure types moved to editor_model::tileset.
//! This module is now a thin re-export wrapper.

pub use editor_model::tileset::{
    AsepriteFrame, AsepriteMetadata, AsepriteSlice, AsepriteTag,
    TileCoord, TileGrid, TileRef, TilesetAsset, TilesetId,
    TilesetManager, TilesetMetadata,
};
