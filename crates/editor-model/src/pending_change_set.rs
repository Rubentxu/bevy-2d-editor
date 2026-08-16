//! Pending ChangeSet types for the ChangeWorkbench UI (ADR-0039).
//!
//! These are pure data types — no domain logic, no WASM, no thread-local state.
//! Stored in `EditorSession` in `editor-application`.

use serde::{Deserialize, Serialize};

/// A pending ChangeSet stored while awaiting user approval in the ChangeWorkbench.
///
/// The ops are stored as `serde_json::Value` to decouple the WASM boundary from
/// the concrete `SceneCommand` type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingChangeSet {
    /// Unique change-set identifier.
    pub id: String,
    /// Where the change originated (e.g. "Human", "Agent", "Recipe").
    pub origin: String,
    /// Who authored this change (e.g. "user", "agent:foo").
    pub actor: String,
    /// Human-readable rationale.
    pub rationale: String,
    /// Operations in this ChangeSet, stored as JSON to decouple WASM boundary.
    pub ops: Vec<serde_json::Value>,
    /// When this ChangeSet was submitted (Unix ms).
    pub submitted_at_ms: u64,
}

/// Summary of a pending ChangeSet returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingChangeSetSummary {
    /// Change-set ID.
    pub id: String,
    /// Where the change originated.
    pub origin: String,
    /// Who authored this change.
    pub actor: String,
    /// Rationale description.
    pub rationale: String,
    /// Number of operations in this ChangeSet.
    pub op_count: usize,
    /// When submitted (Unix ms).
    pub submitted_at_ms: u64,
}

impl From<&PendingChangeSet> for PendingChangeSetSummary {
    fn from(cs: &PendingChangeSet) -> Self {
        Self {
            id: cs.id.clone(),
            origin: cs.origin.clone(),
            actor: cs.actor.clone(),
            rationale: cs.rationale.clone(),
            op_count: cs.ops.len(),
            submitted_at_ms: cs.submitted_at_ms,
        }
    }
}
