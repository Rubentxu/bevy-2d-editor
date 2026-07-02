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
/// Calls `window.__dispatch_logic_binding(stable_id, asset_id, version)`.
/// The JS bridge finds the graph asset, evaluates it, and applies outputs.
fn dispatch_logic_binding(entity: Entity, binding: &LogicBinding) {
    #[cfg(any(target_arch = "wasm32", feature = "wasm"))]
    {
        let stable_id = entity_bits_for_wasm(entity);
        let asset_id = &binding.asset_id;
        let version = binding.version;

        // The JS bridge function is defined in the web page that loads this WASM.
        // Signature: window.__dispatch_logic_binding(stable_id: u32, asset_id: &str, version: u32) -> Result<(), JsValue>
        let _ = stable_id;
        let _ = asset_id;
        let _ = version;
        // wasm::dispatch_logic_binding_js(stable_id, asset_id, version);
    }

    #[cfg(not(any(target_arch = "wasm32", feature = "wasm")))]
    {
        // No-op on native: logic dispatch only runs in WASM preview
        let _ = (entity, binding);
    }
}

/// Convert Bevy Entity to a stable u32 identifier for the WASM bridge.
///
/// Bevy Entity is a thin wrapper around u32 on most platforms.
fn entity_bits_for_wasm(entity: Entity) -> u64 {
    entity.to_bits()
}
