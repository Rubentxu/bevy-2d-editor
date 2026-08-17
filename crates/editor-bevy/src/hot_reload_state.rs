//! HIGH-1 phase 2: hot-reload state sub-module.
//!
//! Owns the HOT_RELOAD_BUS (queue of reload requests sent from
//! TypeScript via WASM bridge) and the PLAY_MODE_REQUEST flag (set by
//! WASM exports, consumed by a Bevy system).

use std::cell::RefCell;

/// Hot-reload request envelope — sent from TypeScript layer via WASM bridge
/// and consumed by process_hot_reload_requests in the Update schedule.
#[derive(Clone, Debug)]
pub enum HotReloadRequest {
    /// A source file (e.g. `.rs`) was saved; invalidate its cached content.
    Source { file_id: String },
    /// An asset file (e.g. `.bsn`) was saved or deleted; invalidate its body cache.
    Asset { asset_id: String },
    /// Full reload: clear source cache, asset body cache, and logic graph doc.
    ForceReloadAll,
}

/// Marker variant used by the PlayMode request flag (pub for lib.rs
/// re-export and tests).
#[derive(Clone)]
pub enum PlayModeRequest {
    Enter,
    Exit,
}

thread_local! {
    /// Thread-local hot-reload request bus — matches COMMAND_BUS/EVENT_BUS pattern.
    /// Consumed by process_hot_reload_requests each frame.
    pub static HOT_RELOAD_BUS: RefCell<Vec<HotReloadRequest>> = const { RefCell::new(Vec::new()) };
    /// Thread-local request flag set by WASM exports, consumed by a Bevy system.
    /// Follows the established DIRTY_FLAG pattern.
    pub static PLAY_MODE_REQUEST: RefCell<Option<PlayModeRequest>> = const { RefCell::new(None) };
}
