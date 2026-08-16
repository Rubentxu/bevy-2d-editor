//! v0.90 PR1 (MUST) — RuntimeDelta wiring regression test (spec §3).
//!
//! This test lives in `editor-core` because `compute_runtime_deltas_internal`
//! is defined in `editor_core::preview_runtime` and `editor-application` does
//! not have a non-wasm32 dependency on `editor-core` (per ADR-0031/0032
//! dep direction). The test uses an inline fake `EditorSessionPort` impl to
//! avoid the cross-crate cycle.

use editor_model::EditorSessionPort;
use editor_model::RuntimeDelta;
use editor_model::StableId;
use editor_model::logic_activation::LogicActivationEvent;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

/// Inline fake session impl used by the test harness.
struct FakeSession {
    tunable_baselines: BTreeMap<String, serde_json::Value>,
    runtime_delta_buffer: VecDeque<RuntimeDelta>,
    pending_causality_edges: BTreeMap<StableId, Vec<editor_model::CausalityEdge>>,
    last_rebuild_cause: Option<editor_model::RebuildCause>,
    scene_states: BTreeMap<String, editor_model::SceneSessionState>,
    asset_states: BTreeMap<String, editor_model::AssetSessionState>,
    logic_states: BTreeMap<String, editor_model::LogicSessionState>,
    preview_inspector: editor_model::PreviewInspectorState,
    source_files: editor_model::SourceFilesCache,
    logic_activation_ring: VecDeque<LogicActivationEvent>,
    recent_change_sets: BTreeMap<String, Vec<editor_model::ChangeSetSummary>>,
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
    ) -> &mut BTreeMap<StableId, Vec<editor_model::CausalityEdge>> {
        &mut self.pending_causality_edges
    }
    fn runtime_delta_buffer_mut(&mut self) -> &mut VecDeque<RuntimeDelta> {
        while self.runtime_delta_buffer.len() > 64 {
            self.runtime_delta_buffer.pop_front();
        }
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
        self.recent_change_sets
            .get(scene_path)
            .cloned()
            .unwrap_or_default()
    }
    fn logic_activation_ring_mut(&mut self) -> &mut VecDeque<LogicActivationEvent> {
        &mut self.logic_activation_ring
    }

    fn all_recent_change_sets(&self) -> Vec<editor_model::ChangeSetSummary> {
        self.recent_change_sets
            .values()
            .flat_map(|v| v.iter().cloned())
            .collect()
    }
    fn push_recent_change_set(
        &mut self,
        scene_path: &str,
        summary: editor_model::ChangeSetSummary,
    ) {
        self.recent_change_sets
            .entry(scene_path.to_string())
            .or_insert_with(Vec::new)
            .push(summary);
    }
}

fn fresh_session() {
    let session = FakeSession {
        tunable_baselines: BTreeMap::new(),
        runtime_delta_buffer: VecDeque::with_capacity(64),
        pending_causality_edges: BTreeMap::new(),
        last_rebuild_cause: None,
        scene_states: BTreeMap::new(),
        asset_states: BTreeMap::new(),
        logic_states: BTreeMap::new(),
        preview_inspector: editor_model::PreviewInspectorState::default(),
        source_files: editor_model::SourceFilesCache::default(),
        recent_change_sets: BTreeMap::new(),
        logic_activation_ring: VecDeque::with_capacity(64),
    };
    let arc: Arc<Mutex<dyn EditorSessionPort>> = Arc::new(Mutex::new(session));
    editor_model::ports::register_editor_session(arc);
}

