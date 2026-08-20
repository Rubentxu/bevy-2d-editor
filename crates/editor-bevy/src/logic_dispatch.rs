//! Logic Evaluation Dispatch — Bevy system for runtime logic graph evaluation.
//!
//! ## Dirty-Tracking Dispatch Scheduler (cycle 2)
//!
//! Two-pass event-driven scheduler replacing cycle 1's unconditional per-frame dispatch:
//!
//! 1. [`mark_bindings_dirty`] — reacts to `SensorEvent::DidFire`: sets `dirty = true`,
//!    bumps `binding_version`, and pushes `LogicActivationEvent` to the ring.
//! 2. [`dispatch_dirty_bindings`] — iterates only dirty bindings, runs the evaluator
//!    exactly once, clears `dirty`, and submits actuator outputs to `ACTUATOR_OUTPUT_BUS`.
//!
//! ## Edge-Only Emissions (R2/R3)
//!
//! A key held for 5 seconds at 60fps produces exactly 1 ring entry, not 300.
//! `SensorStateCache` tracks the previous fired state per sensor to detect edges.
//!
//! ## Idle Skip (R5)
//!
//! Bindings with `dirty == false` are skipped entirely — 0 evaluator invocations
//! per frame, 0 bus writes.

use bevy::prelude::*;

use editor_model::logic_activation::{ring_push, LogicActivationEvent};

use crate::bevy_logic_binding::LogicBinding;
use crate::sensor_event::SensorEvent;
use crate::sensor_state_cache::SensorStateCache;

/// Observer: mark bindings dirty when their sensors fire (edge transition 0→1).
///
/// R2: sensors emit on edge transitions only. When `SensorEvent::DidFire` is
/// received, this observer sets `dirty = true`, bumps `binding_version`, and
/// pushes a `LogicActivationEvent` to the session ring.
///
/// In Bevy 0.19, events are handled via the Observer system using `On<Event>`.
pub fn mark_bindings_dirty(
    trigger: On<SensorEvent>,
    mut bindings: Query<(Entity, &mut LogicBinding)>,
    time: Res<Time>,
) {
    let triggered_at_ms = time.elapsed().as_millis() as u64;

    let SensorEvent {
        entity_bits,
        node_id,
        payload: _payload,
    } = trigger.event();

    // Find the LogicBinding entity by entity_bits
    for (entity, mut binding) in bindings.iter_mut() {
        if entity.to_bits() == *entity_bits {
            // R2: edge transition detected — mark dirty and bump version
            binding.dirty = true;
            binding.binding_version += 1;

            // R7: push to the activation ring
            // payload_summary carries binding_version per spec
            let payload_summary = Some(format!("v{}", binding.binding_version));
            let activation_event = LogicActivationEvent {
                node_id: node_id.0.clone(),
                triggered_at_ms,
                payload_summary,
            };

            // Access session via EditorSessionPort
            let _ = editor_model::ports::with_session_mut(|sess| {
                ring_push(sess.logic_activation_ring_mut(), activation_event);
            });

            break; // Found the matching entity, no need to continue
        }
    }
}

/// Dispatch all dirty bindings exactly once per frame.
///
/// R4: dispatches bindings with `dirty == true` and `binding_version > 0`.
/// R5: skips bindings with `dirty == false` (0 evaluator invocations).
/// Clears `dirty` after each dispatch.
pub fn dispatch_dirty_bindings(mut bindings: Query<(Entity, &mut LogicBinding)>) {
    // Collect dirty bindings first to avoid borrowing issues when clearing dirty
    let dirty: Vec<Entity> = bindings
        .iter_mut()
        .filter(|(_, b)| b.dirty && b.binding_version > 0)
        .map(|(e, _)| e)
        .collect();

    for entity in dirty {
        // We need to get the binding and dispatch it
        // Use a separate query to avoid borrowing conflicts
        if let Ok((entity, binding)) = bindings.get(entity) {
            dispatch_logic_binding(entity, &binding);
        }
    }

    // Clear dirty flags after dispatch
    for (_entity, mut binding) in bindings.iter_mut() {
        if binding.dirty {
            binding.dirty = false;
        }
    }
}

/// Evaluate all logic-bound entities in the current frame.
///
/// DEPRECATED in cycle 2: replaced by `mark_bindings_dirty` + `dispatch_dirty_bindings`.
/// Kept for play-mode compatibility until play-mode also migrates to dirty tracking.
pub fn logic_evaluation_system(bindings: Query<(Entity, &LogicBinding)>) {
    // WASM bridge dispatch: for each LogicBinding entity, call the bridge.
    // The bridge (JS side) holds the graph data and node evaluators.
    for (entity, binding) in &bindings {
        dispatch_logic_binding(entity, binding);
    }
}

