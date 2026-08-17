//! v0.90 PR4 — Sub-state types foundation (SceneSessionState, AssetSessionState,
//! LogicSessionState) added to `editor_model::session` and exposed through the
//! `EditorSessionPort` trait's new `scene_state_mut(path)`,
//! `asset_state_mut(path)`, `logic_state_mut(path)` methods.
//!
//! The actual thread_local removal (SCENE_DOC, SCENE_ASSET_CATALOG,
//! ASSET_OPERATION_LOG, LOGIC_GRAPH_DOC, LOGIC_OPERATION_LOG) is deferred to
//! v0.90 PR5 — this PR provides the seam.

#[path = "support/mod.rs"]
mod support;

use editor_model::EditorSessionPort;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

// Define only the fields needed for this test's custom logic.
// All 11 fields exist in support::FakeSession but we declare only the 4 we use.
struct FakeSession {
    scene_states: BTreeMap<String, support::SceneSessionState>,
    asset_states: BTreeMap<String, support::AssetSessionState>,
    logic_states: BTreeMap<String, support::LogicSessionState>,
    recent_change_sets: BTreeMap<String, Vec<support::ChangeSetSummary>>,
}

impl EditorSessionPort for FakeSession {
    fn scene_state_mut(&mut self, path: &str) -> &mut support::SceneSessionState {
        self.scene_states
            .entry(path.to_string())
            .or_insert_with(support::SceneSessionState::default)
    }
    fn asset_state_mut(&mut self, path: &str) -> &mut support::AssetSessionState {
        self.asset_states
            .entry(path.to_string())
            .or_insert_with(support::AssetSessionState::default)
    }
    fn logic_state_mut(&mut self, path: &str) -> &mut support::LogicSessionState {
        self.logic_states
            .entry(path.to_string())
            .or_insert_with(support::LogicSessionState::default)
    }
    fn recent_change_sets_for(&self, _scene_path: &str) -> Vec<support::ChangeSetSummary> {
        Vec::new()
    }
    fn tunable_baselines_mut(&mut self) -> &mut BTreeMap<String, serde_json::Value> {
        unimplemented!()
    }
    fn last_rebuild_cause_mut(&mut self) -> &mut Option<support::RebuildCause> {
        unimplemented!()
    }
    fn pending_causality_edges_mut(
        &mut self,
    ) -> &mut BTreeMap<support::StableId, Vec<support::CausalityEdge>> {
        unimplemented!()
    }
    fn runtime_delta_buffer_mut(
        &mut self,
    ) -> &mut std::collections::VecDeque<support::RuntimeDelta> {
        unimplemented!()
    }
    fn preview_inspector_mut(&mut self) -> &mut support::PreviewInspectorState {
        unimplemented!()
    }
    fn source_files_mut(&mut self) -> &mut support::SourceFilesCache {
        unimplemented!()
    }
    fn logic_activation_ring_mut(
        &mut self,
    ) -> &mut std::collections::VecDeque<support::LogicActivationEvent> {
        unimplemented!()
    }
    fn all_recent_change_sets(&self) -> Vec<support::ChangeSetSummary> {
        self.recent_change_sets
            .values()
            .flat_map(|v| v.iter().cloned())
            .collect()
    }
    fn active_document_path(&self) -> Option<&str> {
        None
    }
    fn push_recent_change_set(
        &mut self,
        scene_path: &str,
        summary: support::ChangeSetSummary,
    ) {
        self.recent_change_sets
            .entry(scene_path.to_string())
            .or_insert_with(Vec::new)
            .push(summary);
    }
}

fn fresh_session() {
    let session = FakeSession {
        scene_states: BTreeMap::new(),
        asset_states: BTreeMap::new(),
        logic_states: BTreeMap::new(),
        recent_change_sets: BTreeMap::new(),
    };
    let arc: Arc<Mutex<dyn EditorSessionPort>> = Arc::new(Mutex::new(session));
    editor_model::ports::register_editor_session(arc);
}

#[test]
fn scene_state_mut_idempotent() {
    fresh_session();
    let p = "scenes/level1.bsn".to_string();
    // First call inserts the default; second call returns the same entry.
    let _ = editor_model::ports::with_session_mut(|s| {
        s.scene_state_mut(&p).reload_count = 1;
    });
    let count =
        editor_model::ports::with_session_mut(|s| s.scene_state_mut(&p).reload_count).unwrap();
    assert_eq!(
        count, 1,
        "second call returns the same entry, not a fresh default"
    );
}

#[test]
fn asset_state_mut_idempotent() {
    fresh_session();
    let p = "assets/level1/player.bsn".to_string();
    let _ = editor_model::ports::with_session_mut(|s| {
        // First call creates the state; second call returns the same entry.
        s.asset_state_mut(&p).catalog_warnings.push(
            editor_model::scene_asset_catalog::CatalogWarning::MissingComponentSchema {
                entity_id: "E1".to_string(),
                component_type_id: "Transform2D".to_string(),
            },
        );
    });
    let warnings_len = editor_model::ports::with_session_mut(|s| {
        s.asset_state_mut(&p).catalog_warnings.len()
    }).unwrap();
    assert_eq!(
        warnings_len, 1,
        "second call returns the same state entry, not a fresh default"
    );
}

#[test]
fn logic_state_mut_idempotent() {
    fresh_session();
    let p = "logic/player_movement.lg".to_string();
    let _ = editor_model::ports::with_session_mut(|s| {
        s.logic_state_mut(&p)
            .graph_docs
            .insert("g1".to_string(), editor_model::LogicGraphAsset::default());
    });
    let len =
        editor_model::ports::with_session_mut(|s| s.logic_state_mut(&p).graph_docs.len()).unwrap();
    assert_eq!(
        len, 1,
        "second call returns the same map, not a fresh default"
    );
}

#[test]
fn separate_paths_get_separate_states() {
    fresh_session();
    let a = "scenes/a.bsn".to_string();
    let b = "scenes/b.bsn".to_string();
    let _ = editor_model::ports::with_session_mut(|s| {
        s.scene_state_mut(&a).reload_count = 1;
    });
    let _ = editor_model::ports::with_session_mut(|s| {
        s.scene_state_mut(&b).reload_count = 99;
    });
    let ca = editor_model::ports::with_session_mut(|s| s.scene_state_mut(&a).reload_count).unwrap();
    let cb = editor_model::ports::with_session_mut(|s| s.scene_state_mut(&b).reload_count).unwrap();
    assert_eq!(ca, 1);
    assert_eq!(cb, 99);
}

// v0.90 PR5: New EditorSessionPort methods (preview_inspector_mut,
// source_files_mut, recent_change_sets_for, logic_activation_ring_mut).
// The FakeSession must implement them all.

use editor_model::ChangeSetSummary;
use editor_model::logic_activation::{LogicActivationEvent, LogicActivationRing};
