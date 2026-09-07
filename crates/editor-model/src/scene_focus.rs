//! `SceneFocus` — the ADT for the editor's currently-focused scene.
//!
//! Replaces the three coupled thread-locals that used to live in `editor-bevy`
//! (`SCENE_DOC`, `OPERATION_LOG`, `DIRTY_FLAG`) and were responsible for
//! keeping the active document, its undo log, and the dirty flag in sync
//! (H2.3, ADR-0031 invariant).
//!
//! ## Shape (H2.3, hexagonal + ADT + FP)
//!
//! ```text
//! pub enum SceneFocus {
//!     Empty,
//!     Focused { doc, log, dirty },
//! }
//! ```
//!
//! - **Hexagonal**: `SceneFocus` is a value type with no Bevy or application
//!   dependencies. Its mutation API takes the apply callback as a parameter
//!   (`&dyn ApplyCommandFn`) so the type never imports the Bevy processor.
//! - **ADT**: `Empty | Focused` makes "no scene loaded" an explicit, named
//!   state. The previous `Option<SceneDocument>` + separate `Option<OperationLog>`
//!   + separate `bool` allowed invalid combinations; the ADT does not.
//! - **FP**: `SceneFocus` is a value that callers transform via pure
//!   functions. `apply`, `undo`, `redo`, `mark_saved` are pure functions of
//!   `(self, env, callback) -> Result<Self, Error>`. `dirty` is updated
//!   automatically by these operations (no manual flag-flipping scattered
//!   across the codebase).
//!
//! ## Why a singleton (not a `BTreeMap`-keyed slot)
//!
//! The companion `EditorSession.scene_states: BTreeMap<Path, SceneSessionState>`
//! (added by H2.1) holds *cached* per-path state for fast switching. `active_scene`
//! is a separate, orthogonal concept: "the scene that is currently focused in
//! the editor UI". Modeling it as a separate ADT field avoids the dual-source-
//! of-truth problem of "what is the active scene path?" being independent of
//! "what document is currently focused?".

#![allow(missing_docs)]

use crate::command::{Command, CommandEnvelope, CommandError};
use crate::document::SceneDocument;
use crate::operation_log::{ApplyCommandFn, OperationLog, OperationLogError};
use serde::{Deserialize, Serialize};

/// Outcome of an [`SceneFocus::apply`] call.
///
/// Mirrors the legacy `ApplyResult` from `editor-bevy::scene_session` so the
/// WASM bindings can stay unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyOutcome {
    /// The inverse of the applied command (if any), wrapped in a fresh envelope.
    /// `None` if the command had no inverse.
    pub inverse: Option<CommandEnvelope>,
    /// A snapshot of the document state after the command was applied.
    pub snapshot: SceneDocument,
}

/// Error type for [`SceneFocus`] operations.
#[derive(Debug)]
pub enum SceneFocusError {
    /// Operation requested on an empty focus (no scene loaded).
    NoActiveScene,
    /// The apply callback returned an error.
    Apply(CommandError),
    /// The operation log returned an error during undo/redo.
    Log(OperationLogError),
}

impl std::fmt::Display for SceneFocusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoActiveScene => write!(f, "no scene is currently focused"),
            Self::Apply(e) => write!(f, "apply callback failed: {e}"),
            Self::Log(e) => write!(f, "operation log failed: {e}"),
        }
    }
}

impl std::error::Error for SceneFocusError {}

impl From<OperationLogError> for SceneFocusError {
    fn from(e: OperationLogError) -> Self {
        Self::Log(e)
    }
}

/// ADT for the editor's currently-focused scene.
///
/// Invariants (enforced by the ADT shape itself):
/// - `Empty` means "no document is loaded; no undo history exists; nothing is
///   dirty".
/// - `Focused { doc, log, dirty }` means "a scene is loaded with this document,
///   this undo history, and this dirty flag". The three fields are always
///   co-present.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum SceneFocus {
    /// No scene is focused.
    #[default]
    Empty,
    /// A scene is focused. Carries the document, its undo log, and the dirty flag.
    Focused {
        /// The active scene document (the source of truth for the scene's contents).
        doc: SceneDocument,
        /// The undo/redo history for `doc`.
        log: OperationLog,
        /// `true` if `doc` has unsaved changes since the last `mark_saved`.
        dirty: bool,
    },
}

impl SceneFocus {
    /// Returns `true` if no scene is currently focused.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns `true` if a scene is focused.
    pub fn is_focused(&self) -> bool {
        matches!(self, Self::Focused { .. })
    }

    /// Returns `Some(&doc)` if a scene is focused, otherwise `None`.
    pub fn doc(&self) -> Option<&SceneDocument> {
        match self {
            Self::Empty => None,
            Self::Focused { doc, .. } => Some(doc),
        }
    }

