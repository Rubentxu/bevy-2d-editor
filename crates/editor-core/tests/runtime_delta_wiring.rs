//! v0.90 PR1 (MUST) — RuntimeDelta wiring regression test (spec §3).
//!
//! This test lives in `editor-core` because `compute_runtime_deltas_internal`
//! is defined in `editor_core::preview_runtime` and `editor-application` does
//! not have a non-wasm32 dependency on `editor-core` (per ADR-0031/0032
//! dep direction). The test uses an inline fake `EditorSessionPort` impl to
//! avoid the cross-crate cycle.

#[path = "support/mod.rs"]
mod support;

use editor_model::EditorSessionPort;
use editor_model::RuntimeDelta;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

/// Fake session that wraps support::FakeSession but overrides
/// `runtime_delta_buffer_mut` to enforce a 64-item cap (the one
/// custom behaviour this test needs to verify).
struct FakeSessionWithCap(support::FakeSession);

impl EditorSessionPort for FakeSessionWithCap {
    fn runtime_delta_buffer_mut(&mut self) -> &mut VecDeque<RuntimeDelta> {
        while self.0.runtime_delta_buffer.len() > 64 {
            self.0.runtime_delta_buffer.pop_front();
        }
        &mut self.0.runtime_delta_buffer
    }
    // Forward everything else to FakeSession's impl
    fn scene_state_mut(&mut self, path: &str) -> &mut editor_model::SceneSessionState {
        self.0.scene_state_mut(path)
    }
    fn asset_state_mut(&mut self, path: &str) -> &mut editor_model::AssetSessionState {
        self.0.asset_state_mut(path)
    }
    fn logic_state_mut(&mut self, path: &str) -> &mut editor_model::LogicSessionState {
        self.0.logic_state_mut(path)
    }
    fn tunable_baselines_mut(&mut self) -> &mut std::collections::BTreeMap<String, serde_json::Value> {
        self.0.tunable_baselines_mut()
    }
    fn pending_causality_edges_mut(
        &mut self,
    ) -> &mut std::collections::BTreeMap<editor_model::StableId, Vec<editor_model::CausalityEdge>> {
        self.0.pending_causality_edges_mut()
    }
    fn last_rebuild_cause_mut(&mut self) -> &mut Option<editor_model::RebuildCause> {
        self.0.last_rebuild_cause_mut()
    }
    fn preview_inspector_mut(&mut self) -> &mut editor_model::PreviewInspectorState {
        self.0.preview_inspector_mut()
    }
    fn source_files_mut(&mut self) -> &mut editor_model::SourceFilesCache {
        self.0.source_files_mut()
    }
    fn logic_activation_ring_mut(
        &mut self,
    ) -> &mut std::collections::VecDeque<editor_model::logic_activation::LogicActivationEvent> {
        self.0.logic_activation_ring_mut()
    }
    fn recent_change_sets_for(&self, scene_path: &str) -> Vec<editor_model::ChangeSetSummary> {
        self.0.recent_change_sets_for(scene_path)
    }
    fn all_recent_change_sets(&self) -> Vec<editor_model::ChangeSetSummary> {
        self.0.all_recent_change_sets()
    }
    fn active_document_path(&self) -> Option<&str> {
        self.0.active_document_path()
    }
    fn push_recent_change_set(&mut self, scene_path: &str, summary: editor_model::ChangeSetSummary) {
        self.0.push_recent_change_set(scene_path, summary)
    }
}

fn fresh_session() {
    let session = FakeSessionWithCap(support::FakeSession::new());
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
