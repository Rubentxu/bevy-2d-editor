//! Logic Evaluation Dispatch — Bevy system for runtime logic graph evaluation.
//!
//! Queries all entities with a `LogicBinding` component and dispatches their
//! graphs through the WASM bridge. Actuator outputs are applied back to Bevy
//! component values via the ACTUATOR_OUTPUT_BUS.

use bevy::prelude::*;

use crate::bevy_logic_binding::LogicBinding;

/// Evaluate all logic-bound entities in the current frame.
///
/// Queries for `LogicBinding` and calls the WASM bridge to dispatch each graph.
/// The bridge handles node evaluation (sensors → controllers → actuators) and
/// writes actuator outputs back to Bevy component values on the entity.
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