    /// Returns `Some(&mut doc)` if a scene is focused, otherwise `None`.
    pub fn doc_mut(&mut self) -> Option<&mut SceneDocument> {
        match self {
            Self::Empty => None,
            Self::Focused { doc, .. } => Some(doc),
        }
    }

    /// Returns `Some(&log)` if a scene is focused, otherwise `None`.
    pub fn log(&self) -> Option<&OperationLog> {
        match self {
            Self::Empty => None,
            Self::Focused { log, .. } => Some(log),
        }
    }

    /// Returns `Some(&mut log)` if a scene is focused, otherwise `None`.
    pub fn log_mut(&mut self) -> Option<&mut OperationLog> {
        match self {
            Self::Empty => None,
            Self::Focused { log, .. } => Some(log),
        }
    }

    /// Returns `Some(dirty)` if a scene is focused, otherwise `None`.
    pub fn dirty(&self) -> Option<bool> {
        match self {
            Self::Empty => None,
            Self::Focused { dirty, .. } => Some(*dirty),
        }
    }

    /// Focus on a fresh scene document with an empty undo log.
    ///
    /// Replaces whatever was previously focused. The new focus starts clean
    /// (`dirty = false`) because the document has just been loaded.
    pub fn focus(&mut self, doc: SceneDocument) {
        *self = Self::Focused {
            doc,
            log: OperationLog::new(),
            dirty: false,
        };
    }

    /// Focus on a scene document with a pre-existing undo log (e.g. when
    /// switching to a scene that was previously loaded and edited).
    pub fn focus_with_log(&mut self, doc: SceneDocument, log: OperationLog) {
        *self = Self::Focused {
            doc,
            log,
            dirty: false,
        };
    }

    /// Drop the current focus, returning to `Empty`.
    pub fn unfocus(&mut self) {
        *self = Self::Empty;
    }

    /// Apply a command to the focused scene document, record its inverse in
    /// the operation log, and mark the scene dirty.
    ///
    /// Returns `Err(NoActiveScene)` if no scene is focused.
    pub fn apply(
        &mut self,
        envelope: &CommandEnvelope,
        apply_fn: &dyn ApplyCommandFn,
    ) -> Result<ApplyOutcome, SceneFocusError> {
        let focused = match self {
            Self::Empty => return Err(SceneFocusError::NoActiveScene),
            Self::Focused { doc, log, dirty } => {
                // Phase 1: apply via callback. The callback may re-enter the
                // session through the port; the take/write-back dance inside
                // editor-bevy ensures the RefCell borrows are released for the
                // duration of this call.
                let inverse = apply_fn
                    .apply(doc, &envelope.command)
                    .map_err(SceneFocusError::Apply)?;
                // Phase 2: record in log.
                log.record(envelope, inverse.clone());
                // Phase 3: mark dirty + capture snapshot.
                *dirty = true;
                (doc.clone(), inverse)
            }
        };
        let (snapshot, inverse) = focused;
        Ok(ApplyOutcome {
            inverse: Some(CommandEnvelope {
                command: inverse,
                metadata: envelope.metadata.clone(),
            }),
            snapshot,
        })
    }

    /// Undo the most recent command in the focused scene's log.
    ///
    /// Returns the post-undo document snapshot. Marks the scene dirty
    /// (undoing an operation also counts as an unsaved change).
    /// Returns `Err(NoActiveScene)` if no scene is focused.
    pub fn undo(
        &mut self,
        apply_fn: &dyn ApplyCommandFn,
    ) -> Result<SceneDocument, SceneFocusError> {
        match self {
            Self::Empty => Err(SceneFocusError::NoActiveScene),
            Self::Focused { doc, log, dirty } => {
                let snapshot = log.undo(doc, apply_fn)?;
                *dirty = true;
                Ok(snapshot)
            }
        }
    }

    /// Redo the next command in the focused scene's log.
    ///
    /// Returns the post-redo document snapshot. Marks the scene dirty.
    /// Returns `Err(NoActiveScene)` if no scene is focused.
    pub fn redo(
        &mut self,
        apply_fn: &dyn ApplyCommandFn,
    ) -> Result<SceneDocument, SceneFocusError> {
        match self {
            Self::Empty => Err(SceneFocusError::NoActiveScene),
            Self::Focused { doc, log, dirty } => {
                let snapshot = log.redo(doc, apply_fn)?;
                *dirty = true;
                Ok(snapshot)
            }
        }
    }

