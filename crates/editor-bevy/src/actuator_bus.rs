//! Actuator Output Bus — typed command bus for logic actuator results.
//!
//! H2.5 Block A2: bus is now session-owned via `editor_model::ports::with_session_mut`.
//! No thread-local cells remain. The session-owned `editor_model::runtime::ActuatorBus`
//! replaces the previous private `thread_local! ACTUATOR_OUTPUT_BUS`.
//!
//! - `ActuatorOutput` / `ActuatorBus`: typed payload + FIFO queue (in editor_model::runtime)
//! - `submit_actuator_output()`: called by actuator evaluators; pushes to session bus
//! - `apply_actuator_outputs`: Bevy system that drains the session bus and writes to components

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::logic_evaluator::PortValue;
pub use editor_model::runtime::{ActuatorBus, ActuatorOutput};

/// Submit an actuator output to the session-owned bus.
///
/// Called by actuator evaluators during graph evaluation to queue a component write.
/// Returns silently if no session is installed (e.g. tests that exercise the bus
/// without booting the full editor).
pub fn submit_actuator_output(entity: Entity, field: &str, value: PortValue) {
    let output = ActuatorOutput {
        entity_bits: entity.to_bits(),
        field: field.to_string(),
        value,
    };
    editor_model::ports::with_session_mut(|s| {
        s.runtime_actuator_outputs_mut().submit(output);
    });
}

/// Drain all pending actuator outputs from the session-owned bus.
///
/// Returns the collected outputs and leaves the bus empty.
/// Call this from `apply_actuator_outputs` at the start of each frame.
/// Returns `Vec::new()` if no session is installed.
pub fn drain_actuator_outputs() -> Vec<ActuatorOutput> {
    editor_model::ports::with_session_mut(|s| s.runtime_actuator_outputs_mut().drain())
        .unwrap_or_default()
}

/// Bevy system: drain the actuator output bus and write values back to entity components.
///
/// For each `ActuatorOutput`:
/// 1. Reconstruct the Bevy `Entity` from `entity_bits`
/// 2. Match on `field` name to write the typed `PortValue`
///
/// Currently supported field targets on known components:
/// - `Transform` + "translation" → `Vec3` (from `PortValue::Vec2 { x, y }`)
/// - `Transform` + "rotation" → `f32` (Z rotation in radians, from `PortValue::Float`)
/// - `Transform` + "scale" → `Vec3` (from `PortValue::Vec2 { x, y }` or `PortValue::Float` for uniform)
/// - `Sprite` + "color" → `Color` (simplified: uses `PortValue::Vec2` as {r, g})
///
/// Extension point: add more field matches as more actuator types land.
pub fn apply_actuator_outputs(
    mut transforms: Query<&mut Transform>,
    mut sprites: Query<&mut Sprite>,
) {
    let outputs = drain_actuator_outputs();
    if outputs.is_empty() {
        return;
    }

    // Group outputs by entity to minimize query overhead
    let mut by_entity: HashMap<Entity, Vec<&ActuatorOutput>> = HashMap::new();
    for output in &outputs {
        let entity = Entity::from_bits(output.entity_bits);
        by_entity.entry(entity).or_default().push(output);
    }

    for (entity, entity_outputs) in by_entity {
        for output in entity_outputs {
            apply_single_output(entity, output, &mut transforms, &mut sprites);
        }
    }
}

