//! `SceneSession` — facade over `EditorSession.active_scene` (H2.3).
//!
//! Replaces the four coupled invariants that previously lived as thread_locals
//! in `editor-bevy` (`SCENE_DOC`, `OPERATION_LOG`, `DIRTY_FLAG`, plus the
//! `SceneRegistry` slot swap). The four invariants are now collapsed into
//! the `SceneFocus` ADT (H2.3) living on `EditorSession.active_scene`, which
//! is accessed through `EditorSessionPort::active_scene_mut()`.
//!
//! ## Mapping
//!
//! | This module                | Replaces (H2.3)                                  |
//! | -------------------------- | ------------------------------------------------ |
//! | `with_active_doc`          | `session.active_scene_mut().doc()`               |
//! | `with_active_doc_mut`      | `session.active_scene_mut().doc_mut()`           |
//! | `replace_active_doc`       | `focus.focus(new_doc)`                           |
//! | `clear_active_doc`         | `focus.unfocus()`                                |
//! | `with_log` / `with_log_mut`| `focus.log()` / `focus.log_mut()`                |
//! | `apply_command`            | `focus.apply(env, &ProcessorApply)`              |
//! | `undo`                     | `focus.undo(&ProcessorApply)`                    |
//! | `redo`                     | `focus.redo(&ProcessorApply)`                    |
//! | `mark_dirty` / `clear_dirty` / `is_dirty` | re-exports of `scene_state`     |
//! | `swap_scene`               | `perform_scene_swap` body (still uses registry)  |
//!
//! ## Re-entrancy safe apply_command
//!
//! `apply_command` uses a take/write-back pattern: the focus is extracted
//! from the session sub-state, the session lock is released, `ProcessorApply`
//! runs (which may trigger rebuilds that re-acquire the session lock), then
//! the mutated focus is written back. This prevents deadlock when the
//! processor triggers a preview rebuild that itself calls back into the
//! session.

use serde::{Deserialize, Serialize};

use crate::document::SceneDocument;
use crate::operation_log::{OperationLog, ProcessorApply};
use crate::scene_state::{self, with_registry, with_registry_mut};
use editor_model::SceneFocus;
use editor_model::command::CommandEnvelope;
use editor_model::ports;
use editor_model::scene_focus::SceneFocusError;

/// Key used to store/retrieve the "active" scene path in EditorSession.
/// The active scene path is set by `activate_document` in EditorSession
/// and read here to look up the correct per-path sub-state.
pub const ACTIVE_SCENE_PATH: &str = "_active";

/// Run a closure with mutable access to the global `SceneFocus` (the
/// `EditorSession.active_scene` field).
///
/// Returns `None` if the session is not yet registered or has been poisoned.
/// The closure's return value is passed through.
fn with_focus_mut<R, F: FnOnce(&mut SceneFocus) -> R>(f: F) -> Option<R> {
    ports::with_session_mut(|session| f(session.active_scene_mut()))
}

/// Result of an `apply_command` call. Mirrors the `CommandResult` shape
/// returned by `dispatch_command` so callers can stay unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub inverse: Option<CommandEnvelope>,
    pub snapshot: SceneDocument,
}

/// Error type for `apply_command` so the function does not have to
/// return a JSON string at the seam.
#[derive(Debug)]
pub enum ApplyError {
    NoActiveDocument,
    SessionUnavailable,
    Processor(editor_model::command::CommandError),
    Log(editor_model::operation_log::OperationLogError),
    Focus(SceneFocusError),
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoActiveDocument => write!(f, "no scene loaded"),
            Self::SessionUnavailable => write!(f, "editor session unavailable"),
            Self::Processor(m) => write!(f, "processor: {m}"),
            Self::Log(m) => write!(f, "operation log: {m}"),
            Self::Focus(m) => write!(f, "focus: {m}"),
        }
    }
}

impl std::error::Error for ApplyError {}

/// Borrow the active `SceneDocument` immutably.
pub fn with_active_doc<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&SceneDocument) -> R,
{
    with_focus_mut(|focus| focus.doc().map(f)).flatten()
}

/// Borrow the active `SceneDocument` mutably.
///
/// Used sparingly; most mutations should go through `apply_command`,
/// `replace_active_doc`, or `swap_scene` so that the other invariants
/// stay consistent.
pub fn with_active_doc_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut SceneDocument) -> R,
{
    with_focus_mut(|focus| focus.doc_mut().map(f)).flatten()
}

