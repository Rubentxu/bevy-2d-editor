//! HIGH-1 phase 2: scene-asset state sub-module.
//!
//! Owns the SceneAssetCatalog, the active SceneAssetDocument, the catalog
//! warnings buffer, the AssetOperationLog (per-asset undo/redo), and the
//! ASSET_BODY_CACHE (BTreeMap<asset_ref, SceneAssetDocument> for O(1)
//! lookups during instance placement projection). Also owns the
//! RESYNC_REPORTS and VALIDATION_ISSUES accumulators.

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::ValidationIssue;
use crate::asset_command::AssetOperationLog;
use crate::document::StableId;
use crate::scene_asset::SceneAssetDocument;
use crate::scene_instance_overrides::ResyncReport;
use editor_model::scene_asset_catalog::{CatalogWarning, SceneAssetCatalog};

/// v0.91 PR2: Reserved key for the "active asset" slot on
/// `EditorSession::asset_states`.
pub const ACTIVE_ASSET_PATH: &str = "_active";

thread_local! {
    /// v0.91 PR2 transitional: `SCENE_ASSET_DOC`, `ASSET_OPERATION_LOG`,
    /// `ASSET_BODY_CACHE`, `RESYNC_REPORTS`, `VALIDATION_ISSUES` remain as
    /// thread_locals. Migration to `EditorSession` is the responsibility of
    /// PR3 (causality migration) and PR5 (`OperationLog` type move).
    /// The two thread_locals that PR2 *does* migrate are
    /// `SCENE_ASSET_CATALOG` and `SCENE_ASSET_CATALOG_WARNINGS` — they
    /// were merged into `EditorSessionPort::asset_state_mut()` (see
    /// `editor_model::AssetSessionState`).
    pub static SCENE_ASSET_DOC: RefCell<Option<SceneAssetDocument>> = const { RefCell::new(None) };
    /// Asset operation log: per-asset undo/redo history (ADR-0007).
    /// Migrated to `EditorSession` in v0.91 PR5 (requires the `OperationLog`
    /// type to move from `editor-core` to `editor-model`).
    pub static ASSET_OPERATION_LOG: RefCell<AssetOperationLog> = const { RefCell::new(AssetOperationLog::new_const()) };
    /// Asset body cache: BTreeMap<asset_ref, SceneAssetDocument> for O(1) lookups.
    pub static ASSET_BODY_CACHE: RefCell<Option<BTreeMap<String, SceneAssetDocument>>> = const { RefCell::new(None) };
    /// Resync reports: accumulated during load/resync, drained by get_resync_reports().
    pub static RESYNC_REPORTS: RefCell<Vec<(StableId, ResyncReport)>> = const { RefCell::new(Vec::new()) };
    /// Validation issues: accumulated during get_validation_issues, drained after.
    pub static VALIDATION_ISSUES: RefCell<Vec<ValidationIssue>> = const { RefCell::new(Vec::new()) };
}

/// Get an immutable borrowed reference to the SceneAssetCatalog (v0.91 PR2:
/// reads from `EditorSession::asset_states["_active"].catalog`).
pub fn with_asset_catalog<F, R>(f: F) -> R
where
    F: FnOnce(&SceneAssetCatalog) -> R,
{
    let mut catalog = editor_model::ports::with_session_mut(|sess| {
        let state = sess.asset_state_mut(ACTIVE_ASSET_PATH);
        if state.catalog.is_none() {
            state.catalog = Some(SceneAssetCatalog::new());
        }
        state.catalog.clone()
    })
    .flatten()
    .unwrap_or_else(|| SceneAssetCatalog::new());
    f(&catalog)
}

/// Get a mutable borrowed reference to the SceneAssetCatalog, initializing
/// if needed. v0.91 PR2: writes to the session.
pub fn with_asset_catalog_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut SceneAssetCatalog) -> R,
{
    let mut catalog = editor_model::ports::with_session_mut(|sess| {
        let state = sess.asset_state_mut(ACTIVE_ASSET_PATH);
        if state.catalog.is_none() {
            state.catalog = Some(SceneAssetCatalog::new());
        }
        state.catalog.clone()
    })
    .flatten()
    .unwrap_or_else(|| SceneAssetCatalog::new());
    let result = f(&mut catalog);
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(ACTIVE_ASSET_PATH).catalog = Some(catalog);
    });
    result
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
/// v0.91 PR2: reads from `EditorSession::asset_states["_active"].catalog_warnings`.
pub fn get_asset_catalog_warnings() -> Vec<CatalogWarning> {
    editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(ACTIVE_ASSET_PATH)
            .catalog_warnings
            .clone()
    })
    .unwrap_or_default()
}

/// Clear all accumulated catalog warnings.
/// v0.91 PR2: writes to the session.
pub fn clear_asset_catalog_warnings() {
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(ACTIVE_ASSET_PATH)
            .catalog_warnings
            .clear();
    });
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
