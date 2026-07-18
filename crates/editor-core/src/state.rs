//! HIGH-1 (god-module lib.rs) phase 1: thread-local state + access helpers.
//!
//! Extracted from lib.rs in v0.59.0. Contains:
//! - 3 `thread_local!` blocks (DIRTY_FLAG/SCENE_REGISTRY/SCENE_ASSET_*
//!   + ASSET_OPERATION_LOG/ASSET_BODY_CACHE/RESYNC_REPORTS/VALIDATION_ISSUES/
//!   LOGIC_GRAPH_DOC/LOGIC_OPERATION_LOG, HOT_RELOAD_BUS, PLAY_MODE_REQUEST)
//! - 16 `with_*` access helpers that lazily initialize + borrow the locals
//! - `mark_dirty()` and the catalog warning helpers
//!
//! All functions are `pub(super)` (lib-private) so callers in lib.rs can
//! use them via `crate::state::with_registry(...)`. Future phases of
//! HIGH-1 can split this further (e.g., scene_state, asset_state,
//! logic_state, hot_reload_state).

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::asset_command::AssetOperationLog;
use crate::auto_layer::AutoLayerId;
use crate::document::StableId;
use crate::logic_command::LogicOperationLog;
use crate::logic_graph::LogicGraphAsset;
use crate::scene_asset::SceneAssetDocument;
use crate::scene_asset_catalog::{CatalogWarning, SceneAssetCatalog};
use crate::scene_instance_overrides::ResyncReport;
use crate::scenes::SceneRegistry;
use crate::ValidationIssue;

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

/// Main thread-local state block. Holds the editor's persistent in-memory
/// caches, registries, and cross-system flags. All `with_*` helpers in
/// this module provide lazy-initializing accessors for these locals.
thread_local! {
    pub static DIRTY_FLAG: RefCell<bool> = const { RefCell::new(false) };
    pub static SCENE_REGISTRY: RefCell<Option<SceneRegistry>> = const { RefCell::new(None) };
    // Scene Asset catalog, document, and warnings holders (ADR-0008 §Decision).
    // Mirror of SCENE_REGISTRY/SCENE_DOC pattern for scene assets.
    pub static SCENE_ASSET_CATALOG: RefCell<Option<SceneAssetCatalog>> = const { RefCell::new(None) };
    pub static SCENE_ASSET_DOC: RefCell<Option<SceneAssetDocument>> = const { RefCell::new(None) };
    pub static SCENE_ASSET_CATALOG_WARNINGS: RefCell<Vec<CatalogWarning>> = const { RefCell::new(Vec::new()) };
    // Asset operation log: per-asset undo/redo history (ADR-0007).
    // Mirror of OPERATION_LOG pattern for scene assets.
    pub static ASSET_OPERATION_LOG: RefCell<AssetOperationLog> = const { RefCell::new(AssetOperationLog::new_const()) };
    // Asset body cache: BTreeMap<asset_ref, SceneAssetDocument> for O(1) lookups
    // during instance placement projection. No invalidation hooks yet (Task 1.5).
    pub static ASSET_BODY_CACHE: RefCell<Option<BTreeMap<String, SceneAssetDocument>>> = const { RefCell::new(None) };
    // Resync reports: accumulated during load/resync, drained by get_resync_reports().
    pub static RESYNC_REPORTS: RefCell<Vec<(StableId, ResyncReport)>> = const { RefCell::new(Vec::new()) };
    // Validation issues: accumulated during get_validation_issues, drained after.
    pub static VALIDATION_ISSUES: RefCell<Vec<ValidationIssue>> = const { RefCell::new(Vec::new()) };
    // Logic Graph document: the active logic graph being edited.
    pub static LOGIC_GRAPH_DOC: RefCell<Option<LogicGraphAsset>> = const { RefCell::new(None) };
    // Logic operation log: per-graph undo/redo history.
    pub static LOGIC_OPERATION_LOG: RefCell<LogicOperationLog> = const { RefCell::new(LogicOperationLog::new_const()) };
}

// Thread-local hot-reload request bus — matches COMMAND_BUS/EVENT_BUS pattern.
// Consumed by process_hot_reload_requests each frame.
thread_local! {
    pub static HOT_RELOAD_BUS: RefCell<Vec<HotReloadRequest>> = const { RefCell::new(Vec::new()) };
}

// Thread-local request flag set by WASM exports, consumed by a Bevy system.
// Follows the established DIRTY_FLAG pattern.
thread_local! {
    pub static PLAY_MODE_REQUEST: RefCell<Option<PlayModeRequest>> = const { RefCell::new(None) };
}

/// Get an immutable borrowed reference to the SceneRegistry, initializing if needed.
pub fn with_registry<F, R>(f: F) -> R
where
    F: FnOnce(&SceneRegistry) -> R,
{
    SCENE_REGISTRY.with(|cell| {
        let mut_ref = &mut *cell.borrow_mut();
        if mut_ref.is_none() {
            *mut_ref = Some(SceneRegistry::default());
        }
        f(mut_ref.as_ref().unwrap())
    })
}