/// Borrow the active `OperationLog` immutably.
pub fn with_log<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&OperationLog) -> R,
{
    with_focus_mut(|focus| focus.log().map(f)).flatten()
}

/// Borrow the active `OperationLog` mutably.
pub fn with_log_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut OperationLog) -> R,
{
    with_focus_mut(|focus| focus.log_mut().map(f)).flatten()
}

/// Replace the active document wholesale. Marks the scene dirty via
/// `SceneFocus::replace_doc`. Does NOT touch the operation log (the caller
/// is responsible for either keeping it or resetting it; this is the contract
/// used by `load_scene_json`).
pub fn replace_active_doc(doc: SceneDocument) {
    let id = doc.scene_id.clone();
    with_registry_mut(|r| {
        r.store_to(&id, doc.clone(), OperationLog::new_const());
        r.set_current(Some(id));
    });
    if let Some(focus) = with_focus_mut(|f| {
        if f.is_empty() {
            f.focus(doc.clone());
            Some(())
        } else {
            f.replace_doc(doc.clone()).ok()
        }
    })
    .flatten()
    {
        let _ = focus;
    }
    scene_state::mark_dirty();
}

/// Read the current scene ID as seen by the registry, after the focus settles.
/// Useful in tests for asserting that `swap_scene` advanced the active-id pointer.
pub fn current_scene_id() -> Option<String> {
    with_registry(|r| r.current_id())
}

/// Clear the active focus. The registry is left untouched because the caller
/// usually switches into a different scene immediately after.
pub fn clear_active_doc() {
    with_focus_mut(|f| f.unfocus());
}

/// Apply a command to the focused scene, record the inverse in the
/// operation log, and mark the scene dirty.
///
/// ## Re-entrancy safe (H2.3)
///
/// Uses a take/write-back pattern: the `SceneFocus` is extracted from the
/// session sub-state via `EditorSessionPort::active_scene_mut()` BEFORE the
/// processor runs, releasing the session lock for the duration of the call.
/// This prevents deadlock if `ProcessorApply` (or any code it calls) needs
/// to re-acquire the session lock for nested operations.
pub fn apply_command(envelope: &CommandEnvelope) -> Result<ApplyResult, ApplyError> {
    let mut session = ports::with_session_mut(|_| ()).ok_or(ApplyError::SessionUnavailable)?;
    // Phase 1: extract focus from the session (drops the Mutex guard).
    let mut focus =
        ports::with_session_mut(|s| std::mem::replace(s.active_scene_mut(), SceneFocus::Empty))
            .ok_or(ApplyError::SessionUnavailable)?;

    if focus.is_empty() {
        // Restore Empty and return error.
        ports::with_session_mut(|s| *s.active_scene_mut() = SceneFocus::Empty);
        return Err(ApplyError::NoActiveDocument);
    }

    // Phase 2: apply the command. The session Mutex is NOT held during
    // this call (we already took the focus out).
    let outcome = match focus.apply(envelope, &ProcessorApply) {
        Ok(o) => o,
        Err(e) => {
            // Restore focus on error.
            ports::with_session_mut(|s| *s.active_scene_mut() = focus);
            return Err(match e {
                SceneFocusError::Apply(p) => ApplyError::Processor(p),
                SceneFocusError::Log(l) => ApplyError::Log(l),
                SceneFocusError::NoActiveScene => ApplyError::NoActiveDocument,
            });
        }
    };

    // Phase 3: write the mutated focus back.
    ports::with_session_mut(|s| *s.active_scene_mut() = focus);

    scene_state::mark_dirty();
    Ok(ApplyResult {
        inverse: outcome.inverse,
        snapshot: outcome.snapshot,
    })
}

/// Undo the most recent command in the focused scene's log.
pub fn undo() -> Option<SceneDocument> {
    let mut focus =
        ports::with_session_mut(|s| std::mem::replace(s.active_scene_mut(), SceneFocus::Empty))?;
    if focus.is_empty() {
        ports::with_session_mut(|s| *s.active_scene_mut() = SceneFocus::Empty);
        return None;
    }
    let snapshot = focus.undo(&ProcessorApply).ok()?;
    ports::with_session_mut(|s| *s.active_scene_mut() = focus);
    scene_state::mark_dirty();
    Some(snapshot)
}

