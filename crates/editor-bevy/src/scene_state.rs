//! HIGH-1 phase 2: scene state sub-module.
//!
//! Owns the `SceneRegistry` and exposes the cross-system dirty flag. The
//! `DIRTY_FLAG` itself was removed in H2.3: the dirty bit is now carried by
//! `EditorSession.active_scene` (`SceneFocus::Focused.dirty`) and accessed
//! through `EditorSessionPort::active_scene_mut()`.
//!
//! `SCENE_REGISTRY` is still a `thread_local!` here — registry access is a
//! tight, hot read path that doesn't warrant the port cell round-trip yet.
//! A future slice can move it into `EditorSession` if needed.

use std::cell::RefCell;

use crate::scenes::SceneRegistry;

thread_local! {
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

/// Run a closure with mutable access to the editor session's
/// `SceneFocus::Focused.dirty` flag (H2.3).
///
/// No-op if the session has not been registered yet (WASM startup has not
/// completed) or if no scene is currently focused. The closure receives
/// `true` if a focused scene exists, `false` otherwise.
fn with_dirty_mut<R, F: FnOnce(&mut bool) -> R>(f: F) -> Option<R> {
    editor_model::ports::with_session_mut(|session| {
        let focus = session.active_scene_mut();
        if let editor_model::SceneFocus::Focused { dirty, .. } = focus {
            Some(f(dirty))
        } else {
            None
        }
    })
    .flatten()
}

/// Mark the current scene as dirty (sets `EditorSession.active_scene.dirty =
/// true` + marks the registry's current scene dirty).
///
/// Triggers `rebuild_preview_world` on the next frame.
pub fn mark_dirty() {
    with_dirty_mut(|dirty| *dirty = true);
    with_registry_mut(|r| r.mark_current_dirty());
}

/// Read the cross-system dirty flag without touching it.
pub fn is_dirty() -> bool {
    with_dirty_mut(|dirty| *dirty).unwrap_or(false)
}

/// Reset the cross-system dirty flag to `false`. Callers MUST also re-mark
/// the active scene dirty after loading fresh data, otherwise the next
/// preview frame will not rebuild.
pub fn clear_dirty() {
    with_dirty_mut(|dirty| *dirty = false);
    with_registry_mut(|r| r.clear_current_dirty());
}
