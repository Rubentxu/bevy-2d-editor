//! Hot-reload request types — pure (no Bevy, no WASM) types.
//!
//! These types live in editor-model so editor-application (which holds
//! EditorSession) can own them without creating an
//! editor-application -> editor-bevy dependency edge.

use serde::{Deserialize, Serialize};

/// Request to hot-reload a node or graph asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadRequest {
    /// Stable asset ID of the node/graph to reload.
    pub asset_id: String,
}

/// Request to switch play mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayModeRequest {
    /// `true` to enter play mode, `false` to exit.
    pub play: bool,
}
