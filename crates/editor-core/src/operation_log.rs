//! Operation Log for the Bevy 2D Editor.
//!
//! The canonical reversible history of typed editor commands, used for undo/redo
//! and future agent auditing (Hito 0 §6.4, CONTEXT.md).
//!
//! Design:
//! - `Vec<LogEntry>` with cursor index
//! - Cursor -1 = before start; 0..len-1 = at entry; cursor moves on apply/undo/redo
//! - New command after undo truncates the redo branch
//! - `Batch` is stored as one entry (gesture granularity)
//! - FIFO eviction when over max_size

use crate::command::{Command, CommandEnvelope, CommandMetadata};
use crate::document::{SceneDocument, StableId};
use crate::processor;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Single entry in the operation log: the forward command, its inverse, and metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    pub forward: Command,
    pub inverse: Command,
    pub metadata: CommandMetadata,
    /// Where the originating ChangeSet came from (ADR-0032 ChangeOrigin).
    /// Serialized as a plain string — "Human", "Agent", "Recipe", etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// Actor who authored the originating ChangeSet.
    /// Typically the same as `metadata.authorship`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    /// ID of the ChangeSet this entry belongs to (e.g. "cmd-1234567890").
    /// Used by the ChangeWorkbench to correlate log entries with ChangeSet summaries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_id: Option<String>,
}

impl LogEntry {
    pub fn new(
        forward: Command,
        inverse: Command,
        metadata: CommandMetadata,
        origin: Option<String>,
        actor: Option<String>,
        change_id: Option<String>,
    ) -> Self {
        Self {
            forward,
            inverse,
            metadata,
            origin,
            actor,
            change_id,
        }
    }
}

/// Default maximum history size (number of entries).
pub const DEFAULT_MAX_LOG_SIZE: usize = 1000;

/// Cursor sentinel meaning "before start, all entries are redo-able".
const CURSOR_BEFORE_START: isize = -1;

/// Append-only history with cursor-based undo/redo.
#[derive(Debug, Clone)]
pub struct OperationLog {
    entries: Vec<LogEntry>,
    /// Index of the last applied entry, or CURSOR_BEFORE_START (-1) if empty/undone all.
    cursor: isize,
    max_size: usize,
}

impl OperationLog {
    /// Create a new empty log with default max size (1000 entries).
    pub fn new() -> Self {
        Self::with_max_size(DEFAULT_MAX_LOG_SIZE)
    }

    /// Const constructor for use in `thread_local!` initializers.
    pub const fn new_const() -> Self {
        Self {
            entries: Vec::new(),
            cursor: CURSOR_BEFORE_START,
            max_size: DEFAULT_MAX_LOG_SIZE,
        }
    }