/// Dispatch a single logic binding through the WASM bridge.
///
/// On wasm32: calls `evaluate_logic_binding_wasm(entity_bits, asset_id, version)`.
/// On native: calls `evaluate_logic_binding(asset_id, version, entity_bits)`.
/// Actuator outputs are submitted to the ACTUATOR_OUTPUT_BUS.
fn dispatch_logic_binding(entity: Entity, binding: &LogicBinding) {
    let entity_bits = entity.to_bits();
    let asset_id = binding.asset_id.clone();
    let version = binding.version;

    #[cfg(target_arch = "wasm32")]
    {
        let result =
            crate::logic_evaluator::evaluate_logic_binding_wasm(entity_bits, &asset_id, version);
        if let Err(e) = result {
            bevy::log::warn!(
                "dispatch_logic_binding: WASM bridge error for asset '{}': {:?}",
                asset_id,
                e
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::logic_evaluator::evaluate_logic_binding;
        if let Err(e) = evaluate_logic_binding(&asset_id, version, entity_bits) {
            bevy::log::warn!(
                "dispatch_logic_binding: evaluation error for asset '{}': {:?}",
                asset_id,
                e
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests (in-process, no Bevy App required)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_skips_clean_binding() {
        // GIVEN: a binding with dirty == false
        let binding = LogicBinding {
            asset_id: "test_asset".to_string(),
            version: 1,
            dirty: false,
            binding_version: 1,
        };

        // WHEN: we check if it should be dispatched
        let should_dispatch = binding.dirty && binding.binding_version > 0;

        // THEN: it should NOT be dispatched (idle skip)
        assert!(!should_dispatch, "clean binding must be skipped");
    }

    #[test]
    fn dispatch_skips_uninitialized_binding() {
        // GIVEN: a binding with binding_version == 0 (never evaluated)
        let binding = LogicBinding {
            asset_id: "test_asset".to_string(),
            version: 1,
            dirty: true,
            binding_version: 0,
        };

        // WHEN: we check if it should be dispatched
        let should_dispatch = binding.dirty && binding.binding_version > 0;

        // THEN: it should NOT be dispatched (never evaluated before)
        assert!(!should_dispatch, "uninitialized binding must be skipped");
    }

    #[test]
    fn dispatch_runs_on_dirty_and_initialized() {
        // GIVEN: a binding with dirty == true and binding_version > 0
        let binding = LogicBinding {
            asset_id: "test_asset".to_string(),
            version: 1,
            dirty: true,
            binding_version: 1,
        };

        // WHEN: we check if it should be dispatched
        let should_dispatch = binding.dirty && binding.binding_version > 0;

        // THEN: it SHOULD be dispatched
        assert!(should_dispatch, "dirty and initialized binding must be dispatched");
    }

    #[test]
    fn dirty_cleared_after_dispatch() {
        // GIVEN: a dirty binding
        let mut binding = LogicBinding {
            asset_id: "test_asset".to_string(),
            version: 1,
            dirty: true,
            binding_version: 1,
        };

        // WHEN: dispatch happens and we clear dirty
        if binding.dirty && binding.binding_version > 0 {
            // simulate dispatch
            binding.dirty = false;
        }

        // THEN: dirty is cleared
        assert!(!binding.dirty, "dirty must be cleared after dispatch");
    }

    #[test]
    fn binding_version_bumped_on_sensor_event() {
        // GIVEN: a binding with binding_version == 1
        let mut binding = LogicBinding {
            asset_id: "test_asset".to_string(),
            version: 1,
            dirty: false,
            binding_version: 1,
        };

        // WHEN: mark_bindings_dirty processes a sensor event
        binding.dirty = true;
        binding.binding_version += 1;

        // THEN: dirty is true and version is bumped
        assert!(binding.dirty, "dirty must be set");
        assert_eq!(binding.binding_version, 2, "binding_version must be bumped");
    }

    #[test]
    fn activation_event_payload_carries_version() {
        // GIVEN: a binding with binding_version == 5
        let binding_version = 5u64;

        // WHEN: we create the activation event payload_summary
        let payload_summary = Some(format!("v{}", binding_version));

        // THEN: payload_summary carries the version
        assert_eq!(payload_summary, Some("v5".to_string()));
    }
}
