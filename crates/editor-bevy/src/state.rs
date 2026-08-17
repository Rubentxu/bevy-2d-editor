//! HIGH-1 phase 2: state facade.
//!
//! Re-exports the four state sub-modules so existing call sites that
//! `use crate::state::*` continue to work without modification.
//!
//! The actual thread-local declarations and `with_*` helpers now live in:
//! - [`scene_state`]: SCENE_REGISTRY, DIRTY_FLAG, mark_dirty
//! - [`asset_state`]: SCENE_ASSET_CATALOG/DOC, ASSET_OPERATION_LOG,
//!   ASSET_BODY_CACHE, RESYNC_REPORTS, VALIDATION_ISSUES
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
    with_logic_graph, with_logic_graph_catalog, with_logic_graph_catalog_mut, with_logic_graph_mut,
    with_logic_log, with_logic_log_mut,
};
pub use crate::scene_state::{
    DIRTY_FLAG, SCENE_REGISTRY, mark_dirty, with_registry, with_registry_mut,
};

// v0.91 PR2: SCENE_ASSET_CATALOG and SCENE_ASSET_CATALOG_WARNINGS are no longer
// thread_locals — they live on `EditorSession::asset_states["_active"]` (see
// `editor_model::AssetSessionState`). The remaining thread_locals (DOC,
// OPERATION_LOG, BODY_CACHE, RESYNC_REPORTS, VALIDATION_ISSUES) stay for
// PR3 (causality migration) and PR5 (`OperationLog` type move).
pub use crate::asset_state::{
    ASSET_BODY_CACHE, ASSET_OPERATION_LOG, RESYNC_REPORTS, SCENE_ASSET_DOC, VALIDATION_ISSUES,
};
// v0.91 PR2: LOGIC_GRAPH_DOC is removed (migrated to session).
// LOGIC_GRAPH_CATALOG and LOGIC_OPERATION_LOG stay as thread_locals (PR3/PR5).
pub use crate::logic_state::{LOGIC_GRAPH_CATALOG, LOGIC_OPERATION_LOG};