    /// Create a new empty log with custom max size.
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            cursor: CURSOR_BEFORE_START,
            max_size,
        }
    }

    /// Record a command that was just applied externally to the document.
    ///
    /// The caller is responsible for actually mutating the document (typically
    /// via `processor::apply`). This method only handles log bookkeeping:
    /// truncating the redo branch, appending the entry, evicting old entries,
    /// and advancing the cursor.
    ///
    /// `origin` and `actor` are stored in the log entry for later querying via
    /// [`recent_change_sets_for`](Self::recent_change_sets_for). If not available
    /// (e.g., direct command dispatch without a ChangeSet), this method infers them
    /// from `metadata.authorship`.
    pub fn record(&mut self, envelope: &CommandEnvelope, inverse: Command) {
        self._record(envelope, inverse, None, None, None)
    }

    /// Record a command with explicit provenance (origin and actor).
    ///
    /// Use this method when the `ChangeSet` origin and actor are available
    /// (e.g., when routing through `TransactionKernel::apply_atomic`).
    pub fn record_with_provenance(
        &mut self,
        envelope: &CommandEnvelope,
        inverse: Command,
        origin: String,
        actor: String,
        change_id: Option<String>,
    ) {
        self._record(envelope, inverse, Some(origin), Some(actor), change_id)
    }

    fn _record(
        &mut self,
        envelope: &CommandEnvelope,
        inverse: Command,
        origin: Option<String>,
        actor: Option<String>,
        change_id: Option<String>,
    ) {
        // Truncate redo branch: drop entries after current cursor
        if self.cursor < self.entries.len() as isize - 1 {
            let keep = (self.cursor + 1) as usize;
            self.entries.truncate(keep);
        }

        // Infer origin and actor from metadata.authorship if not provided.
        let (origin, actor) = match (origin, actor) {
            (Some(o), Some(a)) => (o, a),
            _ => {
                let actor = envelope.metadata.authorship.clone();
                let origin = if actor == "user" {
                    "Human".to_string()
                } else if actor.starts_with("agent:") {
                    "Agent".to_string()
                } else if actor == "system" {
                    "Migration".to_string()
                } else {
                    "Human".to_string()
                };
                (origin, actor)
            }
        };

        // Append new entry
        self.entries.push(LogEntry::new(
            envelope.command.clone(),
            inverse,
            envelope.metadata.clone(),
            Some(origin),
            Some(actor),
            change_id,
        ));
        // Evict oldest if over max
        while self.entries.len() > self.max_size {
            self.entries.remove(0);
            self.cursor -= 1;
        }
        // Advance cursor to new last entry
        self.cursor = self.entries.len() as isize - 1;
    }

    /// Apply the inverse of the entry at the cursor, moving the cursor back one.
    /// Returns the post-undo document snapshot.
    pub fn undo(&mut self, doc: &mut SceneDocument) -> Result<SceneDocument, OperationLogError> {
        if !self.can_undo() {
            return Err(OperationLogError::NothingToUndo);
        }
        let entry = &self.entries[self.cursor as usize];
        processor::apply(doc, &entry.inverse)
            .map_err(|e| OperationLogError::InverseFailed(e.to_string()))?;
        self.cursor -= 1;
        Ok(doc.clone())
    }

    /// Apply the forward of the entry after the cursor, moving the cursor forward.
    /// Returns the post-redo document snapshot.
    pub fn redo(&mut self, doc: &mut SceneDocument) -> Result<SceneDocument, OperationLogError> {
        if !self.can_redo() {
            return Err(OperationLogError::NothingToRedo);
        }
        self.cursor += 1;
        let entry = &self.entries[self.cursor as usize];
        processor::apply(doc, &entry.forward).map_err(|e| OperationLogError::CommandFailed(e))?;
        Ok(doc.clone())
    }

    /// True if there is an entry that can be undone.
    pub fn can_undo(&self) -> bool {
        self.cursor >= 0
    }

    /// True if there is an entry that can be redone.
    pub fn can_redo(&self) -> bool {
        self.cursor < self.entries.len() as isize - 1
    }

    /// Number of entries currently in the log.
    pub fn get_log_size(&self) -> usize {
        self.entries.len()
    }

    /// Current cursor position (-1 = before start, otherwise index of last applied).
    pub fn get_cursor(&self) -> isize {
        self.cursor
    }

    /// Read-only access to the entries.
    pub fn get_log(&self) -> &[LogEntry] {
        &self.entries
    }

    /// Reset the log to empty (for new project / testing).
    pub fn clear(&mut self) {
        self.entries.clear();
        self.cursor = CURSOR_BEFORE_START;
    }

    /// Query the recent change sets that touched a specific entity.
    ///
    /// Returns summaries of all log entries whose forward command (or any
    /// nested batch command) references the given `stable_id`. Entries are
    /// returned in reverse chronological order (most recent first).
    ///
    /// Each summary includes the origin, actor, applied-at timestamp (from
    /// `metadata.timestamp`), and the count of operations in this entry
    /// that touch the entity.
    ///
    /// The caller is responsible for bounding the result (e.g., the
    /// `EditorSession.recent_change_sets` deque is capped at 50).
    pub fn recent_change_sets_for(&self, stable_id: &StableId) -> Vec<RecentChangeSummary> {
        let mut results = Vec::new();
        for entry in self.entries.iter().rev() {
            let ops_touched = count_ops_touching_stable_id(&entry.forward, stable_id);
            if ops_touched > 0 {
                results.push(RecentChangeSummary {
                    // origin/actor are stored as Some(...) since _record always sets them
                    change_id: entry.change_id.clone(),
                    origin: entry.origin.clone().unwrap_or_else(|| "Human".to_string()),
                    actor: entry
                        .actor
                        .clone()
                        .unwrap_or_else(|| entry.metadata.authorship.clone()),
                    applied_at: entry.metadata.timestamp,
                    ops_touched,
                });
            }
        }
        results
    }
}

