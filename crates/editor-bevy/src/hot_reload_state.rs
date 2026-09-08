//! HIGH-1 phase 2: hot-reload state sub-module.
//!
//! Owns the dual-write fallback thread_locals for hot-reload requests
//! and play-mode requests. The canonical owner is `EditorSession`
//! (via `editor_model::runtime::{HotReloadRequest, PlayModeRequest}`);
//! these thread_locals only exist so legacy tests + non-session
//! entrypoints keep working while the migration (H2.5 Block G) lands.
//! OPEN until §S-G parity tests prove full coverage.

/// Re-export from `editor_model::runtime` (Block G consolidated the
/// duplicate declarations — `editor_bevy::hot_reload_state::{HotReloadRequest,
/// PlayModeRequest}` and `editor_model::runtime::{HotReloadRequest,
/// PlayModeRequest}` are now the SAME type).
pub use editor_model::runtime::{HotReloadRequest, PlayModeRequest};

use std::cell::RefCell;

thread_local! {
    /// Dual-write fallback for hot-reload requests.
    ///
    /// Production code writes through `EditorSession.runtime.hot_reload_requests`
    /// via `with_session_mut(|s| s.runtime_hot_reload_requests_mut().push(...))`.
    /// This thread_local only exists so legacy tests + non-session entrypoints
    /// keep working while the migration lands. OPEN until §S-G parity tests
    /// prove full coverage.
    pub static HOT_RELOAD_BUS_FALLBACK: RefCell<Vec<HotReloadRequest>> =
        const { RefCell::new(Vec::new()) };

    /// Dual-write fallback for play-mode requests.
    ///
    /// Production code writes through `EditorSession.runtime.play_mode_request`
    /// via `with_session_mut(|s| *s.runtime_play_mode_request_mut() = ...)`.
    /// This thread_local only exists so legacy tests + non-session entrypoints
    /// keep working while the migration lands. OPEN until §S-G parity tests
    /// prove full coverage.
    pub static PLAY_MODE_REQUEST_FALLBACK: RefCell<Option<PlayModeRequest>> =
        const { RefCell::new(None) };
}