#[test]
fn register_session_via_trait_object() {
    fresh_session();
    let result = editor_model::ports::with_session_mut(|s| s.tunable_baselines_mut().len());
    assert!(result.is_some(), "session should be registered");
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn tunable_baselines_written_via_with_session_mut() {
    fresh_session();
    let mut baselines = BTreeMap::new();
    baselines.insert(
        "E1".to_string(),
        serde_json::json!({"editor.Transform2D": {"translation": {"x": 10.0}}}),
    );
    let _ = editor_model::ports::with_session_mut(|s| {
        *s.tunable_baselines_mut() = baselines;
    });
    let len = editor_model::ports::with_session_mut(|s| s.tunable_baselines_mut().len());
    assert_eq!(len, Some(1));
}

#[test]
fn runtime_delta_buffer_cap_holds_at_64() {
    fresh_session();
    for i in 0..70 {
        let _ = editor_model::ports::with_session_mut(|s| {
            s.runtime_delta_buffer_mut().push_back(RuntimeDelta {
                instance_id: format!("E{i}"),
                target_local_id: String::new(),
                component_type_id: "editor.Transform2D".to_string(),
                field_path: "translation.x".to_string(),
                baseline_value: serde_json::json!(0.0),
                runtime_value: serde_json::json!(1.0),
                captured_at_ms: i as u64,
                apply_back_eligible: true,
            });
        });
    }
    let len = editor_model::ports::with_session_mut(|s| s.runtime_delta_buffer_mut().len());
    assert_eq!(len, Some(64), "cap should hold at 64");
}

#[test]
fn compute_runtime_deltas_diff_finds_changed_field() {
    // Set baselines with 1 instance, 1 component, 2 fields.
    fresh_session();
    let mut baselines = BTreeMap::new();
    baselines.insert(
        "inst1".to_string(),
        serde_json::json!({
            "editor.Transform2D": {
                "translation": {"x": 10.0, "y": 5.0}
            }
        }),
    );
    let _ = editor_model::ports::with_session_mut(|s| {
        *s.tunable_baselines_mut() = baselines;
    });

    // Call the diff function with a fake runtime getter.
    use editor_core::document::ComponentInstance;
    let appended = editor_core::preview_runtime::compute_runtime_deltas_internal(
        |instance_id| {
            if instance_id == "inst1" {
                let mut values = serde_json::Map::new();
                let mut translation = serde_json::Map::new();
                translation.insert("x".to_string(), serde_json::json!(20.0));
                translation.insert("y".to_string(), serde_json::json!(5.0));
                let mut component_obj = serde_json::Map::new();
                component_obj.insert(
                    "translation".to_string(),
                    serde_json::Value::Object(translation),
                );
                values.insert(
                    "editor.Transform2D".to_string(),
                    serde_json::Value::Object(component_obj),
                );
                Some(ComponentInstance {
                    type_id: "editor.Transform2D".to_string(),
                    values: serde_json::Value::Object(values),
                })
            } else {
                None
            }
        },
        12345,
    );
    assert_eq!(appended, 1, "one field changed → one delta");

    let len = editor_model::ports::with_session_mut(|s| s.runtime_delta_buffer_mut().len());
    assert_eq!(len, Some(1));
    let first =
        editor_model::ports::with_session_mut(|s| s.runtime_delta_buffer_mut().front().cloned())
            .unwrap();
    let delta = first.expect("at least one delta");
    assert_eq!(delta.instance_id, "inst1");
    assert_eq!(delta.component_type_id, "editor.Transform2D");
    assert_eq!(delta.field_path, "translation.x");
    assert_eq!(delta.baseline_value, serde_json::json!(10.0));
    assert_eq!(delta.runtime_value, serde_json::json!(20.0));
    // v0.91 PR1: editor.Transform2D's built-in seed has apply_back: Never
    // (per D4 conservative default). The delta is captured but NOT eligible.
    // The ApplyBackPanel filters out ineligible deltas at display time.
    assert!(
        !delta.apply_back_eligible,
        "editor.Transform2D's built-in policy is Never — delta must be ineligible"
    );
    assert_eq!(delta.captured_at_ms, 12345);
}

#[test]
fn compute_runtime_deltas_unchanged_returns_zero() {
    fresh_session();
    let mut baselines = BTreeMap::new();
    baselines.insert(
        "inst1".to_string(),
        serde_json::json!({
            "editor.Transform2D": {"translation": {"x": 10.0}}
        }),
    );
    let _ = editor_model::ports::with_session_mut(|s| {
        *s.tunable_baselines_mut() = baselines;
    });

    use editor_core::document::ComponentInstance;
    let appended = editor_core::preview_runtime::compute_runtime_deltas_internal(
        |instance_id| {
            if instance_id == "inst1" {
                let mut values = serde_json::Map::new();
                let mut translation = serde_json::Map::new();
                translation.insert("x".to_string(), serde_json::json!(10.0));
                let mut component_obj = serde_json::Map::new();
                component_obj.insert(
                    "translation".to_string(),
                    serde_json::Value::Object(translation),
                );
                values.insert(
                    "editor.Transform2D".to_string(),
                    serde_json::Value::Object(component_obj),
                );
                Some(ComponentInstance {
                    type_id: "editor.Transform2D".to_string(),
                    values: serde_json::Value::Object(values),
                })
            } else {
                None
            }
        },
        12345,
    );
    assert_eq!(appended, 0);
    let len = editor_model::ports::with_session_mut(|s| s.runtime_delta_buffer_mut().len());
    assert_eq!(len, Some(0));
}