/// Redo the next command in the focused scene's log.
pub fn redo() -> Option<SceneDocument> {
    let mut focus =
        ports::with_session_mut(|s| std::mem::replace(s.active_scene_mut(), SceneFocus::Empty))?;
    if focus.is_empty() {
        ports::with_session_mut(|s| *s.active_scene_mut() = SceneFocus::Empty);
        return None;
    }
    let snapshot = focus.redo(&ProcessorApply).ok()?;
    ports::with_session_mut(|s| *s.active_scene_mut() = focus);
    scene_state::mark_dirty();
    Some(snapshot)
}

/// Snapshot of the active document, or `None` if no scene is loaded.
pub fn snapshot_active_doc() -> Option<SceneDocument> {
    with_focus_mut(|f| f.doc().cloned()).flatten()
}

/// The number of operations in the log, the current cursor, and whether
/// undo/redo are available. Mirrors the JSON returned by `get_log_state`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStateSnapshot {
    pub size: usize,
    pub cursor: isize,
    pub can_undo: bool,
    pub can_redo: bool,
}

pub fn log_state_snapshot() -> LogStateSnapshot {
    with_focus_mut(|f| {
        f.log().map(|log| LogStateSnapshot {
            size: log.get_log_size(),
            cursor: log.get_cursor(),
            can_undo: log.can_undo(),
            can_redo: log.can_redo(),
        })
    })
    .flatten()
    .unwrap_or(LogStateSnapshot {
        size: 0,
        cursor: -1,
        can_undo: false,
        can_redo: false,
    })
}

/// Persist the active document and log into the registry's slot for
/// `old_id` and load whatever is in `new_id` (or an empty scratch doc if
/// the slot is empty) into the active focus.
pub fn swap_scene(old_id: &str, new_id: &str) {
    // Phase 1: take the current focus and stash into the registry's old_id slot.
    let taken = ports::with_session_mut(|s| {
        std::mem::replace(s.active_scene_mut(), SceneFocus::Empty).take()
    })
    .flatten();
    let (doc, log) = match taken {
        Some((d, l)) => (d, l),
        None => (
            SceneDocument {
                version: "0.1".to_string(),
                scene_id: format!("scratch-{}", crate::time::now_nanos()),
                name: old_id.to_string(),
                entities: Vec::new(),
                instances: std::collections::BTreeMap::new(),
                extension_data: Default::default(),
            },
            OperationLog::new_const(),
        ),
    };

    with_registry_mut(|r| {
        r.store_to(old_id, doc, log);
        r.set_current(Some(new_id.to_string()));
    });

    // Phase 2: load whatever is in the new_id slot (or an empty doc).
    let new_pair = with_registry(|r| r.swap_in(new_id));
    if let Some((new_doc, new_log)) = new_pair {
        ports::with_session_mut(|s| s.active_scene_mut().focus_with_log(new_doc, new_log));
    } else {
        let empty = SceneDocument {
            version: "0.1".to_string(),
            scene_id: new_id.to_string(),
            name: new_id.to_string(),
            entities: Vec::new(),
            instances: std::collections::BTreeMap::new(),
            extension_data: Default::default(),
        };
        with_registry_mut(|r| {
            r.store_to(new_id, empty.clone(), OperationLog::new_const());
        });
        ports::with_session_mut(|s| {
            s.active_scene_mut().focus(empty);
        });
    }

    scene_state::mark_dirty();
}

/// Replace the active document with a freshly built empty doc keyed to `id`.
/// Used after the active scene is deleted so the editor does not leave a
/// stale focused scene.
pub fn replace_with_empty(id: &str) {
    let current_id = with_registry(|r| r.current_id());
    let log = OperationLog::new_const();
    let doc = SceneDocument {
        version: "0.1".to_string(),
        scene_id: id.to_string(),
        name: id.to_string(),
        entities: Vec::new(),
        instances: std::collections::BTreeMap::new(),
        extension_data: Default::default(),
    };

    with_registry_mut(|r| r.store_to(id, doc.clone(), log.clone()));

    if current_id.as_deref() == Some(id) {
        ports::with_session_mut(|s| {
            s.active_scene_mut().focus_with_log(doc, log);
        });
    }

    with_registry_mut(|r| r.clear_current_dirty());
    scene_state::mark_dirty();
}

/// Re-export the dirty-flag accessors so the four coupled invariants
/// can be touched through one namespace.
pub use scene_state::{clear_dirty, is_dirty, mark_dirty};
