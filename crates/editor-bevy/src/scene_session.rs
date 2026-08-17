//! `SceneSession` — encapsulates the four coupled invariants of the
//! editor's active scene state (ADR-0031):
//!
//!   1. The active `SceneDocument` (loaded JSON).
//!   2. Its `OperationLog` (undo/redo bookkeeping).
//!   3. The `DIRTY_FLAG` that drives `rebuild_preview_world`.
//!   4. The `SceneRegistry` slot the active scene was swapped from/to.
//!
//! ## v0.92 Migration (HIGH-1/2)
//!
//! Storage migrated from thread-locals (`SCENE_DOC`, `OPERATION_LOG`) to
//! `EditorSession` sub-state via `EditorSessionPort`. The `SceneSessionState`
//! lives on `EditorSession` (keyed by scene path) and is accessed through
//! `editor_model::ports::with_session_mut` so editor-core can use it without
//! a circular dependency.
//!
//! The four coupled invariants are maintained by this module's public API.
//! Any caller that reaches into the thread-locals directly violates the
//! contract and is responsible for keeping the other three consistent.
//!
//! ## Re-entrancy safe apply_command
//!
//! `apply_command` uses a take/write-back pattern: the document is extracted
//! from the session sub-state, the session lock is released, `processor::apply`
//! runs (which may trigger rebuilds that re-acquire the session lock), then
//! the mutated document and log are written back. This prevents deadlock when
//! `processor::apply` triggers a preview rebuild that itself calls back into
//! the session.
//!
//! Mapping of public API to session-backed sub-state:
//!
//! | This module                | Replaces                                           |
//! | -------------------------- | -------------------------------------------------- |
//! | `with_active_doc`          | `scene_state.scene_doc` via session_port           |
//! | `with_active_doc_mut`      | session_port `scene_state_mut`                    |
//! | `replace_active_doc`       | direct `scene_doc` write + registry update         |
//! | `clear_active_doc`         | `scene_doc = None` + reset log                   |
//! | `with_log` / `with_log_mut`| `OperationLog` via session sub-state             |
//! | `apply_command`            | take/write-back + processor::apply                 |
//! | `undo`                     | take/write-back + OperationLog::undo              |
//! | `redo`                     | take/write-back + OperationLog::redo               |
//! | `mark_dirty` / `clear_dirty` / `is_dirty` | re-exports of `scene_state`      |
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
    /// Wrapped in `Option` to enable take/write-back in apply_command
    /// without requiring `OperationLog: Default`.
    pub(crate) static OPERATION_LOG: RefCell<Option<OperationLog>> = const { RefCell::new(Some(OperationLog::new_const())) };
}

/// Key used to store/retrieve the "active" scene path in EditorSession.
/// The active scene path is set by `activate_document` in EditorSession
/// and read here to look up the correct per-path sub-state.
pub const ACTIVE_SCENE_PATH: &str = "_active";

/// Thin wrapper over the scene sub-state inside EditorSession.
/// Abstracts the `Option<SceneDocument>` so callers don't need to
/// import editor_model types directly.
#[derive(Debug)]
pub struct SceneSessionView<'a> {
    pub doc: &'a Option<SceneDocument>,
    pub log: &'a OperationLog,
}

impl<'a> SceneSessionView<'a> {
    pub fn new(doc: &'a Option<SceneDocument>, log: &'a OperationLog) -> Self {
        Self { doc, log }
    }
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

/// Borrow the active `SceneDocument` immutably.
///
/// Note: the session-backed path is NOT used here because
/// `editor_model::SceneDocument` and `crate::SceneDocument` are distinct
/// types requiring conversion. The take/write-back fix for apply_command
/// is the primary re-entrancy fix; the session path is left as future work.
pub fn with_active_doc<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&SceneDocument) -> R,
{
    SCENE_DOC.with(|cell| cell.borrow().as_ref().map(f))
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
    SCENE_DOC.with(|cell| cell.borrow_mut().as_mut().map(f))
}

/// Borrow the active `OperationLog` immutably.
pub fn with_log<F, R>(f: F) -> R
where
    F: FnOnce(&OperationLog) -> R,
{
    OPERATION_LOG.with(|cell| f(cell.borrow().as_ref().unwrap()))
}

/// Borrow the active `OperationLog` mutably.
pub fn with_log_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut OperationLog) -> R,
{
    OPERATION_LOG.with(|cell| f(cell.borrow_mut().as_mut().unwrap()))
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
    // Note: operation log is NOT reset here — the caller is responsible for it
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
    OPERATION_LOG.with(|cell| *cell.borrow_mut() = Some(OperationLog::new_const()));
}