impl Default for OperationLog {
    fn default() -> Self {
        Self::new()
    }
}

/// A summary of one log entry for the recent-change-sets query.
///
/// Returned by [`OperationLog::recent_change_sets_for`].
///
/// The `applied_at` field uses Unix milliseconds (matching `CommandMetadata.timestamp`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentChangeSummary {
    /// ID of the ChangeSet this entry belongs to (e.g. "cmd-1234567890").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_id: Option<String>,
    /// Where the originating ChangeSet came from (e.g. "Human", "Agent").
    pub origin: String,
    /// Who authored the change (e.g., "user" or "agent:foo").
    pub actor: String,
    /// Unix milliseconds when the command was issued.
    pub applied_at: u64,
    /// Number of operations in this entry that touched the queried stable ID.
    pub ops_touched: usize,
}

/// Count how many operations in a command touch a specific stable_id.
///
/// Handles Batch commands recursively. Returns 0 if the command does not
/// reference the entity.
fn count_ops_touching_stable_id(cmd: &Command, stable_id: &StableId) -> usize {
    match cmd {
        Command::CreateEntity { id, .. } if id == stable_id => 1,
        Command::DeleteEntity { id, .. } if id == stable_id => 1,
        Command::AddComponent { entity_id, .. } if entity_id == stable_id => 1,
        Command::RemoveComponent { entity_id, .. } if entity_id == stable_id => 1,
        Command::SetComponentField { entity_id, .. } if entity_id == stable_id => 1,
        Command::SetComponentFieldOnMultiple { entity_ids, .. }
            if entity_ids.iter().any(|id| id == stable_id) =>
        {
            1
        }
        Command::ReparentEntity { entity_id, .. } if entity_id == stable_id => 1,
        Command::RenameEntity { entity_id, .. } if entity_id == stable_id => 1,
        Command::PlaceInstance { instance_id, .. } if instance_id == stable_id => 1,
        Command::RemoveInstance { instance_id, .. } if instance_id == stable_id => 1,
        Command::ReplaceInstanceAsset { instance_id, .. } if instance_id == stable_id => 1,
        Command::UpsertOverride { instance_id, .. } if instance_id == stable_id => 1,
        Command::RevertOverride { instance_id, .. } if instance_id == stable_id => 1,
        Command::Batch { commands, .. } => commands
            .iter()
            .map(|c| count_ops_touching_stable_id(c, stable_id))
            .sum(),
        _ => 0,
    }
}

/// Errors returned by operation log operations.
#[derive(Debug, Error)]
pub enum OperationLogError {
    #[error("Nothing to undo")]
    NothingToUndo,

    #[error("Nothing to redo")]
    NothingToRedo,

    #[error("Command failed: {0}")]
    CommandFailed(#[from] crate::command::CommandError),

    #[error("Inverse application failed: {0}")]
    InverseFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{CommandEnvelope, CommandMetadata};
    use crate::document::{ComponentInstance, Entity, LocalId, StableId};
    use serde_json::json;
    use std::collections::BTreeMap;

    fn empty_doc() -> SceneDocument {
        SceneDocument {
            version: "0.1".to_string(),
            scene_id: "test".to_string(),
            name: "Test".to_string(),
            entities: vec![],
            instances: BTreeMap::new(),
        }
    }

    fn envelope_with(command: Command) -> CommandEnvelope {
        CommandEnvelope {
            command,
            metadata: CommandMetadata::now("test"),
        }
    }

    fn create_inverse() -> (Command, Command) {
        let fwd = Command::CreateEntity {
            id: StableId::new("ent_01"),
            name: "Foo".to_string(),
            components: vec![],
        };
        let inv = Command::DeleteEntity {
            id: StableId::new("ent_01"),
        };
        (fwd, inv)
    }

    // ===== LogEntry serialization =====

    #[test]
    fn test_log_entry_roundtrip() {
        let entry = LogEntry::new(
            Command::CreateEntity {
                id: StableId::new("e1"),
                name: "Test".to_string(),
                components: vec![],
            },
            Command::DeleteEntity {
                id: StableId::new("e1"),
            },
            CommandMetadata::now("user").with_rationale("test"),
            Some("Human".to_string()),
            Some("user".to_string()),
        );
        let json = serde_json::to_string(&entry).unwrap();
        let rt: LogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, rt);
    }

