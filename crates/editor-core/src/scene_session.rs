//! `SceneSession` — encapsulates the four coupled invariants of the
//! editor's active scene state:
//!
//!   1. The active `SceneDocument` (loaded JSON).
//!   2. Its `OperationLog` (undo/redo bookkeeping).
//!   3. The `DIRTY_FLAG` that drives `rebuild_preview_world`.
//!   4. The `SceneRegistry` slot the active scene was swapped from/to.
//!
//! The thread-locals (SCENE_DOC, OPERATION_LOG, DIRTY_FLAG,
//! SCENE_REGISTRY) remain the storage mechanism in this revision; the
//! module only owns the *contract* and prevents external code from
//! mutating any one of the four without coordinating the others. Future
//! work (out of scope for Wave D3) may swap the storage for a single
//! owned `Rc<RefCell<SceneSession>>`; the public API is already shaped
//! to remain stable under that change.
//!
//! Pre-D3 the four were reachable directly through the re-exports in
//! `state.rs`, which made it possible to mutate the document without
//! marking the registry dirty, or to clear the operation log without
//! resetting the dirty flag, etc. The methods in this module are the
//! only path that maintains the cross-invariants correctly; any caller
//! that reaches into the thread-locals directly violates the contract
//! and is responsible for keeping the other three consistent.
//!
//! Mapping of public API to existing thread-local usage:
//!
//! | This module                | Replaces                                           |
//! | -------------------------- | -------------------------------------------------- |
//! | `with_active_doc`          | direct `SCENE_DOC.with` reads                       |
//! | `with_active_doc_mut`      | direct `SCENE_DOC.with` mut borrows                |
//! | `replace_active_doc`       | `SCENE_DOC.with(|s| *s.borrow_mut() = ...)`         |
//! | `clear_active_doc`         | `SCENE_DOC.with(|s| *s.borrow_mut() = None)`        |
//! | `with_log` / `with_log_mut`| direct `OPERATION_LOG.with` reads/mut              |
//! | `apply_command`            | `dispatch_command` body that does apply+record    |
//! | `undo`                     | `OPERATION_LOG.with(|l| l.borrow_mut().undo())`   |
//! | `redo`                     | `OPERATION_LOG.with(|l| l.borrow_mut().redo())`   |
//! | `mark_dirty` / `clear_dirty` / `is_dirty` | re-exports of `scene_state`  |
//! | `swap_scene`               | `perform_scene_swap` body                         |

use std::cell::RefCell;

use serde::{Deserialize, Serialize};

use crate::command::CommandEnvelope;
use crate::document::SceneDocument;
use crate::operation_log::OperationLog;
use crate::processor;
use crate::scene_state::{self, with_registry, with_registry_mut};

thread_local! {
    /// Active scene document. Always paired with the `OPERATION_LOG`
    /// below; use `with_active_doc` to read and `with_active_doc_mut`
    /// to mutate.
    pub(crate) static SCENE_DOC: RefCell<Option<SceneDocument>> = const { RefCell::new(None) };
    /// Operation log for the active scene. Always paired with
    /// `SCENE_DOC`; use `with_log` to read and `with_log_mut` to mutate.
    pub(crate) static OPERATION_LOG: RefCell<OperationLog> = const { RefCell::new(OperationLog::new_const()) };
}

/// Result of a `apply_command` call. Mirrors the `CommandResult` shape
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
    Processor(crate::command::CommandError),
    Serialize(String),
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoActiveDocument => write!(f, "no scene loaded"),
            Self::Processor(m) => write!(f, "processor: {m}"),
            Self::Serialize(m) => write!(f, "serialize: {m}"),
        }
    }
}

impl std::error::Error for ApplyError {}

/// Borrow the active `SceneDocument` immutably. The closure runs
/// with a `&SceneDocument`; if no scene is loaded the closure is
/// skipped and `None` is returned.
pub fn with_active_doc<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&SceneDocument) -> R,
{
    SCENE_DOC.with(|cell| cell.borrow().as_ref().map(f))
}

/// Borrow the active `SceneDocument` mutably. Used sparingly; most
/// mutations should go through `apply_command`, `replace_active_doc`,
/// or `swap_scene` so that the other invariants stay consistent.
pub fn with_active_doc_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut SceneDocument) -> R,
{
    SCENE_DOC.with(|cell| cell.borrow_mut().as_mut().map(f))
}

