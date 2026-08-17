//! Thread-local state for World Workspace (ADR-0037).
//!
//! Provides `WORLD_DOC` and `WORLD_CATALOG` thread-local accessors
//! for use within the WASM runtime. These mirror the existing
//! `SCENE_DOC` / `SCENE_CATALOG` pattern.
//!
//! # Lifetime model
//!
//! - `WORLD_DOC` — set once per editor session when a world is opened
//! - `WORLD_CATALOG` — set once per editor session when the session initializes
//!
//! Both are cleared when the editor session ends.

use editor_model::scene_asset_catalog::SceneAssetCatalog;
use editor_model::world::WorldDocument;
use std::cell::RefCell;

/// Thread-local handle for the active `WorldDocument`.
///
/// Set by the WASM boundary when a world is opened; cleared on session end.
thread_local! {
    static WORLD_DOC: RefCell<Option<WorldDocument>> = RefCell::new(None);
}

/// Thread-local handle for the active `SceneAssetCatalog`.
///
/// Set once during session initialization; shared across all worlds in the session.
thread_local! {
    static WORLD_CATALOG: RefCell<Option<SceneAssetCatalog>> = RefCell::new(None);
}

// ─────────────────────────────────────────────────────────────────────────────
// Accessors
// ─────────────────────────────────────────────────────────────────────────────

/// Get a reference to the currently active `WorldDocument`.
///
/// # Panics
///
/// Panics if no world document is currently set. Callers should check
/// `is_world_doc_set()` first if the document may not be loaded.
pub fn with_world_doc<F, R>(f: F) -> R
where
    F: FnOnce(&WorldDocument) -> R,
{
    WORLD_DOC.with(|cell| {
        let borrowed = cell.borrow();
        let doc = borrowed
            .as_ref()
            .expect("WORLD_DOC is not set; call set_world_doc() first");
        f(doc)
    })
}

/// Get a mutable reference to the currently active `WorldDocument`.
///
/// # Panics
///
/// Panics if no world document is currently set.
pub fn with_world_doc_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut WorldDocument) -> R,
{
    WORLD_DOC.with(|cell| {
        let mut borrowed = cell.borrow_mut();
        let doc = borrowed
            .as_mut()
            .expect("WORLD_DOC is not set; call set_world_doc() first");
        f(doc)
    })
}

/// Get a reference to the active `SceneAssetCatalog`.
pub fn with_world_catalog<F, R>(f: F) -> R
where
    F: FnOnce(&SceneAssetCatalog) -> R,
{
    WORLD_CATALOG.with(|cell| {
        let borrowed = cell.borrow();
        let catalog = borrowed
            .as_ref()
            .expect("WORLD_CATALOG is not set; call set_world_catalog() first");
        f(catalog)
    })
}

/// Get a mutable reference to the active `SceneAssetCatalog`.
pub fn with_world_catalog_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut SceneAssetCatalog) -> R,
{
    WORLD_CATALOG.with(|cell| {
        let mut borrowed = cell.borrow_mut();
        let catalog = borrowed
            .as_mut()
            .expect("WORLD_CATALOG is not set; call set_world_catalog() first");
        f(catalog)
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Setters
// ─────────────────────────────────────────────────────────────────────────────

/// Set the active `WorldDocument`, replacing any existing value.
pub fn set_world_doc(doc: WorldDocument) {
    WORLD_DOC.with(|cell| {
        *cell.borrow_mut() = Some(doc);
    });
}

/// Set the active `SceneAssetCatalog`, replacing any existing value.
pub fn set_world_catalog(catalog: SceneAssetCatalog) {
    WORLD_CATALOG.with(|cell| {
        *cell.borrow_mut() = Some(catalog);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Utility
// ─────────────────────────────────────────────────────────────────────────────

/// Returns `true` if a `WorldDocument` is currently set.
pub fn is_world_doc_set() -> bool {
    WORLD_DOC.with(|cell| cell.borrow().is_some())
}

/// Returns `true` if a `SceneAssetCatalog` is currently set.
pub fn is_world_catalog_set() -> bool {
    WORLD_CATALOG.with(|cell| cell.borrow().is_some())
}

/// Clear the active `WorldDocument` (e.g., when the world is closed).
pub fn clear_world_doc() {
    WORLD_DOC.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// Clear the active `SceneAssetCatalog`.
pub fn clear_world_catalog() {
    WORLD_CATALOG.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// Run a closure with a borrowed `WorldDocument`, returning `None` if no
/// document is currently set.
pub fn with_world_doc_opt<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&WorldDocument) -> R,
{
    WORLD_DOC.with(|cell| {
        let borrowed = cell.borrow();
        borrowed.as_ref().map(f)
    })
}

/// Run a closure with a borrowed `SceneAssetCatalog`, returning `None` if no
/// catalog is currently set.
pub fn with_world_catalog_opt<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&SceneAssetCatalog) -> R,
{
    WORLD_CATALOG.with(|cell| {
        let borrowed = cell.borrow();
        borrowed.as_ref().map(f)
    })
}
