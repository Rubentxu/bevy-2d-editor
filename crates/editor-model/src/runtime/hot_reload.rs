//! Hot-reload request types — pure (no Bevy, no WASM) types.
//!
//! These types live in editor-model so editor-application (which holds
//! EditorSession) can own them without creating an
//! editor-application -> editor-bevy dependency edge.

use serde::{Deserialize, Serialize};

/// Request to hot-reload a node, source file, or graph asset.
///
/// Variants match the legacy `editor_bevy::hot_reload_state::HotReloadRequest`
/// shape 1:1 so the dual-write migration (Block G) can collapse the
/// duplicated declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HotReloadRequest {
    /// A source file (e.g. `.rs`) was saved; invalidate its cached content.
    Source {
        /// Stable source file ID (relative path or canonical hash).
        file_id: String,
    },
    /// An asset file (e.g. `.bsn`) was saved or deleted; invalidate its body cache.
    Asset {
        /// Stable asset ID of the node/graph to reload.
        asset_id: String,
    },
    /// Full reload: clear source cache, asset body cache, and logic graph doc.
    ForceReloadAll,
}

/// Request to switch play mode.
///
/// Variants match the legacy `editor_bevy::hot_reload_state::PlayModeRequest`
/// shape 1:1 so the dual-write migration (Block G) can collapse the
/// duplicated declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayModeRequest {
    /// Enter play mode (snapshot transforms, capture baselines).
    Enter,
    /// Exit play mode (compute deltas, restore transforms).
    Exit,
}
