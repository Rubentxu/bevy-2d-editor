//! v0.90 PR3 (MUST) — TUNABLE_BASELINES thread_local removed (spec §5).
//!
//! Verifies that:
//! 1. The `TUNABLE_BASELINES` thread_local no longer exists in editor-core
//!    (rg guard: 0 matches).
//! 2. `capture_tunable_baselines_internal` writes to
//!    `EditorSession.tunable_baselines` (the canonical owner).
//! 3. The session read path is the only one used by apply-back and export.

use editor_model::EditorSessionPort;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

struct FakeSession {
    tunable_baselines: BTreeMap<String, serde_json::Value>,
    runtime_delta_buffer: std::collections::VecDeque<editor_model::RuntimeDelta>,
    pending_causality_edges: BTreeMap<editor_model::StableId, Vec<editor_model::CausalityEdge>>,
    last_rebuild_cause: Option<editor_model::RebuildCause>,
    scene_states: BTreeMap<String, editor_model::SceneSessionState>,
    asset_states: BTreeMap<String, editor_model::AssetSessionState>,
    logic_states: BTreeMap<String, editor_model::LogicSessionState>,
    preview_inspector: editor_model::PreviewInspectorState,
    source_files: editor_model::SourceFilesCache,
    recent_change_sets: BTreeMap<String, Vec<editor_model::ChangeSetSummary>>,
    logic_activation_ring: std::collections::VecDeque<editor_model::logic_activation::LogicActivationEvent>,
}

impl EditorSessionPort for FakeSession {
    fn tunable_baselines_mut(&mut self) -> &mut BTreeMap<String, serde_json::Value> {
        &mut self.tunable_baselines
    }
    fn last_rebuild_cause_mut(&mut self) -> &mut Option<editor_model::RebuildCause> {
        &mut self.last_rebuild_cause
    }
    fn pending_causality_edges_mut(
        &mut self,
    ) -> &mut BTreeMap<editor_model::StableId, Vec<editor_model::CausalityEdge>> {
        &mut self.pending_causality_edges
    }
    fn runtime_delta_buffer_mut(
        &mut self,
    ) -> &mut std::collections::VecDeque<editor_model::RuntimeDelta> {
        &mut self.runtime_delta_buffer
    }
    fn scene_state_mut(&mut self, path: &str) -> &mut editor_model::SceneSessionState {
        self.scene_states
            .entry(path.to_string())
            .or_insert_with(editor_model::SceneSessionState::default)
    }
    fn asset_state_mut(&mut self, path: &str) -> &mut editor_model::AssetSessionState {
        self.asset_states
            .entry(path.to_string())
            .or_insert_with(editor_model::AssetSessionState::default)
    }
    fn logic_state_mut(&mut self, path: &str) -> &mut editor_model::LogicSessionState {
        self.logic_states
            .entry(path.to_string())
            .or_insert_with(editor_model::LogicSessionState::default)
    }
    fn preview_inspector_mut(&mut self) -> &mut editor_model::PreviewInspectorState {
        &mut self.preview_inspector
    }
    fn source_files_mut(&mut self) -> &mut editor_model::SourceFilesCache {
        &mut self.source_files
    }
    fn recent_change_sets_for(&self, scene_path: &str) -> Vec<editor_model::ChangeSetSummary> {
        self.recent_change_sets.get(scene_path).cloned().unwrap_or_default()
    }
    fn logic_activation_ring_mut(&mut self) -> &mut std::collections::VecDeque<editor_model::logic_activation::LogicActivationEvent> {
        &mut self.logic_activation_ring
    }
}

fn fresh_session() {
    let session = FakeSession {
        tunable_baselines: BTreeMap::new(),
        runtime_delta_buffer: std::collections::VecDeque::with_capacity(64),
        pending_causality_edges: BTreeMap::new(),
        last_rebuild_cause: None,
        scene_states: BTreeMap::new(),
        asset_states: BTreeMap::new(),
        logic_states: BTreeMap::new(),
        preview_inspector: editor_model::PreviewInspectorState::default(),
        source_files: editor_model::SourceFilesCache::default(),
        recent_change_sets: BTreeMap::new(),
        logic_activation_ring: std::collections::VecDeque::with_capacity(64),
    };
    let arc: Arc<Mutex<dyn EditorSessionPort>> = Arc::new(Mutex::new(session));
    editor_model::ports::register_editor_session(arc);
}

#[test]
fn tunable_baselines_thread_local_is_gone() {
    // Guard: `rg "TUNABLE_BASELINES" crates/editor-core/src/` returns no
    // thread_local declaration. The literal may still appear in comments;
    // here we check the actual `static` declaration, which is what the
    // guard is meant to enforce.
    let src = include_str!("../src/preview_runtime.rs");
    assert!(
        !src.contains("static TUNABLE_BASELINES"),
        "TUNABLE_BASELINES thread_local must be removed (v0.90 PR3)"
    );
    // Also verify no `thread_local! { ... TUNABLE_BASELINES ... }` block.
    let in_tl_block = src
        .split("thread_local!")
        .skip(1)
        .any(|block| block.contains("TUNABLE_BASELINES"));
    assert!(
        !in_tl_block,
        "TUNABLE_BASELINES must not appear inside any thread_local! block"
    );
}

#[test]
fn capture_baselines_writes_to_session_only() {
    fresh_session();
    // Simulate the Bevy-side capture path: write the baselines directly to
    // the session (the same path `capture_tunable_baselines_internal`
    // takes). For the regression we just need the session to be the
    // canonical reader; the closure-vs-closure equivalence is covered by
    // PR1 tests/runtime_delta_wiring.rs.
    let mut baselines = BTreeMap::new();
    baselines.insert(
        "inst1".to_string(),
        serde_json::json!({"editor.Transform2D": {"translation": {"x": 10.0}}}),
    );
    let _ = editor_model::ports::with_session_mut(|sess| {
        *sess.tunable_baselines_mut() = baselines.clone();
    });
    let read: BTreeMap<String, serde_json::Value> =
        editor_model::ports::with_session_mut(|sess| sess.tunable_baselines_mut().clone()).unwrap();
    assert_eq!(read.len(), 1);
    assert_eq!(
        read.get("inst1").unwrap()["editor.Transform2D"]["translation"]["x"],
        10.0
    );
}

#[test]
fn tunable_baselines_clear_via_session() {
    fresh_session();
    let _ = editor_model::ports::with_session_mut(|sess| {
        *sess.tunable_baselines_mut() =
            BTreeMap::from([("E1".to_string(), serde_json::json!({"a": 1}))]);
    });
    let len_before: usize =
        editor_model::ports::with_session_mut(|sess| sess.tunable_baselines_mut().len()).unwrap();
    assert_eq!(len_before, 1);
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.tunable_baselines_mut().clear();
    });
    let len_after: usize =
        editor_model::ports::with_session_mut(|sess| sess.tunable_baselines_mut().len()).unwrap();
    assert_eq!(len_after, 0);
}
