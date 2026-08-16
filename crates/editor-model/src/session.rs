//! Session types for the Bevy 2D Editor domain model.
//!
//! These types are at the bottom of the dependency chain (`editor-model` has no
//! dependencies on `editor-core` or `editor-application`). Moving them here
//! breaks the `editor-application → editor-core → editor-application` circular
//! dependency that blocked the v0.88 `EditorSession` migration.
//!
//! ## v0.90 PR4 additions
//!
//! [`SceneSessionState`], [`AssetSessionState`], [`LogicSessionState`] are the
//! per-path sub-state types that replace the `SCENE_DOC`, `SCENE_ASSET_CATALOG`,
//! `LOGIC_GRAPH_DOC`, `ASSET_OPERATION_LOG`, and `LOGIC_OPERATION_LOG`
//! thread_locals in `editor-core`. The owning maps live on `EditorSession`
//! (in `editor-application::session`) and are exposed through the
//! `EditorSessionPort` trait via `scene_state_mut(path)`,
//! `asset_state_mut(path)`, and `logic_state_mut(path)` accessors that
//! create-on-write (idempotent).
//! dependency that blocked the ADR-0031 `EditorSession` migration.

pub use crate::time::Timestamp;

/// Metadata about the most recently applied change within a [`HistoryScope`].
///
/// Stored by [`TransactionKernel::apply_atomic`] after each successful apply
/// so the UI can display provenance and the scope revision stays consistent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedChangeMeta {
    /// Change set ID that was applied.
    pub change_id: String,
    /// Where the change originated.
    pub origin: crate::transaction::ChangeOrigin,
    /// Actor who authored the change.
    pub actor: String,
    /// Timestamp when the change was applied.
    pub applied_at: Timestamp,
}

/// Explicit operation-history scope for one document or domain.
///
/// ADR-0031 rule: "operation histories are scoped explicitly" — each document
/// (scene, scene asset, logic graph) has its own `HistoryScope` that survives
/// document deactivation and is only reset on explicit "forget history" actions.
///
/// [`TransactionKernel::apply_atomic`]: super::transaction::TransactionKernel::apply_atomic
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryScope {
    revision: u64,
    /// Metadata about the most recently applied change, if any.
    last_change: Option<AppliedChangeMeta>,
}

impl HistoryScope {
    /// Construct a new history scope with revision 0 and no prior change.
    pub fn new() -> Self {
        Self {
            revision: 0,
            last_change: None,
        }
    }

    /// Returns the current revision number.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the next revision number and increments the stored value.
    pub fn next_revision(&mut self) -> u64 {
        let next = self.revision + 1;
        self.revision = next;
        next
    }

    /// Returns metadata about the most recently applied change, if any.
    pub fn last_change(&self) -> Option<&AppliedChangeMeta> {
        self.last_change.as_ref()
    }

    /// Record metadata about an applied change.
    ///
    /// Called by [`TransactionKernel::apply_atomic`] after a successful apply
    /// to record provenance.
    pub fn record_applied(&mut self, meta: AppliedChangeMeta) {
        self.last_change = Some(meta);
    }
}

impl Default for HistoryScope {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// v0.90 PR4: Sub-state types for per-path session state.
// ─────────────────────────────────────────────────────────────────────────────
//
// These types hold the per-path session state currently scattered across
// thread_locals in editor-core. Each sub-state type is a `Default` struct
// that owns the data for a single (scene | asset | logic) path. The owning
// map lives on `EditorSession` (keyed by path string); the `EditorSessionPort`
// trait exposes `scene_state_mut(path)`, `asset_state_mut(path)`,
// `logic_state_mut(path)` accessors that create-on-write.

/// Per-scene session state. Replaces the `SCENE_DOC` + per-scene
/// `OPERATION_LOG` thread_locals (the latter is a session-owned map in
/// editor-application::session; the former is the editor-core thread_local
/// removed in v0.90 PR4).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SceneSessionState {
    /// JSON-serialized snapshot of the active scene document, if any.
    /// `None` means "no scene active". `Some(_)` carries the document.
    pub scene_doc: Option<crate::document::SceneDocument>,
    /// Number of times this scene has been reloaded (hot-reload counter).
    pub reload_count: u32,
}

/// Per-asset session state. Replaces the `SCENE_ASSET_CATALOG`
/// thread_local in editor-core (line 19) and the per-asset
/// `ASSET_OPERATION_LOG` (line 662 of asset_command.rs).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AssetSessionState {
    /// Cached scene asset bodies keyed by asset path.
    pub asset_bodies: std::collections::BTreeMap<String, crate::scene_asset::SceneAssetDocument>,
    /// Pending operation log for asset commands (one log per asset path).
    /// **v0.90 PR4 placeholder**: stored as serialized bytes until
    /// `OperationLog` itself moves from `editor-core` to `editor-model` in
    /// PR5. `Vec::new()` means "no log yet"; load via `AssetSessionState::deserialize_log`.
    pub operation_log_bytes: Vec<u8>,
}

/// Per-logic-graph session state. Replaces the `LOGIC_GRAPH_DOC` +
/// `LOGIC_OPERATION_LOG` thread_locals in editor-core (logic_state.rs:13
/// and logic_state.rs:155; logic_command.rs:371).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LogicSessionState {
    /// Cached logic graph bodies keyed by logic graph path.
    pub graph_docs: std::collections::BTreeMap<String, crate::logic_graph::LogicGraphAsset>,
    /// Pending operation log for logic commands.
    /// **v0.90 PR4 placeholder**: stored as serialized bytes (see
    /// `AssetSessionState::operation_log_bytes` for rationale).
    pub operation_log_bytes: Vec<u8>,
}

// ─────────────────────────────────────────────────────────────────────────────
// v0.90 PR5: Move ChangeSetSummary from editor-application to editor-model.
// ─────────────────────────────────────────────────────────────────────────────
//
// editor-model::EditorSessionPort::recent_change_sets_for needs this type in
// its signature. Moving it here keeps editor-model the single source of truth
// for editor session state (per the v0.88 PR B architecture).

use serde::{Deserialize, Serialize};

/// A query-friendly summary of a recently applied change set.
///
/// Returned by [`OperationLog::recent_change_sets_for`](crate::operation_log::OperationLog::recent_change_sets_for).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeSetSummary {
    /// Where the change originated.
    pub origin: String,
    /// Who authored this change.
    pub actor: String,
    /// Timestamp when the change was applied (Unix milliseconds).
    pub applied_at_ms: u64,
    /// Number of operations in this entry that touched the queried stable ID.
    pub ops_touched: usize,
}

// v0.90 PR5: PreviewInspectorState and SourceFilesCache moved from
// editor-application to editor-model (they are referenced by the
// EditorSessionPort trait's new methods).

/// Runtime preview inspector state (live preview world data).
#[derive(Debug, Clone, Default)]
pub struct PreviewInspectorState {
    /// Live runtime metrics (FPS, frame time, rebuild count) as JSON.
    pub metrics: serde_json::Value,
    /// Per-instance runtime-to-editor ID mapping.
    pub mapping: Vec<serde_json::Value>,
    /// Per-StableId provenance records from play mode.
    pub provenance: std::collections::BTreeMap<String, serde_json::Value>,
    /// Last rebuild cause (§6).
    pub last_rebuild_cause: Option<crate::rebuild_cause::RebuildCause>,
}

/// In-memory cache for source file contents.
#[derive(Debug, Clone, Default)]
pub struct SourceFilesCache {
    /// File path → file content.
    pub files: std::collections::BTreeMap<String, String>,
}
