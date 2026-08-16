//! RebuildCause — documents why the preview world was last rebuilt.
//!
//! §6: Recorded on every rebuild; 6 exhaustive variants.

use serde::{Deserialize, Serialize};

/// Why the preview world was last rebuilt.
///
/// Each variant documents a distinct trigger. The match must remain exhaustive —
/// adding a new variant is a deliberate architecture decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RebuildCause {
    /// User-initiated edit command (e.g. move sprite, property change).
    UserEdit {
        /// Identifier of the command that triggered the rebuild.
        command_id: String,
    },
    /// Hot-reload triggered by an external file change.
    HotReload {
        /// Identifier of the file that triggered hot-reload.
        file_id: String,
    },
    /// Entered Play mode, forcing a full scene reload.
    PlayModeEnter,
    /// Exited Play mode, returning to Edit mode.
    PlayModeExit,
    /// Switched to a different scene document.
    SceneSwitch {
        /// Scene path before the switch.
        from: String,
        /// Scene path after the switch.
        to: String,
    },
    /// A referenced Scene Asset was resynced, invalidating instance projections.
    AssetResync {
        /// Asset reference that was resynced.
        asset_ref: String,
    },
}
