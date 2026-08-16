//! v0.90 PR4 — Sub-state types foundation (SceneSessionState, AssetSessionState,
//! LogicSessionState) added to `editor_model::session` and exposed through the
//! `EditorSessionPort` trait's new `scene_state_mut(path)`,
//! `asset_state_mut(path)`, `logic_state_mut(path)` methods.
//!
//! The actual thread_local removal (SCENE_DOC, SCENE_ASSET_CATALOG,
//! ASSET_OPERATION_LOG, LOGIC_GRAPH_DOC, LOGIC_OPERATION_LOG) is deferred to
//! v0.90 PR5 — this PR provides the seam.

use editor_model::{
    AssetSessionState, EditorSessionPort, LogicSessionState, SceneSessionState,
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

struct FakeSession {
    scene_states: BTreeMap<String, SceneSessionState>,
    asset_states: BTreeMap<String, AssetSessionState>,
    logic_states: BTreeMap<String, LogicSessionState>,
}

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
    fn tunable_baselines_mut(&mut self) -> &mut BTreeMap<String, serde_json::Value> {
        unimplemented!()
    }
    fn last_rebuild_cause_mut(&mut self) -> &mut Option<editor_model::RebuildCause> {
        unimplemented!()
    }
    fn pending_causality_edges_mut(
        &mut self,
    ) -> &mut BTreeMap<editor_model::StableId, Vec<editor_model::CausalityEdge>> {
        unimplemented!()
    }
    fn runtime_delta_buffer_mut(
        &mut self,
    ) -> &mut std::collections::VecDeque<editor_model::RuntimeDelta> {
        unimplemented!()
    }
}

fn fresh_session() {
    let session = FakeSession {
        scene_states: BTreeMap::new(),
        asset_states: BTreeMap::new(),
        logic_states: BTreeMap::new(),
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
    let count = editor_model::ports::with_session_mut(|s| s.scene_state_mut(&p).reload_count)
        .unwrap();
    assert_eq!(count, 1, "second call returns the same entry, not a fresh default");
}

#[test]
fn asset_state_mut_idempotent() {
    fresh_session();
    let p = "assets/level1/player.bsn".to_string();
    let _ = editor_model::ports::with_session_mut(|s| {
        s.asset_state_mut(&p).operation_log_bytes = vec![1, 2, 3];
    });
    let len = editor_model::ports::with_session_mut(|s| s.asset_state_mut(&p).operation_log_bytes.len())
        .unwrap();
    assert_eq!(len, 3, "second call returns the same map, not a fresh default");
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
    let len = editor_model::ports::with_session_mut(|s| s.logic_state_mut(&p).graph_docs.len())
        .unwrap();
    assert_eq!(len, 1, "second call returns the same map, not a fresh default");
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