    // ===== apply / record =====

    #[test]
    fn test_record_appends_and_advances_cursor() {
        let mut log = OperationLog::new();
        assert_eq!(log.get_log_size(), 0);
        assert!(!log.can_undo());

        let (fwd, inv) = create_inverse();
        log.record(&envelope_with(fwd), inv);
        assert_eq!(log.get_log_size(), 1);
        assert!(log.can_undo());
        assert!(!log.can_redo());
        assert_eq!(log.get_cursor(), 0);
    }

    #[test]
    fn test_record_multiple_advances_cursor() {
        let mut log = OperationLog::new();
        for i in 0..3 {
            let fwd = Command::CreateEntity {
                id: StableId::new(format!("e{}", i)),
                name: format!("E{}", i),
                components: vec![],
            };
            let inv = Command::DeleteEntity {
                id: StableId::new(format!("e{}", i)),
            };
            log.record(&envelope_with(fwd), inv);
        }
        assert_eq!(log.get_log_size(), 3);
        assert_eq!(log.get_cursor(), 2);
        assert!(log.can_undo());
        assert!(!log.can_redo());
    }

    #[test]
    fn test_batch_logged_as_single_entry() {
        let mut log = OperationLog::new();
        let batch = Command::Batch {
            label: "drag".to_string(),
            commands: vec![
                Command::CreateEntity {
                    id: StableId::new("e1"),
                    name: "E1".to_string(),
                    components: vec![],
                },
                Command::CreateEntity {
                    id: StableId::new("e2"),
                    name: "E2".to_string(),
                    components: vec![],
                },
            ],
        };
        let inv_batch = Command::Batch {
            label: "inverse".to_string(),
            commands: vec![
                Command::DeleteEntity {
                    id: StableId::new("e2"),
                },
                Command::DeleteEntity {
                    id: StableId::new("e1"),
                },
            ],
        };
        log.record(&envelope_with(batch), inv_batch);
        assert_eq!(log.get_log_size(), 1);
        let entry = &log.get_log()[0];
        match &entry.forward {
            Command::Batch { commands, .. } => assert_eq!(commands.len(), 2),
            _ => panic!("Expected Batch"),
        }
    }

    // ===== FIFO eviction =====

    #[test]
    fn test_max_size_evicts_oldest() {
        let mut log = OperationLog::with_max_size(3);
        for i in 0..5 {
            let fwd = Command::CreateEntity {
                id: StableId::new(format!("e{}", i)),
                name: format!("E{}", i),
                components: vec![],
            };
            let inv = Command::DeleteEntity {
                id: StableId::new(format!("e{}", i)),
            };
            log.record(&envelope_with(fwd), inv);
        }
        // Only last 3 should remain: e2, e3, e4
        assert_eq!(log.get_log_size(), 3);
        match &log.get_log()[0].forward {
            Command::CreateEntity { id, .. } => assert_eq!(id.as_str(), "e2"),
            _ => panic!("Expected CreateEntity"),
        }
        match &log.get_log()[2].forward {
            Command::CreateEntity { id, .. } => assert_eq!(id.as_str(), "e4"),
            _ => panic!("Expected CreateEntity"),
        }
        // Cursor should be at end (len - 1)
        assert_eq!(log.get_cursor(), 2);
    }

