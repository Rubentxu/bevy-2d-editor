//! HIGH-1 phase 2 / H2.4: scene-asset state sub-module.
//!
//! Owns the SceneAssetCatalog, the active SceneAssetDocument, the catalog
//! warnings buffer, the AssetOperationLog (per-asset undo/redo), and the
//! ASSET_BODY_CACHE (BTreeMap<asset_ref, SceneAssetDocument> for O(1)
//! lookups during instance placement projection). Also owns the
//! RESYNC_REPORTS and VALIDATION_ISSUES accumulators.
//!
//! ## H2.4 (Asset Family collapse)
//!
//! All five asset-related thread_locals (`SCENE_ASSET_DOC`,
//! `ASSET_OPERATION_LOG`, `ASSET_BODY_CACHE`, `RESYNC_REPORTS`,
//! `VALIDATION_ISSUES`) have been removed. State now lives in two slots:
//!
//! - `EditorSession.active_asset: AssetFocus` (singleton ADT, parallels
//!   `SceneFocus` from H2.3) — carries `doc + log` of the focused asset.
//! - `EditorSession.asset_states["<path>"]: AssetSessionState` — carries
//!   per-path `catalog`, `catalog_warnings`, `body_cache`,
//!   `resync_reports`, `validation_issues`.
//!
//! The `with_*` helpers below are thin wrappers around the
//! `EditorSessionPort` so legacy call-sites continue to work.

use std::collections::BTreeMap;

use crate::asset_command::AssetOperationLog;
use crate::scene_asset::SceneAssetDocument;
use editor_model::scene_asset_catalog::{CatalogWarning, SceneAssetCatalog};

/// v0.91 PR2: Reserved key for the "active asset" slot on
/// `EditorSession::asset_states`.
pub const ACTIVE_ASSET_PATH: &str = "_active";

/// Get an immutable borrowed reference to the SceneAssetCatalog.
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
/// if needed. Writes back to the session.
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

/// Collect all catalog warnings accumulated during load_project.
pub fn get_asset_catalog_warnings() -> Vec<CatalogWarning> {
    editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(ACTIVE_ASSET_PATH)
            .catalog_warnings
            .clone()
    })
    .unwrap_or_default()
}

/// Clear all accumulated catalog warnings.
pub fn clear_asset_catalog_warnings() {
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(ACTIVE_ASSET_PATH)
            .catalog_warnings
            .clear();
    });
}

// ─────────────────────────────────────────────────────────────────────────
// H2.4 — AssetFocus ADT accessors (replaces SCENE_ASSET_DOC +
// ASSET_OPERATION_LOG thread_locals)
// ─────────────────────────────────────────────────────────────────────────

/// Get an immutable borrowed reference to the focused asset's document.
pub fn with_asset_doc<F, R>(f: F) -> R
where
    F: FnOnce(Option<&SceneAssetDocument>) -> R,
{
    // Use a trampoline: wrap `f` in an `Option` so the session closure
    // can take it once (returning Some(R)) and we can fall back to it
    // outside (when no session is registered).
    let mut slot: Option<F> = Some(f);
    let result: Option<Option<R>> = editor_model::ports::with_session_mut(|sess| -> Option<R> {
        let f = slot.take().expect("f is taken exactly once");
        let focus = sess.active_asset_mut();
        let doc_ptr = match focus {
            editor_model::AssetFocus::Empty => None,
            editor_model::AssetFocus::Focused { doc, .. } => Some(doc as *const _),
        };
        match doc_ptr {
            None => Some(f(None)),
            Some(ptr) => {
                // SAFETY: focus is borrowed mutably for the lifetime of
                // the closure, so the underlying `doc` is alive and
                // exclusively accessible for the entire `f` call.
                let doc_ref: &SceneAssetDocument = unsafe { &*ptr };
                Some(f(Some(doc_ref)))
            }
        }
    });
    // result is `Option<Option<R>>`:
    //   Outer None       → no session registered.
    //   Outer Some(None) → session registered, closure returned None (impossible
    //                      in practice because the closure always wraps in Some).
    //   Outer Some(Some(r)) → the answer.
    match result {
        Some(Some(r)) => r,
        _ => {
            let f = slot.expect("f is only here if not consumed above");
            f(None)
        }
    }
}

/// Get a mutable borrowed reference to the focused asset's document.
pub fn with_asset_doc_mut<F, R>(f: F) -> R
where
    F: FnOnce(Option<&mut SceneAssetDocument>) -> R,
{
    let mut slot: Option<F> = Some(f);
    let result: Option<Option<R>> = editor_model::ports::with_session_mut(|sess| -> Option<R> {
        let f = slot.take().expect("f is taken exactly once");
        let focus = sess.active_asset_mut();
        match focus {
            editor_model::AssetFocus::Empty => Some(f(None)),
            editor_model::AssetFocus::Focused { doc, .. } => Some(f(Some(doc))),
        }
    });
    match result {
        Some(Some(r)) => r,
        _ => {
            let f = slot.expect("f is only here if not consumed above");
            f(None)
        }
    }
}

