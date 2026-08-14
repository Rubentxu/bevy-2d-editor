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
use crate::scene_asset_catalog::{CatalogWarning, SceneAssetCatalog};
use crate::scene_instance_overrides::ResyncReport;

thread_local! {
    /// Scene Asset catalog, document, and warnings holders (ADR-0008 §Decision).
    /// Mirror of SCENE_REGISTRY/SCENE_DOC pattern for scene assets.
    pub static SCENE_ASSET_CATALOG: RefCell<Option<SceneAssetCatalog>> = const { RefCell::new(None) };
    pub static SCENE_ASSET_DOC: RefCell<Option<SceneAssetDocument>> = const { RefCell::new(None) };
    pub static SCENE_ASSET_CATALOG_WARNINGS: RefCell<Vec<CatalogWarning>> = const { RefCell::new(Vec::new()) };
    /// Asset operation log: per-asset undo/redo history (ADR-0007).
    pub static ASSET_OPERATION_LOG: RefCell<AssetOperationLog> = const { RefCell::new(AssetOperationLog::new_const()) };
    /// Asset body cache: BTreeMap<asset_ref, SceneAssetDocument> for O(1) lookups.
    pub static ASSET_BODY_CACHE: RefCell<Option<BTreeMap<String, SceneAssetDocument>>> = const { RefCell::new(None) };
    /// Resync reports: accumulated during load/resync, drained by get_resync_reports().
    pub static RESYNC_REPORTS: RefCell<Vec<(StableId, ResyncReport)>> = const { RefCell::new(Vec::new()) };
    /// Validation issues: accumulated during get_validation_issues, drained after.
    pub static VALIDATION_ISSUES: RefCell<Vec<ValidationIssue>> = const { RefCell::new(Vec::new()) };
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
