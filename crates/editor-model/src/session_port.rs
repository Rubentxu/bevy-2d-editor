//! EditorSessionPort — abstraction over `EditorSession` for cross-crate access.
//!
//! `editor-core` (Bevy systems) cannot import `editor-application` (per ADR-0031/0032
//! dep direction: `editor-application → editor-core`, never the reverse). To give
//! `editor-core` a way to read/write the session-owned sub-state, this trait lives in
//! `editor-model` (which `editor-core` can import) and `editor-application` provides
//! the concrete impl for its own `EditorSession`.
//!
//! ## Pattern (v0.88 PR B / v0.90 PR1)
//!
//! ```text
//!   editor-core (Bevy systems)
//!        │ uses editor_model::ports::with_session_mut(|s| { ... })
//!        ▼
//!   editor-model::ports (the trait + thread_local registry)
//!        │ dyn EditorSessionPort
//!        ▼
//!   editor-application::EditorSession (impl EditorSessionPort for EditorSession)
//! ```
//!
//! The thread_local registry holds `Arc<Mutex<dyn EditorSessionPort>>` (NOT the
//! concrete `EditorSession`) so `editor-model` does not need to know about
//! `editor-application`'s types.

use crate::causality::CausalityEdge;
use crate::ids::StableId;
use crate::rebuild_cause::RebuildCause;
use crate::runtime_delta::RuntimeDelta;
use std::collections::{BTreeMap, VecDeque};

/// Application-level port to `EditorSession` for editor-core (Bevy systems) and
/// any other crate that cannot import `editor-application`.
///
/// `editor-application` provides the impl. The trait is object-safe (`dyn
/// EditorSessionPort` is valid) so the global registry in
/// `editor_model::ports` can hold a type-erased session.
pub trait EditorSessionPort {
    /// Scene session state (per scene path).
    fn scene_state_mut(&mut self, path: &str) -> &mut crate::session::SceneSessionState;

    /// Asset session state (per asset path).
    fn asset_state_mut(&mut self, path: &str) -> &mut crate::session::AssetSessionState;

    /// Logic session state (per logic graph path).
    fn logic_state_mut(&mut self, path: &str) -> &mut crate::session::LogicSessionState;

    /// Preview inspector state (FPS, mapping, provenance, last rebuild cause).
    fn preview_inspector_mut(&mut self) -> &mut crate::session::PreviewInspectorState;

    /// Source files cache.
    fn source_files_mut(&mut self) -> &mut crate::session::SourceFilesCache;

    /// Recent change-set summaries per scene path (capped at 50 per scene).
    fn recent_change_sets_for(&self, scene_path: &str) -> Vec<crate::session::ChangeSetSummary>;

    /// Logic activation ring (capped at 64).
    fn logic_activation_ring_mut(
        &mut self,
    ) -> &mut VecDeque<crate::logic_activation::LogicActivationEvent>;

    /// Authoring baselines captured at `PlayModeEnter`.
    ///
    /// Map key is a stable identifier for the (instance, component, field) triple.
    /// Map value is the JSON value of the field at authoring time.
    fn tunable_baselines_mut(&mut self) -> &mut BTreeMap<String, serde_json::Value>;

    /// Last recorded rebuild cause (spec §6 D3/D7).
    fn last_rebuild_cause_mut(&mut self) -> &mut Option<RebuildCause>;

    /// Pending causality edges collected during a preview rebuild.
    ///
    /// Keyed by target `StableId`; drained and applied to `PreviewProvenance`
    /// at the end of each rebuild.
    fn pending_causality_edges_mut(&mut self) -> &mut BTreeMap<StableId, Vec<CausalityEdge>>;

    /// Runtime deltas captured on `PlayModeExit`. Ring capped at 64.
    fn runtime_delta_buffer_mut(&mut self) -> &mut VecDeque<RuntimeDelta>;

    // Future v0.90 PRs add more methods:
    // - scene_state_mut(path) -> &mut SceneSessionState
    // - asset_state_mut(path) -> &mut AssetSessionState
    // - logic_state_mut(path) -> &mut LogicSessionState
    // - preview_inspector_mut() -> &mut PreviewInspectorState
    // - source_files_mut() -> &mut SourceFilesCache
    // - recent_change_sets_for(scene_path) -> Vec<ChangeSetSummary>
}
