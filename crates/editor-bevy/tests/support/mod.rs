//! Test support utilities for `editor-core` integration tests.
//!
//! ## `FakeSession` — shared struct with all fields
//!
//! Provides a [`FakeSession`] containing every field that any test file needs.
//! This eliminates ~150 lines of duplicate field declarations across the 5
//! test files that each defined their own `struct FakeSession { ... }` with
//! the same 11 fields.
//!
//! ## `FakeSessionWithDefaults` — ready-to-use impl
//!
//! For tests that don't need to override any `EditorSessionPort` methods,
//! `FakeSessionWithDefaults` wraps `FakeSession` and provides a complete
//! working `EditorSessionPort` impl with sensible defaults (empty maps,
//! None for optionals, no-ops for mutating methods).
//!
//! ## Usage
//!
//! **For tests that need custom `EditorSessionPort` methods:**
//!
//! ```ignore
//! #[path = "support/mod.rs"]
//! mod support;
//!
//! use editor_model::EditorSessionPort;
//! use std::collections::BTreeMap;
//! use std::sync::{Arc, Mutex};
//!
//! // Define only the fields you need; everything else gets defaults
//! struct FakeSession {
//!     scene_states: BTreeMap<String, editor_model::SceneSessionState>,
//!     asset_states: BTreeMap<String, editor_model::AssetSessionState>,
//!     logic_states: BTreeMap<String, editor_model::LogicSessionState>,
//!     recent_change_sets: BTreeMap<String, Vec<editor_model::ChangeSetSummary>>,
//! }
//!
//! // Implement EditorSessionPort manually for your custom methods.
//! // Fields you don't override use the support module's shared impl via
//! // the FakeSessionWithDefaults approach (see below), or return defaults.
//! impl EditorSessionPort for FakeSession {
//!     fn scene_state_mut(&mut self, path: &str) -> &mut editor_model::SceneSessionState {
//!         self.scene_states
//!             .entry(path.to_string())
//!             .or_insert_with(editor_model::SceneSessionState::default())
//!     }
//!     fn all_recent_change_sets(&self) -> Vec<editor_model::ChangeSetSummary> {
//!         self.recent_change_sets
//!             .values()
//!             .flat_map(|v| v.iter().cloned())
//!             .collect()
//!     }
//!     fn push_recent_change_set(&mut self, scene_path: &str, summary: editor_model::ChangeSetSummary) {
//!         self.recent_change_sets
//!             .entry(scene_path.to_string())
//!             .or_insert_with(Vec::new)
//!             .push(summary);
//!     }
//!     // ... delegate all other methods to FakeSessionWithDefaults:
//!     fn tunable_baselines_mut(&mut self) -> &mut std::collections::BTreeMap<String, serde_json::Value> {
//!         &mut support::FakeSessionWithDefaults::default().inner.tunable_baselines
//!     }
//!     // ... or use support::FakeSessionWithDefaults as the base instead.
//! }
//!
//! fn fresh_session() {
//!     let session = FakeSession {
//!         scene_states: BTreeMap::new(),
//!         asset_states: BTreeMap::new(),
//!         logic_states: BTreeMap::new(),
//!         recent_change_sets: BTreeMap::new(),
//!     };
//!     let arc = std::sync::Arc::new(std::sync::Mutex::new(session));
//!     editor_model::ports::register_editor_session(arc);
//! }
//! ```
//!
//! **For tests that don't override any methods:**
//!
//! ```ignore
//! #[path = "support/mod.rs"]
//! mod support;
//!
//! fn fresh_session() {
//!     let session = support::FakeSessionWithDefaults::default();
//!     let arc = std::sync::Arc::new(std::sync::Mutex::new(session));
//!     editor_model::ports::register_editor_session(arc);
//! }
//! ```

pub use editor_model::CausalityEdge;
pub use editor_model::ChangeSetSummary;
pub use editor_model::EditorSessionPort;
pub use editor_model::LogicActivationEvent;
pub use editor_model::LogicSessionState;
pub use editor_model::PreviewInspectorState;
pub use editor_model::RuntimeDelta;
pub use editor_model::SceneSessionState;
pub use editor_model::SourceFilesCache;
pub use editor_model::StableId;
use std::collections::{BTreeMap, VecDeque};