    #[test]
    fn test_max_size_eviction_adjusts_cursor_after_undo() {
        let mut log = OperationLog::with_max_size(3);
        let mut doc = empty_doc();
        // Apply 5 (with doc mutations)
        for i in 0..5 {
            let fwd = Command::CreateEntity {
                id: StableId::new(format!("e{}", i)),
                name: format!("E{}", i),
                components: vec![],
            };
            let inv = Command::DeleteEntity {
                id: StableId::new(format!("e{}", i)),
            };
            processor::apply(&mut doc, &fwd).unwrap();
            log.record(&envelope_with(fwd), inv);
        }
        // Log: [e2, e3, e4], cursor = 2
        assert_eq!(log.get_cursor(), 2);

        // Apply one more (e5) — triggers eviction of e2
        let fwd = Command::CreateEntity {
            id: StableId::new("e5"),
            name: "E5".to_string(),
            components: vec![],
        };
        let inv = Command::DeleteEntity {
            id: StableId::new("e5"),
        };
        processor::apply(&mut doc, &fwd).unwrap();
        log.record(&envelope_with(fwd), inv);

        // Log: [e3, e4, e5], cursor = 2
        assert_eq!(log.get_log_size(), 3);
        assert_eq!(log.get_cursor(), 2);
        match &log.get_log()[0].forward {
            Command::CreateEntity { id, .. } => assert_eq!(id.as_str(), "e3"),
            _ => panic!("Expected CreateEntity"),
        }
    }

    // ===== undo =====

    #[test]
    fn test_undo_applies_inverse_and_moves_cursor_back() {
        let mut log = OperationLog::new();
        let mut doc = empty_doc();
        let (fwd, inv) = create_inverse();
        // Apply forward manually then record
        processor::apply(&mut doc, &fwd).unwrap();
        log.record(&envelope_with(fwd), inv);

        assert_eq!(doc.entities.len(), 1);

        // Undo: applies inverse (DeleteEntity)
        log.undo(&mut doc).unwrap();
        assert_eq!(doc.entities.len(), 0);
        assert_eq!(log.get_cursor(), -1);
        assert!(!log.can_undo());
        // can_redo is true: there's a redoable entry (we just undid it)
        assert!(log.can_redo());
    }

    #[test]
    fn test_undo_at_start_returns_error() {
        let mut log = OperationLog::new();
        let mut doc = empty_doc();
        let result = log.undo(&mut doc);
        assert!(matches!(result, Err(OperationLogError::NothingToUndo)));
    }

    #[test]
    fn test_undo_after_truncate_returns_nothing() {
        let mut log = OperationLog::new();
        let mut doc = empty_doc();
        // Add 2 entries (with doc mutations)
        for i in 0..2 {
            let (fwd, inv) = (
                Command::CreateEntity {
                    id: StableId::new(format!("e{}", i)),
                    name: format!("E{}", i),
                    components: vec![],
                },
                Command::DeleteEntity {
                    id: StableId::new(format!("e{}", i)),
                },
            );
            processor::apply(&mut doc, &fwd).unwrap();
            log.record(&envelope_with(fwd), inv);
        }
        // Undo both: cursor = -1
        log.undo(&mut doc).unwrap();
        log.undo(&mut doc).unwrap();
        // Apply new command: truncates redo branch (already empty), cursor = 0
        let (fwd, inv) = create_inverse();
        processor::apply(&mut doc, &fwd).unwrap();
        log.record(&envelope_with(fwd), inv);
        // Now undo: should work
        log.undo(&mut doc).unwrap();
        assert_eq!(log.get_cursor(), -1);
        // Undo again: error
        let result = log.undo(&mut doc);
        assert!(matches!(result, Err(OperationLogError::NothingToUndo)));
    }

    // ===== redo =====

    #[test]
    fn test_redo_applies_forward_and_moves_cursor_forward() {
        let mut log = OperationLog::new();
        let mut doc = empty_doc();
        let (fwd, inv) = create_inverse();
        // Apply forward manually then record
        processor::apply(&mut doc, &fwd).unwrap();
        log.record(&envelope_with(fwd), inv);

        log.undo(&mut doc).unwrap();
        assert_eq!(log.get_cursor(), -1);
        assert!(log.can_redo());
        assert!(doc.entities.is_empty());

        log.redo(&mut doc).unwrap();
        assert_eq!(log.get_cursor(), 0);
        assert!(doc.entities.iter().any(|e| e.id.as_str() == "ent_01"));
    }