fn apply_single_output(
    entity: Entity,
    output: &ActuatorOutput,
    transforms: &mut Query<&mut Transform>,
    sprites: &mut Query<&mut Sprite>,
) {
    match output.field.as_str() {
        // ── Transform fields ────────────────────────────────────────────────
        "translation" => {
            if let Ok(mut t) = transforms.get_mut(entity) {
                if let PortValue::Vec2 { x, y } = &output.value {
                    t.translation.x = *x;
                    t.translation.y = *y;
                } else if let PortValue::Float(v) = &output.value {
                    t.translation.x = *v;
                }
            }
        }
        "rotation" => {
            if let Ok(mut t) = transforms.get_mut(entity) {
                if let PortValue::Float(r) = &output.value {
                    t.rotation = Quat::from_rotation_z(*r);
                }
            }
        }
        "scale" => {
            if let Ok(mut t) = transforms.get_mut(entity) {
                if let PortValue::Vec2 { x, y } = &output.value {
                    t.scale.x = *x;
                    t.scale.y = *y;
                } else if let PortValue::Float(s) = &output.value {
                    t.scale = Vec3::splat(*s);
                }
            }
        }

        // ── Sprite fields ─────────────────────────────────────────────────
        // Note: Color mutation via PortValue is simplified. Full rgba support
        // would require extending PortValue or a richer type.
        "color" => {
            if let Ok(mut s) = sprites.get_mut(entity) {
                if let PortValue::Vec2 { x: r, y: g } = &output.value {
                    // Vec2 interpreted as {r, g}; full support would need b and a
                    s.color = Color::srgba(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), 1.0, 1.0);
                } else if let PortValue::Float(v) = &output.value {
                    let intensity = v.clamp(0.0, 1.0);
                    s.color = Color::srgba(intensity, intensity, intensity, 1.0);
                }
            }
        }

        // Unknown field — no-op (extensible without breaking)
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Minimal stand-in implementing just enough of `EditorSessionPort` to
    /// drive `runtime_actuator_outputs_mut()` for the actuator bus tests.
    ///
    /// We cannot depend on `tests/support/mod.rs` from inside the lib (path
    /// resolution fails). The real `EditorSession` and the integration tests
    /// use the much fuller `FakeSession` harness; here we only need the
    /// single trait method exercised by `submit_actuator_output`/`drain_actuator_outputs`.
    struct MinimalSession {
        bus: editor_model::runtime::ActuatorBus,
    }

    impl MinimalSession {
        fn new() -> Self {
            Self {
                bus: editor_model::runtime::ActuatorBus::new(),
            }
        }
    }

    // Provide every `EditorSessionPort` method. Most are unused by the actuator
    // bus path; these stubs panic if reached, so accidental regressions surface
    // loudly. The only non-trivial one is `runtime_actuator_outputs_mut`.
    impl editor_model::EditorSessionPort for MinimalSession {
        fn scene_state_mut(&mut self, _: &str) -> &mut editor_model::SceneSessionState {
            unimplemented!("MinimalSession: scene_state_mut not supported")
        }
        fn active_scene_mut(&mut self) -> &mut editor_model::SceneFocus {
            unimplemented!("MinimalSession: active_scene_mut not supported")
        }
        fn active_asset_mut(&mut self) -> &mut editor_model::AssetFocus {
            unimplemented!("MinimalSession: active_asset_mut not supported")
        }
        fn asset_state_mut(&mut self, _: &str) -> &mut editor_model::AssetSessionState {
            unimplemented!("MinimalSession: asset_state_mut not supported")
        }
        fn logic_state_mut(&mut self, _: &str) -> &mut editor_model::LogicSessionState {
            unimplemented!("MinimalSession: logic_state_mut not supported")
        }
        fn world_state_mut(&mut self, _: &str) -> &mut editor_model::WorldSessionState {
            unimplemented!("MinimalSession: world_state_mut not supported")
        }
        fn tunable_baselines_mut(
            &mut self,
        ) -> &mut std::collections::BTreeMap<String, serde_json::Value> {
            unimplemented!("MinimalSession: tunable_baselines_mut not supported")
        }
        fn runtime_delta_buffer_mut(&mut self) -> &mut std::collections::VecDeque<editor_model::RuntimeDelta> {
            unimplemented!("MinimalSession: runtime_delta_buffer_mut not supported")
        }
        fn pending_causality_edges_mut(
            &mut self,
        ) -> &mut std::collections::BTreeMap<editor_model::StableId, Vec<editor_model::CausalityEdge>>
        {
            unimplemented!("MinimalSession: pending_causality_edges_mut not supported")
        }
        fn last_rebuild_cause_mut(&mut self) -> &mut Option<editor_model::RebuildCause> {
            unimplemented!("MinimalSession: last_rebuild_cause_mut not supported")
        }
        fn preview_inspector_mut(&mut self) -> &mut editor_model::PreviewInspectorState {
            unimplemented!("MinimalSession: preview_inspector_mut not supported")
        }
        fn source_files_mut(&mut self) -> &mut editor_model::SourceFilesCache {
            unimplemented!("MinimalSession: source_files_mut not supported")
        }
        fn logic_activation_ring_mut(
            &mut self,
        ) -> &mut std::collections::VecDeque<editor_model::LogicActivationEvent> {
            unimplemented!("MinimalSession: logic_activation_ring_mut not supported")
        }
        fn recent_change_sets_for(&self, _: &str) -> Vec<editor_model::ChangeSetSummary> {
            unimplemented!("MinimalSession: recent_change_sets_for not supported")
        }
        fn all_recent_change_sets(&self) -> Vec<editor_model::ChangeSetSummary> {
            unimplemented!("MinimalSession: all_recent_change_sets not supported")
        }
        fn active_document_path(&self) -> Option<&str> {
            unimplemented!("MinimalSession: active_document_path not supported")
        }
        fn push_recent_change_set(&mut self, _: &str, _: editor_model::ChangeSetSummary) {
            unimplemented!("MinimalSession: push_recent_change_set not supported")
        }
        fn runtime_command_bus_mut(&mut self) -> &mut editor_model::runtime::LinearBus {
            unimplemented!("MinimalSession: runtime_command_bus_mut not supported")
        }
        fn runtime_event_bus_mut(&mut self) -> &mut editor_model::runtime::LinearBus {
            unimplemented!("MinimalSession: runtime_event_bus_mut not supported")
        }
        fn runtime_actuator_outputs_mut(&mut self) -> &mut editor_model::runtime::ActuatorBus {
            &mut self.bus
        }
        fn runtime_hot_reload_requests_mut(
            &mut self,
        ) -> &mut Vec<editor_model::runtime::HotReloadRequest> {
            unimplemented!("MinimalSession: runtime_hot_reload_requests_mut not supported")
        }
        fn runtime_play_mode_request_mut(
            &mut self,
        ) -> &mut Option<editor_model::runtime::PlayModeRequest> {
            unimplemented!("MinimalSession: runtime_play_mode_request_mut not supported")
        }
    }

    fn install_fresh_session() {
        let session = MinimalSession::new();
        let arc: Arc<Mutex<dyn editor_model::EditorSessionPort>> =
            Arc::new(Mutex::new(session));
        editor_model::ports::register_editor_session(arc);
    }

    // §T-apply1: drain_actuator_outputs returns submitted outputs
    #[test]
    fn test_submit_and_drain_roundtrip() {
        use crate::logic_evaluator::PortValue;
        use bevy::prelude::Entity;

        install_fresh_session();

        let entity = Entity::from_bits(42);
        submit_actuator_output(entity, "translation", PortValue::Vec2 { x: 5.0, y: 7.0 });
        submit_actuator_output(entity, "scale", PortValue::Vec2 { x: 2.0, y: 3.0 });

        let outputs = drain_actuator_outputs();
        assert_eq!(outputs.len(), 2, "should have 2 outputs");

        let t = outputs.iter().find(|o| o.field == "translation").unwrap();
        assert_eq!(t.entity_bits, 42);
        assert!(matches!(t.value, PortValue::Vec2 { x: 5.0, y: 7.0 }));

        let s = outputs.iter().find(|o| o.field == "scale").unwrap();
        assert!(matches!(s.value, PortValue::Vec2 { x: 2.0, y: 3.0 }));
    }

    // §T-apply2: bus is empty after drain
    #[test]
    fn test_bus_empty_after_drain() {
        use crate::logic_evaluator::PortValue;
        use bevy::prelude::Entity;

        install_fresh_session();

        let entity = Entity::from_bits(1);
        submit_actuator_output(entity, "translation", PortValue::Vec2 { x: 1.0, y: 2.0 });

        let first = drain_actuator_outputs();
        assert_eq!(first.len(), 1);

        let second = drain_actuator_outputs();
        assert_eq!(second.len(), 0, "bus should be empty after drain");
    }
}