    /// Mark the focused scene as saved (clean). Idempotent.
    ///
    /// Returns `Err(NoActiveScene)` if no scene is focused.
    pub fn mark_saved(&mut self) -> Result<(), SceneFocusError> {
        match self {
            Self::Empty => Err(SceneFocusError::NoActiveScene),
            Self::Focused { dirty, .. } => {
                *dirty = false;
                Ok(())
            }
        }
    }

    /// Replace the focused scene's document wholesale. Preserves the undo log
    /// (the caller is responsible for resetting it via `focus_with_log` if
    /// they want a fresh history). Marks the scene dirty because the document
    /// has changed without being recorded as an undo-able command.
    ///
    /// Returns `Err(NoActiveScene)` if no scene is focused.
    pub fn replace_doc(&mut self, new_doc: SceneDocument) -> Result<(), SceneFocusError> {
        match self {
            Self::Empty => Err(SceneFocusError::NoActiveScene),
            Self::Focused { doc, dirty, .. } => {
                *doc = new_doc;
                *dirty = true;
                Ok(())
            }
        }
    }

    /// Take the focused scene's document and log out of the focus, leaving
    /// `Empty` in its place. Used by scene-swap paths that need to move the
    /// current focus into the per-path cache.
    ///
    /// Returns `None` if the focus is `Empty`.
    pub fn take(&mut self) -> Option<(SceneDocument, OperationLog)> {
        match std::mem::take(self) {
            Self::Empty => None,
            Self::Focused { doc, log, dirty: _ } => Some((doc, log)),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{CommandMetadata, CommandResult};
    use crate::ids::{LocalId, StableId};
    use serde_json::json;
    use std::collections::BTreeMap;

    /// Minimal apply callback that handles CreateEntity / DeleteEntity /
    /// SetComponentField. Mirrors the test stub in `operation_log::tests` but
    /// kept independent so SceneFocus tests don't depend on that module.
    struct TestApply;

    impl ApplyCommandFn for TestApply {
        fn apply(&self, doc: &mut SceneDocument, cmd: &Command) -> Result<Command, CommandError> {
            match cmd {
                Command::CreateEntity {
                    id,
                    name,
                    components,
                } => {
                    doc.entities.push(crate::document::Entity {
                        id: id.clone(),
                        local_id: LocalId::new(id.as_str()),
                        name: name.clone(),
                        parent: None,
                        components: components.clone(),
                        extension_data: BTreeMap::new(),
                    });
                    Ok(Command::DeleteEntity { id: id.clone() })
                }
                Command::DeleteEntity { id } => {
                    doc.entities.retain(|e| e.id != *id);
                    Ok(Command::Noop {})
                }
                Command::SetComponentField {
                    entity_id,
                    type_id,
                    field_path,
                    value,
                } => {
                    if let Some(e) = doc.entities.iter_mut().find(|e| e.id == *entity_id) {
                        if let Some(c) = e.components.iter_mut().find(|c| c.type_id == *type_id) {
                            set_dotted(&mut c.values, field_path, value.clone());
                        }
                    }
                    Ok(Command::Noop {})
                }
                _ => Ok(Command::Noop {}),
            }
        }
    }

    fn set_dotted(root: &mut serde_json::Value, path: &str, value: serde_json::Value) {
        let mut cur = root;
        for seg in path.split('.') {
            match cur {
                serde_json::Value::Object(map) => {
                    cur = map
                        .entry(seg.to_string())
                        .or_insert(serde_json::Value::Null);
                }
                _ => return,
            }
        }
        *cur = value;
    }

    fn empty_doc() -> SceneDocument {
        SceneDocument {
            version: "0.1".to_string(),
            scene_id: "test".to_string(),
            name: "Test".to_string(),
            entities: vec![],
            instances: BTreeMap::new(),
            extension_data: BTreeMap::new(),
        }
    }

    fn envelope(cmd: Command) -> CommandEnvelope {
        CommandEnvelope {
            command: cmd,
            metadata: CommandMetadata::now("test"),
        }
    }

    #[test]
    fn default_is_empty() {
        let f = SceneFocus::default();
        assert!(f.is_empty());
        assert!(!f.is_focused());
        assert!(f.doc().is_none());
        assert!(f.log().is_none());
        assert!(f.dirty().is_none());
    }

    #[test]
    fn focus_unfocus_roundtrip() {
        let mut f = SceneFocus::default();
        assert!(f.is_empty());

        f.focus(empty_doc());
        assert!(f.is_focused());
        assert!(!f.is_empty());
        assert_eq!(f.dirty(), Some(false));
        assert_eq!(f.doc().map(|d| d.scene_id.as_str()), Some("test"));
        assert_eq!(f.log().map(|l| l.get_log_size()), Some(0));

        f.unfocus();
        assert!(f.is_empty());
    }

    #[test]
    fn focus_with_log_preserves_history() {
        let mut f = SceneFocus::default();
        let mut log = OperationLog::new();
        let cmd = Command::CreateEntity {
            id: StableId::new("e1"),
            name: "E1".to_string(),
            components: vec![],
        };
        log.record(
            &envelope(cmd.clone()),
            Command::DeleteEntity {
                id: StableId::new("e1"),
            },
        );

        f.focus_with_log(empty_doc(), log);
        assert_eq!(f.log().unwrap().get_log_size(), 1);
        assert!(f.log().unwrap().can_undo());
    }

    #[test]
    fn apply_marks_dirty() {
        let mut f = SceneFocus::default();
        f.focus(empty_doc());
        assert_eq!(f.dirty(), Some(false));

        let cmd = Command::CreateEntity {
            id: StableId::new("e1"),
            name: "E1".to_string(),
            components: vec![],
        };
        f.apply(&envelope(cmd), &TestApply).unwrap();
        assert_eq!(f.dirty(), Some(true));
        assert_eq!(f.doc().unwrap().entities.len(), 1);
    }

    #[test]
    fn apply_on_empty_returns_error() {
        let mut f = SceneFocus::default();
        let cmd = Command::Noop {};
        let err = f.apply(&envelope(cmd), &TestApply).unwrap_err();
        assert!(matches!(err, SceneFocusError::NoActiveScene));
    }

    #[test]
    fn mark_saved_clears_dirty() {
        let mut f = SceneFocus::default();
        f.focus(empty_doc());

        let cmd = Command::CreateEntity {
            id: StableId::new("e1"),
            name: "E1".to_string(),
            components: vec![],
        };
        f.apply(&envelope(cmd), &TestApply).unwrap();
        assert_eq!(f.dirty(), Some(true));

        f.mark_saved().unwrap();
        assert_eq!(f.dirty(), Some(false));
    }

    #[test]
    fn mark_saved_on_empty_returns_error() {
        let mut f = SceneFocus::default();
        assert!(matches!(
            f.mark_saved().unwrap_err(),
            SceneFocusError::NoActiveScene
        ));
    }

    #[test]
    fn undo_redo_via_focus() {
        let mut f = SceneFocus::default();
        f.focus(empty_doc());

        let cmd = Command::CreateEntity {
            id: StableId::new("e1"),
            name: "E1".to_string(),
            components: vec![],
        };
        f.apply(&envelope(cmd), &TestApply).unwrap();
        assert_eq!(f.doc().unwrap().entities.len(), 1);

        let snap = f.undo(&TestApply).unwrap();
        assert_eq!(snap.entities.len(), 0);
        assert_eq!(f.doc().unwrap().entities.len(), 0);
        assert_eq!(f.dirty(), Some(true));

        let snap = f.redo(&TestApply).unwrap();
        assert_eq!(snap.entities.len(), 1);
        assert_eq!(f.doc().unwrap().entities.len(), 1);
        assert_eq!(f.dirty(), Some(true));
    }

    #[test]
    fn replace_doc_marks_dirty_and_preserves_log() {
        let mut f = SceneFocus::default();
        f.focus(empty_doc());
        let initial_log_size = f.log().unwrap().get_log_size();

        let mut new_doc = empty_doc();
        new_doc.name = "Replaced".to_string();
        f.replace_doc(new_doc).unwrap();

        assert_eq!(f.doc().unwrap().name, "Replaced");
        assert_eq!(f.dirty(), Some(true));
        assert_eq!(f.log().unwrap().get_log_size(), initial_log_size);
    }

    #[test]
    fn take_drains_focus_to_empty() {
        let mut f = SceneFocus::default();
        f.focus(empty_doc());

        let (doc, log) = f.take().unwrap();
        assert_eq!(doc.scene_id, "test");
        assert_eq!(log.get_log_size(), 0);
        assert!(f.is_empty());

        assert!(f.take().is_none());
    }

    #[test]
    fn apply_outcome_serializes() {
        // Sanity check that the WASM-facing surface round-trips through JSON.
        let mut f = SceneFocus::default();
        f.focus(empty_doc());

        let cmd = Command::CreateEntity {
            id: StableId::new("e1"),
            name: "E1".to_string(),
            components: vec![],
        };
        let outcome = f.apply(&envelope(cmd), &TestApply).unwrap();
        let json = serde_json::to_string(&outcome).unwrap();
        let back: ApplyOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(back.snapshot.entities.len(), 1);
    }

    // Suppress dead-code warning for the unused CommandResult import in some
    // test configs.
    #[allow(dead_code)]
    fn _force_use(_: &CommandResult) {}
}