/// Borrow the active `OperationLog` immutably.
pub fn with_log<F, R>(f: F) -> R
where
    F: FnOnce(&OperationLog) -> R,
{
    OPERATION_LOG.with(|cell| f(&cell.borrow()))
}

/// Borrow the active `OperationLog` mutably.
pub fn with_log_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut OperationLog) -> R,
{
    OPERATION_LOG.with(|cell| f(&mut cell.borrow_mut()))
}

/// Replace the active document wholesale. Marks the scene dirty and
/// stores the document into the registry's current-id slot. Does NOT
/// touch the operation log (the caller is responsible for either
/// keeping it or resetting it; this is the contract used by
/// `load_scene_json`).
pub fn replace_active_doc(doc: SceneDocument) {
    let id = doc.scene_id.clone();
    with_registry_mut(|r| {
        r.store_to(&id, doc.clone(), OperationLog::new_const());
        r.set_current(Some(id));
    });
    SCENE_DOC.with(|cell| *cell.borrow_mut() = Some(doc));
    scene_state::mark_dirty();
}

/// Read the current scene ID as seen by the registry, after the
/// thread-locals settle. Useful in tests for asserting that
/// `swap_scene` advanced the active-id pointer.
pub fn current_scene_id() -> Option<String> {
    with_registry(|r| r.current_id())
}

/// Clear the active document and reset the operation log. The
/// registry is left untouched because the caller usually switches
/// into a different scene immediately after.
pub fn clear_active_doc() {
    SCENE_DOC.with(|cell| *cell.borrow_mut() = None);
    OPERATION_LOG.with(|cell| *cell.borrow_mut() = OperationLog::new_const());
}

/// Apply a command to the active document, record the inverse in the
/// operation log, and mark the scene dirty. This is the only path
/// through which `SCENE_DOC` and `OPERATION_LOG` should be mutated
/// together; the old `dispatch_command` impl in `lib.rs` is now a
/// thin wrapper.
pub fn apply_command(envelope: &CommandEnvelope) -> Result<ApplyResult, ApplyError> {
    let result = SCENE_DOC.with(|cell| {
        let mut doc_ref = cell.borrow_mut();
        let doc = doc_ref.as_mut().ok_or(ApplyError::NoActiveDocument)?;
        let inverse = processor::apply(doc, &envelope.command).map_err(ApplyError::Processor)?;
        OPERATION_LOG.with(|l| {
            l.borrow_mut().record(envelope, inverse.clone());
        });
        Ok(ApplyResult {
            inverse: Some(CommandEnvelope {
                command: inverse,
                metadata: envelope.metadata.clone(),
            }),
            snapshot: doc.clone(),
        })
    })?;
    scene_state::mark_dirty();
    Ok(result)
}

/// Undo the most recent command, if any. Returns the post-undo
/// snapshot of the document so callers can avoid a second read.
pub fn undo() -> Option<SceneDocument> {
    let result = SCENE_DOC.with(|cell| {
        let mut doc_ref = cell.borrow_mut();
        let doc = doc_ref.as_mut()?;
        match OPERATION_LOG.with(|l| l.borrow_mut().undo(doc)) {
            Ok(snap) => Some(snap),
            Err(_) => None,
        }
    });
    if result.is_some() {
        scene_state::mark_dirty();
    }
    result
}

/// Redo the next command, if any. Returns the post-redo snapshot.
pub fn redo() -> Option<SceneDocument> {
    let result = SCENE_DOC.with(|cell| {
        let mut doc_ref = cell.borrow_mut();
        let doc = doc_ref.as_mut()?;
        match OPERATION_LOG.with(|l| l.borrow_mut().redo(doc)) {
            Ok(snap) => Some(snap),
            Err(_) => None,
        }
    });
    if result.is_some() {
        scene_state::mark_dirty();
    }
    result
}

/// Snapshot of the active document, or `None` if no scene is loaded.
pub fn snapshot_active_doc() -> Option<SceneDocument> {
    SCENE_DOC.with(|cell| cell.borrow().clone())
}