#[allow(unused_extern_crates)]
extern crate serde_json;

// Re-export the editor_model crate so tests can use `support::editor_model::X`
pub use editor_model;
pub use editor_model::AssetSessionState;
pub use editor_model::RebuildCause;
pub use editor_model::WorldSessionState;

// ─── FakeSession ─────────────────────────────────────────────────────────────

/// Full-featured fake session for `EditorSessionPort` tests.
///
/// Contains every field that any test file needs. Fields not used by a
/// particular test retain their default (empty) values.
///
/// All 11 fields are public so test files can construct only what they need.
///
/// # Example (minimal construction)
///
/// ```ignore
///     FakeSession {
///         scene_states: BTreeMap::new(),
///         asset_states: BTreeMap::new(),
///         logic_states: BTreeMap::new(),
///         world_states: BTreeMap::new(),
///         tunable_baselines: BTreeMap::new(),
///         runtime_delta_buffer: VecDeque::new(),
///         pending_causality_edges: BTreeMap::new(),
///         last_rebuild_cause: None,
///         preview_inspector: PreviewInspectorState::default(),
///         source_files: SourceFilesCache::default(),
///         logic_activation_ring: VecDeque::new(),
///         recent_change_sets: BTreeMap::new(),
///     }
/// ```
#[derive(Debug, Default)]
pub struct FakeSession {
    pub scene_states: BTreeMap<String, SceneSessionState>,
    pub asset_states: BTreeMap<String, AssetSessionState>,
    pub logic_states: BTreeMap<String, LogicSessionState>,
    pub world_states: BTreeMap<String, WorldSessionState>,
    pub tunable_baselines: BTreeMap<String, serde_json::Value>,
    pub runtime_delta_buffer: VecDeque<RuntimeDelta>,
    pub pending_causality_edges: BTreeMap<StableId, Vec<CausalityEdge>>,
    pub last_rebuild_cause: Option<RebuildCause>,
    pub preview_inspector: PreviewInspectorState,
    pub source_files: SourceFilesCache,
    pub logic_activation_ring: VecDeque<LogicActivationEvent>,
    pub recent_change_sets: BTreeMap<String, Vec<ChangeSetSummary>>,
}

impl FakeSession {
    /// Construct a fully-empty FakeSession (all maps/deques empty, all options None).
    pub fn new() -> Self {
        Self::default()
    }
}

// ─── EditorSessionPort impl for FakeSession ─────────────────────────────────
//
// Provides panic defaults for all methods. Tests that need custom behavior
// implement the trait on their own wrapper struct and override specific methods.

impl EditorSessionPort for FakeSession {
    fn scene_state_mut(&mut self, path: &str) -> &mut SceneSessionState {
        self.scene_states
            .entry(path.to_string())
            .or_insert_with(SceneSessionState::default)
    }
    fn asset_state_mut(&mut self, path: &str) -> &mut AssetSessionState {
        self.asset_states
            .entry(path.to_string())
            .or_insert_with(AssetSessionState::default)
    }
    fn logic_state_mut(&mut self, path: &str) -> &mut LogicSessionState {
        self.logic_states
            .entry(path.to_string())
            .or_insert_with(LogicSessionState::default)
    }
    fn world_state_mut(&mut self, path: &str) -> &mut WorldSessionState {
        self.world_states
            .entry(path.to_string())
            .or_insert_with(WorldSessionState::default)
    }
    fn tunable_baselines_mut(&mut self) -> &mut BTreeMap<String, serde_json::Value> {
        &mut self.tunable_baselines
    }
    fn runtime_delta_buffer_mut(&mut self) -> &mut VecDeque<RuntimeDelta> {
        &mut self.runtime_delta_buffer
    }
    fn pending_causality_edges_mut(&mut self) -> &mut BTreeMap<StableId, Vec<CausalityEdge>> {
        &mut self.pending_causality_edges
    }
    fn last_rebuild_cause_mut(&mut self) -> &mut Option<RebuildCause> {
        &mut self.last_rebuild_cause
    }
    fn preview_inspector_mut(&mut self) -> &mut PreviewInspectorState {
        &mut self.preview_inspector
    }
    fn source_files_mut(&mut self) -> &mut SourceFilesCache {
        &mut self.source_files
    }
    fn logic_activation_ring_mut(&mut self) -> &mut VecDeque<LogicActivationEvent> {
        &mut self.logic_activation_ring
    }
    fn recent_change_sets_for(&self, scene_path: &str) -> Vec<ChangeSetSummary> {
        self.recent_change_sets
            .get(scene_path)
            .cloned()
            .unwrap_or_default()
    }
    fn all_recent_change_sets(&self) -> Vec<ChangeSetSummary> {
        self.recent_change_sets
            .values()
            .flat_map(|v| v.iter().cloned())
            .collect()
    }
    fn active_document_path(&self) -> Option<&str> {
        None
    }
    fn push_recent_change_set(&mut self, scene_path: &str, summary: ChangeSetSummary) {
        self.recent_change_sets
            .entry(scene_path.to_string())
            .or_insert_with(Vec::new)
            .push(summary);
    }
}

