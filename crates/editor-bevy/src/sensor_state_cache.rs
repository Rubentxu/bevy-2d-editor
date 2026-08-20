//! Sensor state cache — tracks previous sensor output to detect edge transitions.
//!
//! R2/R3: sensors emit on edge transitions only. A key held for 5 seconds at 60fps
//! MUST produce 1 event, not 300. The cache stores `bool` per (Entity, NodeId).
//!
//! Memory bounded by `entities × sensors_per_binding`. Cleared on entity despawn.
//! Reused in cycle 3 for value-diffing.

use std::collections::HashMap;

use bevy::prelude::Resource;
use editor_model::logic_graph::NodeId;

/// Caches the last-known fired state of each sensor node per entity.
///
/// Used by `evaluate_sensor_node` to detect edge transitions:
/// - `was_fired == false` AND `is_fired == true` → edge 0→1 → emit `SensorEvent::DidFire`
/// - All other transitions (1→0, 1→1, 0→0) → no event
#[derive(Resource, Debug, Default)]
pub struct SensorStateCache {
    /// Per-entity sensor state. Inner `HashMap` is keyed by `NodeId`.
    /// Outer key is `Entity` encoded as u64 bits.
    state: HashMap<u64, HashMap<NodeId, bool>>,
}

impl SensorStateCache {
    /// Get the previous fired state for a sensor.
    ///
    /// Returns `false` if no prior state exists (first evaluation).
    pub fn get_previous(&self, entity_bits: u64, node_id: &NodeId) -> bool {
        self.state
            .get(&entity_bits)
            .and_then(|sensors| sensors.get(node_id))
            .copied()
            .unwrap_or(false)
    }

    /// Update the cached state for a sensor after evaluation.
    pub fn update(&mut self, entity_bits: u64, node_id: NodeId, is_fired: bool) {
        self.state
            .entry(entity_bits)
            .or_default()
            .insert(node_id, is_fired);
    }

    /// Returns whether this is an edge transition (0→1).
    ///
    /// Call AFTER `update` to advance the state.
    pub fn was_edge_fire(prev: bool, curr: bool) -> bool {
        !prev && curr
    }

    /// Remove all cached state for an entity (called on despawn).
    pub fn remove_entity(&mut self, entity_bits: u64) {
        self.state.remove(&entity_bits);
    }

    /// Clear all cached state (used in tests).
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.state.clear();
    }
}