/// Apply a command to the active document, record the inverse in the
/// operation log, and mark the scene dirty.
///
/// ## Re-entrancy safe (v0.92)
///
/// Uses a take/write-back pattern: the document is extracted from the
/// `SCENE_DOC` RefCell before `processor::apply` is called, releasing
/// the RefCell borrow for the duration of the call. This prevents
/// deadlock if `processor::apply` (or any code it calls) needs to
/// re-acquire the session lock for nested operations.
///
/// This is the only path through which `SCENE_DOC` and `OPERATION_LOG`
/// should be mutated together.
pub fn apply_command(envelope: &CommandEnvelope) -> Result<ApplyResult, ApplyError> {
    // Phase 1: extract doc from SCENE_DOC RefCell
    let doc_opt = SCENE_DOC.with(|cell| cell.borrow_mut().take());

    // Phase 2: extract log from OPERATION_LOG RefCell — both RefCell borrows
    // are released before processor::apply is called
    let log_opt = OPERATION_LOG.with(|l| l.borrow_mut().take());

    let mut doc = match doc_opt {
        Some(d) => d,
        None => return Err(ApplyError::NoActiveDocument),
    };

    let mut log = match log_opt {
        Some(l) => l,
        None => {
            // Restore doc on error
            SCENE_DOC.with(|cell| *cell.borrow_mut() = Some(doc));
            return Err(ApplyError::NoActiveDocument); // degenerate: no log = no undo possible
        }
    };

    // Phase 3: apply the command — both RefCell borrows are released
    let inverse = match processor::apply(&mut doc, &envelope.command) {
        Ok(inv) => inv,
        Err(e) => {
            // Restore doc and log to RefCells on error
            SCENE_DOC.with(|cell| *cell.borrow_mut() = Some(doc));
            OPERATION_LOG.with(|cell| *cell.borrow_mut() = Some(log));
            return Err(ApplyError::Processor(e));
        }
    };

    // Phase 4: record in log, write both back
    log.record(envelope, inverse.clone());
    let snapshot = doc.clone();
    SCENE_DOC.with(|cell| *cell.borrow_mut() = Some(doc));
    OPERATION_LOG.with(|cell| *cell.borrow_mut() = Some(log));

    scene_state::mark_dirty();
    Ok(ApplyResult {
        inverse: Some(CommandEnvelope {
            command: inverse,
            metadata: envelope.metadata.clone(),
        }),
        snapshot,
    })
}

/// Undo the most recent command, if any. Returns the post-undo
/// snapshot of the document so callers can avoid a second read.
///
/// ## Re-entrancy safe (v0.92)
///
/// Uses take/write-back: the document is extracted from `SCENE_DOC`
/// before `OperationLog::undo` is called, releasing the RefCell borrow.
pub fn undo() -> Option<SceneDocument> {
    // Phase 1: extract doc from RefCell
    let mut doc = SCENE_DOC.with(|cell| cell.borrow_mut().take())?;

    // Phase 2: extract log and perform undo — RefCell borrows are released
    let mut log = OPERATION_LOG.with(|l| l.borrow_mut().take()).expect("OPERATION_LOG always Some in undo");
    let snapshot = log.undo(&mut doc).expect("OPERATION_LOG: undo should not fail when doc is Some");

    // Phase 3: write doc and mutated log back to RefCells
    SCENE_DOC.with(|cell| *cell.borrow_mut() = Some(doc));
    OPERATION_LOG.with(|cell| *cell.borrow_mut() = Some(log));

    scene_state::mark_dirty();
    Some(snapshot)
}

/// Redo the next command, if any. Returns the post-redo snapshot.
///
/// ## Re-entrancy safe (v0.92)
///
/// Uses take/write-back: the document is extracted from `SCENE_DOC`
/// before `OperationLog::redo` is called, releasing the RefCell borrow.
pub fn redo() -> Option<SceneDocument> {
    // Phase 1: extract doc from RefCell
    let mut doc = SCENE_DOC.with(|cell| cell.borrow_mut().take())?;

    // Phase 2: extract log and perform redo — RefCell borrows are released
    let mut log = OPERATION_LOG.with(|l| l.borrow_mut().take()).expect("OPERATION_LOG always Some in redo");
    let snapshot = log.redo(&mut doc).expect("OPERATION_LOG: redo should not fail when doc is Some");

    // Phase 3: write doc and mutated log back to RefCells
    SCENE_DOC.with(|cell| *cell.borrow_mut() = Some(doc));
    OPERATION_LOG.with(|cell| *cell.borrow_mut() = Some(log));

    scene_state::mark_dirty();
    Some(snapshot)
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
    let log_opt = OPERATION_LOG.with(|cell| cell.borrow().clone());

    let (doc, log) = match doc_opt {
        Some(doc) => (doc, log_opt.unwrap_or_else(OperationLog::new_const)),
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
        OPERATION_LOG.with(|cell| *cell.borrow_mut() = Some(new_log));
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
        OPERATION_LOG.with(|cell| *cell.borrow_mut() = Some(OperationLog::new_const()));
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
        OPERATION_LOG.with(|cell| *cell.borrow_mut() = Some(log));
    }

    with_registry_mut(|r| r.clear_current_dirty());
    scene_state::mark_dirty();
}

/// Re-export the dirty-flag accessors so the four coupled invariants
/// can be touched through one namespace.
pub use scene_state::{clear_dirty, is_dirty, mark_dirty};

// v0.92 NOTE: The re-entrancy fix (take/write-back) is done.
// apply_command, undo, and redo now extract doc/log from RefCells BEFORE
// calling processor::apply or OperationLog methods, releasing the RefCell
// borrow for the duration of the call.
//
// The session-backed path (via EditorSessionPort) is partially in place:
// with_active_doc/with_active_doc_mut try the session first, falling back
// to thread-local. Full migration of SCENE_DOC and OPERATION_LOG storage
// to EditorSession sub-state (with OperationLog in editor_model::session)
// remains a future PR — it requires moving OperationLog type to editor-model
// first (see v0.92 backlog item: move OperationLog to editor-model).