/// Get a mutable borrowed reference to the SceneRegistry, initializing if needed.
pub fn with_registry_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut SceneRegistry) -> R,
{
    SCENE_REGISTRY.with(|cell| {
        let mut_ref = &mut *cell.borrow_mut();
        if mut_ref.is_none() {
            *mut_ref = Some(SceneRegistry::default());
        }
        f(mut_ref.as_mut().unwrap())
    })
}

/// Get an immutable borrowed reference to the SceneAssetCatalog, initializing if needed.
pub fn with_asset_catalog<F, R>(f: F) -> R
where
    F: FnOnce(&SceneAssetCatalog) -> R,
{
    SCENE_ASSET_CATALOG.with(|cell| {
        let mut_ref = &mut *cell.borrow_mut();
        if mut_ref.is_none() {
            *mut_ref = Some(SceneAssetCatalog::new());
        }
        f(mut_ref.as_ref().unwrap())
    })
}

/// Get a mutable borrowed reference to the SceneAssetCatalog, initializing if needed.
pub fn with_asset_catalog_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut SceneAssetCatalog) -> R,
{
    SCENE_ASSET_CATALOG.with(|cell| {
        let mut_ref = &mut *cell.borrow_mut();
        if mut_ref.is_none() {
            *mut_ref = Some(SceneAssetCatalog::new());
        }
        f(mut_ref.as_mut().unwrap())
    })
}

/// Get an immutable borrowed reference to the active SceneAssetDocument.
pub fn with_asset_doc<F, R>(f: F) -> R
where
    F: FnOnce(&Option<SceneAssetDocument>) -> R,
{
    SCENE_ASSET_DOC.with(|cell| f(&*cell.borrow()))
}

/// Get a mutable borrowed reference to the active SceneAssetDocument.
pub fn with_asset_doc_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut Option<SceneAssetDocument>) -> R,
{
    SCENE_ASSET_DOC.with(|cell| f(&mut *cell.borrow_mut()))
}

/// Collect all catalog warnings accumulated during load_project.
pub fn get_asset_catalog_warnings() -> Vec<CatalogWarning> {
    SCENE_ASSET_CATALOG_WARNINGS.with(|cell| cell.borrow().clone())
}

/// Clear all accumulated catalog warnings.
pub fn clear_asset_catalog_warnings() {
    SCENE_ASSET_CATALOG_WARNINGS.with(|cell| cell.borrow_mut().clear());
}

/// Get an immutable borrowed reference to the AssetOperationLog.
pub fn with_asset_log<F, R>(f: F) -> R
where
    F: FnOnce(&AssetOperationLog) -> R,
{
    ASSET_OPERATION_LOG.with(|cell| f(&*cell.borrow()))
}

/// Get a mutable borrowed reference to the AssetOperationLog.
pub fn with_asset_log_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut AssetOperationLog) -> R,
{
    ASSET_OPERATION_LOG.with(|cell| f(&mut *cell.borrow_mut()))
}

/// Get an immutable borrowed reference to the ASSET_BODY_CACHE.
pub fn with_asset_body_cache<F, R>(f: F) -> R
where
    F: FnOnce(&BTreeMap<String, SceneAssetDocument>) -> R,
{
    ASSET_BODY_CACHE.with(|cell| {
        let cache = cell.borrow();
        if cache.is_none() {
            // Initialize empty cache on first access
            f(&BTreeMap::new())
        } else {
            f(cache.as_ref().unwrap())
        }
    })
}

/// Get a mutable borrowed reference to the ASSET_BODY_CACHE.
pub fn with_asset_body_cache_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut BTreeMap<String, SceneAssetDocument>) -> R,
{
    ASSET_BODY_CACHE.with(|cell| {
        let mut cache = cell.borrow_mut();
        if cache.is_none() {
            *cache = Some(BTreeMap::new());
        }
        f(cache.as_mut().unwrap())
    })
}

/// Mark the current scene as dirty (set DIRTY_FLAG + registry flag).
/// Triggers rebuild_preview_world on the next frame.
pub fn mark_dirty() {
    DIRTY_FLAG.with(|d| *d.borrow_mut() = true);
    with_registry_mut(|r| r.mark_current_dirty());
}

/// Get an immutable borrowed reference to the LogicGraphAsset.
pub fn with_logic_graph<F, R>(f: F) -> R
where
    F: FnOnce(&Option<LogicGraphAsset>) -> R,
{
    LOGIC_GRAPH_DOC.with(|cell| f(&*cell.borrow()))
}

/// Get a mutable borrowed reference to the LogicGraphAsset.
pub fn with_logic_graph_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut Option<LogicGraphAsset>) -> R,
{
    LOGIC_GRAPH_DOC.with(|cell| f(&mut *cell.borrow_mut()))
}

/// Get an immutable borrowed reference to the LogicOperationLog.
pub fn with_logic_log<F, R>(f: F) -> R
where
    F: FnOnce(&LogicOperationLog) -> R,
{
    LOGIC_OPERATION_LOG.with(|cell| f(&*cell.borrow()))
}

/// Get a mutable borrowed reference to the LogicOperationLog.
pub fn with_logic_log_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut LogicOperationLog) -> R,
{
    LOGIC_OPERATION_LOG.with(|cell| f(&mut *cell.borrow_mut()))
}

// Keep AutoLayerId imported (used by future extraction phases).
#[allow(dead_code)]
fn _auto_layer_id_marker(_: AutoLayerId) {}
