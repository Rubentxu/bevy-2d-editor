//! HIGH-1 phase 2: scene state sub-module.
//!
//! Owns the SceneRegistry and the cross-system DIRTY_FLAG. Split out from
//! state.rs to keep each state concern in its own file. Re-exported via
//! state.rs for backward compatibility with the existing `crate::state::*`
//! import in lib.rs.

use std::cell::RefCell;

use crate::scenes::SceneRegistry;

thread_local! {
    /// Cross-system dirty flag set by `dispatch_command` and read by
    /// `rebuild_preview_world`. Visible across the WASM→Bevy boundary
    /// because both run on the same thread (single-threaded WASM).
    pub static DIRTY_FLAG: RefCell<bool> = const { RefCell::new(false) };
    /// Scene registry: maps scene_id → loaded scene metadata.
    pub static SCENE_REGISTRY: RefCell<Option<SceneRegistry>> = const { RefCell::new(None) };
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

/// Mark the current scene as dirty (set DIRTY_FLAG + registry flag).
/// Triggers rebuild_preview_world on the next frame.
pub fn mark_dirty() {
    DIRTY_FLAG.with(|d| *d.borrow_mut() = true);
    with_registry_mut(|r| r.mark_current_dirty());
}

/// Read the cross-system dirty flag without touching it.
pub fn is_dirty() -> bool {
    DIRTY_FLAG.with(|d| *d.borrow())
}

/// Reset the cross-system dirty flag to false. Callers MUST also
/// re-mark the active scene dirty after loading fresh data, otherwise
/// the next preview frame will not rebuild.
pub fn clear_dirty() {
    DIRTY_FLAG.with(|d| *d.borrow_mut() = false);
    with_registry_mut(|r| r.clear_current_dirty());
}