/// The number of operations in the log, the current cursor, and
/// whether undo/redo are available. Mirrors the JSON returned by
/// `get_log_state` so the WASM binding can stay unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStateSnapshot {
    pub size: usize,
    pub cursor: isize,
    pub can_undo: bool,
    pub can_redo: bool,
}

pub fn log_state_snapshot() -> LogStateSnapshot {
    with_log(|log| LogStateSnapshot {
        size: log.get_log_size(),
        cursor: log.get_cursor(),
        can_undo: log.can_undo(),
        can_redo: log.can_redo(),
    })
}

/// Persist the active document and log into the registry's slot for
/// `old_id` and load whatever is in `new_id` (or an empty scratch
/// doc if the slot is empty) into the active thread-locals.
///
/// This is the contract used by `perform_scene_swap` and by the
/// scene-rename-and-switch path. The caller MUST hold no other
/// references into the operation log when calling this function
/// (we re-borrow the thread-locals internally).
pub fn swap_scene(old_id: &str, new_id: &str) {
    let doc_opt = SCENE_DOC.with(|cell| cell.borrow().clone());
    let log = OPERATION_LOG.with(|cell| cell.borrow().clone());

    let (doc, log) = match doc_opt {
        Some(doc) => (doc, log),
        None => (
            SceneDocument {
                version: "0.1".to_string(),
                scene_id: format!("scratch-{}", crate::time::now_nanos()),
                name: old_id.to_string(),
                entities: Vec::new(),
                instances: std::collections::BTreeMap::new(),
            },
            OperationLog::new_const(),
        ),
    };

    with_registry_mut(|r| {
        r.store_to(old_id, doc, log);
        // Always advance current_id to the new scene so subsequent
        // operations address the new entry. The pre-D3 implementation
        // left current_id on old_id here, which leaked the old id
        // into the dirty-flag path.
        r.set_current(Some(new_id.to_string()));
    });

    // Take the swap_in result out of the borrow first, then move it
    // into SCENE_DOC. The pre-D3 implementation was correct here, but
    // we now require the result to be detached before the SCENE_DOC
    // borrow so the test framework can see the post-swap state.
    let new_pair = with_registry(|r| r.swap_in(new_id));
    if let Some((new_doc, new_log)) = new_pair {
        SCENE_DOC.with(|cell| *cell.borrow_mut() = Some(new_doc));
        OPERATION_LOG.with(|cell| *cell.borrow_mut() = new_log);
    } else {
        // The new slot is empty — fall back to an empty doc so the
        // editor never ends up with `SCENE_DOC = None` after a swap.
        let empty = SceneDocument {
            version: "0.1".to_string(),
            scene_id: new_id.to_string(),
            name: new_id.to_string(),
            entities: Vec::new(),
            instances: std::collections::BTreeMap::new(),
        };
        with_registry_mut(|r| {
            r.store_to(new_id, empty.clone(), OperationLog::new_const());
        });
        SCENE_DOC.with(|cell| *cell.borrow_mut() = Some(empty));
        OPERATION_LOG.with(|cell| *cell.borrow_mut() = OperationLog::new_const());
    }

    scene_state::mark_dirty();
}

/// Replace the active document with a freshly built empty doc keyed
/// to `id`. Used after the active scene is deleted so the editor
/// does not leave a stale `Some(old_doc)` in the thread-local.
pub fn replace_with_empty(id: &str) {
    let current_id = with_registry(|r| r.current_id());
    let log = OperationLog::new_const();
    let doc = SceneDocument {
        version: "0.1".to_string(),
        scene_id: id.to_string(),
        name: id.to_string(),
        entities: Vec::new(),
        instances: std::collections::BTreeMap::new(),
    };

    with_registry_mut(|r| r.store_to(id, doc.clone(), log.clone()));

    if current_id.as_deref() == Some(id) {
        SCENE_DOC.with(|cell| *cell.borrow_mut() = Some(doc));
        OPERATION_LOG.with(|cell| *cell.borrow_mut() = log);
    }

    with_registry_mut(|r| r.clear_current_dirty());
    scene_state::mark_dirty();
}

/// Re-export the dirty-flag accessors so the four coupled invariants
/// can be touched through one namespace.
pub use scene_state::{clear_dirty, is_dirty, mark_dirty};
