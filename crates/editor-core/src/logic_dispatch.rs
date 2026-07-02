//! Logic Evaluation Dispatch — Bevy system set for runtime logic graph evaluation.
//!
//! Phase 1: event/change-driven dispatch scheduler
//! Phase 2: compiled multi-dispatch with skip-arm optimization
//!
//! The scheduler queries all entities with a `LogicBinding` component and
//! dispatches their graphs through the WASM bridge (where the graph data lives).
//! Actuator outputs are applied back to Bevy component values.

use bevy::prelude::*;

use crate::logic_graph::LogicBinding;
use crate::logic_evaluator::LogicError;

/// Errors that can occur during logic dispatch.
#[derive(Debug, Clone)]
pub enum DispatchError {
    /// WASM bridge call failed.
    WasmBridge(String),
    /// Evaluation error from the logic evaluator.
    Evaluation(LogicError),
}

/// System set for logic evaluation.
///
/// Runs after `rebuild_preview_world` so that all `LogicBinding` entities
/// have been spawned and all sensor events have been emitted in the current frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogicSchedule;

/// Evaluate all logic-bound entities in the current frame.
///
/// Queries for `LogicBinding` and calls the WASM bridge to dispatch each graph.
/// The bridge handles node evaluation (sensors → controllers → actuators) and
/// writes actuator outputs back to Bevy component values on the entity.
pub fn logic_evaluation_system(
    bindings: Query<(Entity, &LogicBinding)>,
) {
    // WASM bridge dispatch: for each LogicBinding entity, call the bridge.
    // The bridge (JS side) holds the graph data and node evaluators.
    for (entity, binding) in &bindings {
        dispatch_logic_binding(entity, binding);
    }
}

/// Dispatch a single logic binding through the WASM bridge.
///
/// On wasm32: calls `evaluate_logic_binding_wasm(stable_id, asset_id, version)`.
/// On native: calls `evaluate_logic_binding(asset_id, version)`.
/// Actuator outputs are submitted to the ACTUATOR_OUTPUT_BUS.
fn dispatch_logic_binding(_entity: Entity, binding: &LogicBinding) {
    let asset_id = binding.asset_id.clone();
    let version = binding.version;

    #[cfg(target_arch = "wasm32")]
    {
        // Call the WASM-exported evaluator
        let stable_id = entity_bits_for_wasm(_entity);
        let result = crate::logic_evaluator::evaluate_logic_binding_wasm(
            stable_id as u32,
            &asset_id,
            version,
        );
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
        // On native, use the logic evaluator directly
        use crate::logic_evaluator::evaluate_logic_binding;
        if let Err(e) = evaluate_logic_binding(&asset_id, version) {
            bevy::log::warn!(
                "dispatch_logic_binding: evaluation error for asset '{}': {:?}",
                asset_id,
                e
            );
        }
    }
}

/// Convert Bevy Entity to a stable u32 identifier for the WASM bridge.
///
/// Bevy Entity is a thin wrapper around u32 on most platforms.
#[allow(dead_code)]
fn entity_bits_for_wasm(entity: Entity) -> u64 {
    entity.to_bits()
}