    #[test]
    fn test_redo_at_end_returns_error() {
        let mut log = OperationLog::new();
        let mut doc = empty_doc();
        let (fwd, inv) = create_inverse();
        log.record(&envelope_with(fwd), inv);
        let result = log.redo(&mut doc);
        assert!(matches!(result, Err(OperationLogError::NothingToRedo)));
    }

    #[test]
    fn test_redo_on_empty_log_returns_error() {
        let mut log = OperationLog::new();
        let mut doc = empty_doc();
        let result = log.redo(&mut doc);
        assert!(matches!(result, Err(OperationLogError::NothingToRedo)));
    }

    // ===== truncate on new command =====

    #[test]
    fn test_truncate_on_new_command_after_undo() {
        let mut log = OperationLog::new();
        let mut doc = empty_doc();
        // Apply 3 entries
        for i in 0..3 {
            let (fwd, inv) = (
                Command::CreateEntity {
                    id: StableId::new(format!("e{}", i)),
                    name: format!("E{}", i),
                    components: vec![],
                },
                Command::DeleteEntity {
                    id: StableId::new(format!("e{}", i)),
                },
            );
            log.record(&envelope_with(fwd), inv);
            // Re-add to doc since processor::apply already applied forward
            doc.entities.push(Entity {
                id: StableId::new(format!("e{}", i)),
                local_id: LocalId::new(format!("e{}", i)),
                name: format!("E{}", i),
                parent: None,
                components: vec![],
            });
        }
        assert_eq!(log.get_log_size(), 3);

        // Undo twice: cursor = 0
        log.undo(&mut doc).unwrap();
        log.undo(&mut doc).unwrap();
        assert_eq!(log.get_cursor(), 0);

        // New command: should truncate entries 1 and 2
        let (fwd, inv) = (
            Command::CreateEntity {
                id: StableId::new("new"),
                name: "New".to_string(),
                components: vec![],
            },
            Command::DeleteEntity {
                id: StableId::new("new"),
            },
        );
        log.record(&envelope_with(fwd), inv);

        assert_eq!(log.get_log_size(), 2);
        // First entry should still be e0
        match &log.get_log()[0].forward {
            Command::CreateEntity { id, .. } => assert_eq!(id.as_str(), "e0"),
            _ => panic!("Expected CreateEntity"),
        }
        // Second entry should be the new one
        match &log.get_log()[1].forward {
            Command::CreateEntity { id, .. } => assert_eq!(id.as_str(), "new"),
            _ => panic!("Expected CreateEntity"),
        }
        // No redo branch
        assert!(!log.can_redo());
    }

    #[test]
    fn test_no_truncate_at_end() {
        let mut log = OperationLog::new();
        for i in 0..3 {
            let (fwd, inv) = (
                Command::CreateEntity {
                    id: StableId::new(format!("e{}", i)),
                    name: format!("E{}", i),
                    components: vec![],
                },
                Command::DeleteEntity {
                    id: StableId::new(format!("e{}", i)),
                },
            );
            log.record(&envelope_with(fwd), inv);
        }
        // Apply one more — cursor at end, no truncate
        let (fwd, inv) = (
            Command::CreateEntity {
                id: StableId::new("e3"),
                name: "E3".to_string(),
                components: vec![],
            },
            Command::DeleteEntity {
                id: StableId::new("e3"),
            },
        );
        log.record(&envelope_with(fwd), inv);
        assert_eq!(log.get_log_size(), 4);
    }

    // ===== can_undo / can_redo state =====

