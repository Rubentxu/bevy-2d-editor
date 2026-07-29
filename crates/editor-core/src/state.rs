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
    get_asset_catalog_warnings, clear_asset_catalog_warnings, with_asset_body_cache,
    with_asset_body_cache_mut, with_asset_catalog, with_asset_catalog_mut, with_asset_doc,
    with_asset_doc_mut, with_asset_log, with_asset_log_mut,
};
pub use crate::hot_reload_state::{HotReloadRequest, PlayModeRequest, HOT_RELOAD_BUS, PLAY_MODE_REQUEST};
pub use crate::logic_state::{
    with_logic_graph, with_logic_graph_mut, with_logic_log, with_logic_log_mut,
    with_logic_graph_catalog, with_logic_graph_catalog_mut,
};
pub use crate::scene_state::{mark_dirty, with_registry, with_registry_mut, DIRTY_FLAG, SCENE_REGISTRY};

// The thread-locals SCENE_ASSET_CATALOG, SCENE_ASSET_DOC, etc. are
// referenced by name from lib.rs. The names live in asset_state now; we
// re-export them via the module path so `SCENE_ASSET_CATALOG.with(...)` in
// lib.rs resolves to crate::asset_state::SCENE_ASSET_CATALOG.
pub use crate::asset_state::{
    ASSET_BODY_CACHE, ASSET_OPERATION_LOG, RESYNC_REPORTS, SCENE_ASSET_CATALOG,
    SCENE_ASSET_CATALOG_WARNINGS, SCENE_ASSET_DOC, VALIDATION_ISSUES,
};
pub use crate::logic_state::{LOGIC_GRAPH_DOC, LOGIC_GRAPH_CATALOG, LOGIC_OPERATION_LOG};
