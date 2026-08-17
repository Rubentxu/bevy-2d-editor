//! AutoLayer — auto-tiling generation engine for tile-based level design.
//!
//! PR2 refactoring: pure types moved to editor_model::auto_layer.
//! This module is now a thin re-export wrapper.

pub use editor_model::auto_layer::{
    AutoLayer, AutoLayerId, AutoRule, Pattern3x3, PatternCell, is_auto_layer_stale, regenerate,
};