/// Get an immutable borrowed reference to the focused asset's undo log.
pub fn with_asset_log<F, R>(f: F) -> R
where
    F: FnOnce(&AssetOperationLog) -> R,
{
    let mut slot: Option<F> = Some(f);
    let empty = AssetOperationLog::new();
    let result: Option<Option<R>> = editor_model::ports::with_session_mut(|sess| -> Option<R> {
        let f = slot.take().expect("f is taken exactly once");
        let focus = sess.active_asset_mut();
        match focus {
            editor_model::AssetFocus::Empty => Some(f(&empty)),
            editor_model::AssetFocus::Focused { log, .. } => Some(f(log)),
        }
    });
    match result {
        Some(Some(r)) => r,
        _ => {
            let f = slot.expect("f is only here if not consumed above");
            f(&empty)
        }
    }
}

/// Get a mutable borrowed reference to the focused asset's undo log.
pub fn with_asset_log_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut AssetOperationLog) -> R,
{
    let mut slot: Option<F> = Some(f);
    let mut empty = AssetOperationLog::new();
    let result: Option<Option<R>> = editor_model::ports::with_session_mut(|sess| -> Option<R> {
        let f = slot.take().expect("f is taken exactly once");
        let focus = sess.active_asset_mut();
        match focus {
            editor_model::AssetFocus::Empty => Some(f(&mut empty)),
            editor_model::AssetFocus::Focused { log, .. } => Some(f(log)),
        }
    });
    match result {
        Some(Some(r)) => r,
        _ => {
            let f = slot.expect("f is only here if not consumed above");
            f(&mut empty)
        }
    }
}

/// Borrow both the focused asset's document and its log mutably at the same
/// time, calling `f` with `(doc, log)`. Returns `Err("No asset open")` if
/// the focus is empty.
///
/// The session mutex is held for the entire call. Re-entrant paths that
/// need to release the mutex should use the kernel dispatch helpers instead
/// (see `dispatch_asset_command_via_kernel` in `lib.rs`).
pub fn with_asset_doc_and_log_mut<F, R>(f: F) -> Result<R, &'static str>
where
    F: FnOnce(&mut SceneAssetDocument, &mut AssetOperationLog) -> R,
{
    editor_model::ports::with_session_mut(|sess| {
        let focus = sess.active_asset_mut();
        match focus {
            editor_model::AssetFocus::Empty => Err("No asset open"),
            editor_model::AssetFocus::Focused { doc, log } => Ok(f(doc, log)),
        }
    })
    .unwrap_or(Err("No session registered"))
}

// ─────────────────────────────────────────────────────────────────────────
// H2.4 — Per-path asset caches (AssetSessionState)
// ─────────────────────────────────────────────────────────────────────────

/// Get an immutable borrowed reference to the ASSET_BODY_CACHE for the
/// active asset path.
pub fn with_asset_body_cache<F, R>(f: F) -> R
where
    F: FnOnce(&BTreeMap<String, SceneAssetDocument>) -> R,
{
    let mut slot: Option<F> = Some(f);
    let empty = BTreeMap::new();
    let result: Option<R> = editor_model::ports::with_session_mut(|sess| {
        let f = slot.take().expect("f is taken exactly once");
        let state = sess.asset_state_mut(ACTIVE_ASSET_PATH);
        match state.body_cache.as_ref() {
            Some(cache) => f(cache),
            None => f(&empty),
        }
    });
    match result {
        Some(r) => r,
        None => {
            let f = slot.expect("f is only here if not consumed above");
            f(&empty)
        }
    }
}

/// Get a mutable borrowed reference to the ASSET_BODY_CACHE for the active
/// asset path, lazily initializing an empty cache.
pub fn with_asset_body_cache_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut BTreeMap<String, SceneAssetDocument>) -> R,
{
    let mut slot: Option<F> = Some(f);
    let mut empty = BTreeMap::new();
    let result: Option<R> = editor_model::ports::with_session_mut(|sess| {
        let f = slot.take().expect("f is taken exactly once");
        let state = sess.asset_state_mut(ACTIVE_ASSET_PATH);
        if state.body_cache.is_none() {
            state.body_cache = Some(BTreeMap::new());
        }
        f(state.body_cache.as_mut().unwrap())
    });
    match result {
        Some(r) => r,
        None => {
            let f = slot.expect("f is only here if not consumed above");
            f(&mut empty)
        }
    }
}

/// H2.4: Drain the resync reports from the active asset path. The previous
/// behaviour (clear after read) is preserved.
pub fn take_resync_reports() -> Vec<(crate::document::StableId, crate::ResyncReport)> {
    editor_model::ports::with_session_mut(|sess| {
        let state = sess.asset_state_mut(ACTIVE_ASSET_PATH);
        std::mem::take(&mut state.resync_reports)
    })
    .unwrap_or_default()
}

/// H2.4: Replace the resync reports on the active asset path.
pub fn set_resync_reports(reports: Vec<(crate::document::StableId, crate::ResyncReport)>) {
    let _ = editor_model::ports::with_session_mut(|sess| {
        let state = sess.asset_state_mut(ACTIVE_ASSET_PATH);
        state.resync_reports = reports;
    });
}

/// H2.4: Drain the validation issues from the active asset path.
pub fn take_validation_issues() -> Vec<crate::ValidationIssue> {
    editor_model::ports::with_session_mut(|sess| {
        let state = sess.asset_state_mut(ACTIVE_ASSET_PATH);
        std::mem::take(&mut state.validation_issues)
    })
    .unwrap_or_default()
}

/// H2.4: Replace the validation issues on the active asset path.
pub fn set_validation_issues(issues: Vec<crate::ValidationIssue>) {
    let _ = editor_model::ports::with_session_mut(|sess| {
        let state = sess.asset_state_mut(ACTIVE_ASSET_PATH);
        state.validation_issues = issues;
    });
}