// ─── FakeSessionWithDefaults ─────────────────────────────────────────────────

/// A `FakeSession` newtype wrapper that implements `EditorSessionPort`
/// by forwarding to the inner `FakeSession`'s own impl.
///
/// Tests that don't need to override any method can use this directly:
///
/// ```ignore
/// let session = support::FakeSessionWithDefaults(support::FakeSession::new());
/// let arc = Arc::new(Mutex::new(session));
/// ```
///
/// Since `FakeSession` itself now implements `EditorSessionPort` with sensible
/// defaults, this wrapper just forwards all calls to it.
#[derive(Debug, Default)]
pub struct FakeSessionWithDefaults(pub FakeSession);

impl std::ops::Deref for FakeSessionWithDefaults {
    type Target = FakeSession;
    fn deref(&self) -> &FakeSession {
        &self.0
    }
}

impl std::ops::DerefMut for FakeSessionWithDefaults {
    fn deref_mut(&mut self) -> &mut FakeSession {
        &mut self.0
    }
}

impl EditorSessionPort for FakeSessionWithDefaults {
    // Forward all calls to FakeSession's impl
    fn scene_state_mut(&mut self, path: &str) -> &mut SceneSessionState {
        self.0.scene_state_mut(path)
    }
    fn asset_state_mut(&mut self, path: &str) -> &mut AssetSessionState {
        self.0.asset_state_mut(path)
    }
    fn logic_state_mut(&mut self, path: &str) -> &mut LogicSessionState {
        self.0.logic_state_mut(path)
    }
    fn world_state_mut(&mut self, path: &str) -> &mut WorldSessionState {
        self.0.world_state_mut(path)
    }
    fn tunable_baselines_mut(&mut self) -> &mut BTreeMap<String, serde_json::Value> {
        self.0.tunable_baselines_mut()
    }
    fn runtime_delta_buffer_mut(&mut self) -> &mut VecDeque<RuntimeDelta> {
        self.0.runtime_delta_buffer_mut()
    }
    fn pending_causality_edges_mut(&mut self) -> &mut BTreeMap<StableId, Vec<CausalityEdge>> {
        self.0.pending_causality_edges_mut()
    }
    fn last_rebuild_cause_mut(&mut self) -> &mut Option<RebuildCause> {
        self.0.last_rebuild_cause_mut()
    }
    fn preview_inspector_mut(&mut self) -> &mut PreviewInspectorState {
        self.0.preview_inspector_mut()
    }
    fn source_files_mut(&mut self) -> &mut SourceFilesCache {
        self.0.source_files_mut()
    }
    fn logic_activation_ring_mut(&mut self) -> &mut VecDeque<LogicActivationEvent> {
        self.0.logic_activation_ring_mut()
    }
    fn recent_change_sets_for(&self, scene_path: &str) -> Vec<ChangeSetSummary> {
        self.0.recent_change_sets_for(scene_path)
    }
    fn all_recent_change_sets(&self) -> Vec<ChangeSetSummary> {
        self.0.all_recent_change_sets()
    }
    fn active_document_path(&self) -> Option<&str> {
        self.0.active_document_path()
    }
    fn push_recent_change_set(&mut self, scene_path: &str, summary: ChangeSetSummary) {
        self.0.push_recent_change_set(scene_path, summary)
    }
}
