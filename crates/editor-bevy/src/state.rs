//! HIGH-1 phase 2: state facade.
//!
//! Re-exports the four state sub-modules so existing call sites that
//! `use crate::state::*` continue to work without modification.
//!
//! The actual thread-local declarations and `with_*` helpers now live in:
//! - [`scene_state`]: SCENE_REGISTRY, mark_dirty (H2.3 — DIRTY_FLAG moved to
//!   `EditorSession.active_scene.dirty`, see `scene_session::*`)
//! - [`asset_state`]: thin wrappers over `EditorSession.active_asset` (H2.4
//!   — SCENE_ASSET_DOC + ASSET_OPERATION_LOG collapsed into `AssetFocus`,
//!   plus ASSET_BODY_CACHE / RESYNC_REPORTS / VALIDATION_ISSUES per-path on
//!   `AssetSessionState`).
//! - [`logic_state`]: LOGIC_GRAPH_DOC, LOGIC_OPERATION_LOG
//! - [`hot_reload_state`]: HOT_RELOAD_BUS, PLAY_MODE_REQUEST

pub use crate::asset_state::{
    clear_asset_catalog_warnings, get_asset_catalog_warnings, with_asset_body_cache,
    with_asset_body_cache_mut, with_asset_catalog, with_asset_catalog_mut, with_asset_doc,
    with_asset_doc_and_log_mut, with_asset_doc_mut, with_asset_log, with_asset_log_mut,
};
pub use crate::hot_reload_state::{
    HOT_RELOAD_BUS, HotReloadRequest, PLAY_MODE_REQUEST, PlayModeRequest,
};
pub use crate::logic_state::{
    with_binding_registry, with_binding_registry_mut, with_logic_graph, with_logic_graph_catalog,
    with_logic_graph_catalog_mut, with_logic_graph_mut, with_logic_log, with_logic_log_mut,
};
// BindingRecord is needed by tests
pub use crate::logic_state::BindingRecord;
pub use crate::logic_state::LOGIC_BINDING_REGISTRY;
pub use crate::scene_state::{SCENE_REGISTRY, mark_dirty, with_registry, with_registry_mut};

// H2.3: SCENE_DOC, OPERATION_LOG and DIRTY_FLAG thread_locals removed —
// they live on `EditorSession.active_scene: SceneFocus` (see
// `editor_model::SceneFocus` and `editor_model::ports::with_session_mut`).

// H2.4 (full): every asset thread_local is gone. SCENE_ASSET_DOC +
// ASSET_OPERATION_LOG collapsed into `EditorSession.active_asset:
// AssetFocus`; SCENE_ASSET_CATALOG + warnings + ASSET_BODY_CACHE +
// RESYNC_REPORTS + VALIDATION_ISSUES live per-path on
// `EditorSession::asset_states[path]`. The helpers in `asset_state.rs`
// are now thin wrappers over the session port.
// No thread_locals remain in this asset family.
// v0.91 PR2: LOGIC_GRAPH_DOC is removed (migrated to session).
// LOGIC_GRAPH_CATALOG and LOGIC_OPERATION_LOG stay as thread_locals (PR3/PR5).
pub use crate::logic_state::{LOGIC_GRAPH_CATALOG, LOGIC_OPERATION_LOG};