    #[test]
    fn test_can_undo_can_redo_reflect_cursor() {
        let mut log = OperationLog::new();
        let mut doc = empty_doc();
        assert!(!log.can_undo());
        assert!(!log.can_redo());

        for i in 0..3 {
            let (fwd, inv) = (
                Command::CreateEntity {
                    id: StableId::new(format!("e{}", i)),
                    name: format!("E{}", i),
                    components: vec![],
                },
                Command::DeleteEntity {
                    id: StableId::new(format!("e{}", i)),
                },
            );
            processor::apply(&mut doc, &fwd).unwrap();
            log.record(&envelope_with(fwd), inv);
        }
        assert!(log.can_undo());
        assert!(!log.can_redo());

        log.undo(&mut doc).unwrap();
        assert!(log.can_undo());
        assert!(log.can_redo());

        log.undo(&mut doc).unwrap();
        log.undo(&mut doc).unwrap();
        assert!(!log.can_undo());
        assert!(log.can_redo());

        log.redo(&mut doc).unwrap();
        log.redo(&mut doc).unwrap();
        log.redo(&mut doc).unwrap();
        assert!(log.can_undo());
        assert!(!log.can_redo());
    }

    // ===== undo + redo roundtrip =====

    #[test]
    fn test_undo_redo_roundtrip_restores_state() {
        let mut log = OperationLog::new();
        let mut doc = empty_doc();
        let (fwd, inv) = create_inverse();
        log.record(&envelope_with(fwd), inv);
        doc.entities.push(Entity {
            id: StableId::new("ent_01"),
            local_id: LocalId::new("ent_01"),
            name: "Foo".to_string(),
            parent: None,
            components: vec![],
        });

        // Undo: doc empty
        log.undo(&mut doc).unwrap();
        assert!(doc.entities.is_empty());

        // Redo: doc has 1 entity
        log.redo(&mut doc).unwrap();
        assert_eq!(doc.entities.len(), 1);
        assert_eq!(doc.entities[0].id.as_str(), "ent_01");
    }

    #[test]
    fn test_undo_redo_preserves_component_values() {
        let mut log = OperationLog::new();
        let mut doc = empty_doc();
        let fwd = Command::SetComponentField {
            entity_id: StableId::new("e1"),
            type_id: "editor.Transform2D".to_string(),
            field_path: "translation.x".to_string(),
            value: json!(999.0),
        };
        let inv = Command::SetComponentField {
            entity_id: StableId::new("e1"),
            type_id: "editor.Transform2D".to_string(),
            field_path: "translation.x".to_string(),
            value: json!(0.0),
        };
        log.record(&envelope_with(fwd.clone()), inv);
        doc.entities.push(Entity {
            id: StableId::new("e1"),
            local_id: LocalId::new("e1"),
            name: "E1".to_string(),
            parent: None,
            components: vec![ComponentInstance {
                type_id: "editor.Transform2D".to_string(),
                values: json!({
                    "translation": {"x": 999.0, "y": 0.0},
                    "rotation": 0.0,
                    "scale": {"x": 1.0, "y": 1.0}
                }),
            }],
        });

        log.undo(&mut doc).unwrap();
        assert_eq!(
            doc.entities[0].components[0].values["translation"]["x"],
            json!(0.0)
        );

        log.redo(&mut doc).unwrap();
        assert_eq!(
            doc.entities[0].components[0].values["translation"]["x"],
            json!(999.0)
        );
    }

    // ===== empty log edge cases =====

    #[test]
    fn test_undo_on_empty_log_returns_error_no_panic() {
        let mut log = OperationLog::new();
        let mut doc = empty_doc();
        let result = log.undo(&mut doc);
        assert!(matches!(result, Err(OperationLogError::NothingToUndo)));
    }

    #[test]
    fn test_redo_on_empty_log_returns_error_no_panic() {
        let mut log = OperationLog::new();
        let mut doc = empty_doc();
        let result = log.redo(&mut doc);
        assert!(matches!(result, Err(OperationLogError::NothingToRedo)));
    }

    #[test]
    fn test_get_log_size_on_empty() {
        let log = OperationLog::new();
        assert_eq!(log.get_log_size(), 0);
    }

    // ===== clear =====

    #[test]
    fn test_clear_resets_log() {
        let mut log = OperationLog::new();
        let (fwd, inv) = create_inverse();
        log.record(&envelope_with(fwd), inv);
        assert_eq!(log.get_log_size(), 1);

        log.clear();
        assert_eq!(log.get_log_size(), 0);
        assert_eq!(log.get_cursor(), -1);
    }
}
