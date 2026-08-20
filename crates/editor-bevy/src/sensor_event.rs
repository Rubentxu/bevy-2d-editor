//! Sensor event — edge-transition event emitted when a sensor fires.
//!
//! R2: sensors emit on edge transitions only (0→1 = fire, 1→0 = no fire, 1→1 = no fire).
//! The `SensorStateCache` resource tracks the previous state to detect edges.

use bevy::prelude::Event;
use editor_model::logic_graph::NodeId;
use serde::{Deserialize, Serialize};

use crate::logic_evaluator::PortValue;

/// Event emitted when a sensor fires (edge transition 0→1).
///
/// Emitted by `evaluate_sensor_node` when the cached state shows the sensor
/// transitioned from not-fired to fired. The `SensorStateCache` resource
/// stores the previous state to detect edges.
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub struct SensorEvent {
    /// Entity bits (from `Entity::to_bits()`) of the entity carrying the sensor.
    pub entity_bits: u64,
    /// The node ID of the sensor node that fired.
    pub node_id: NodeId,
    /// The sampled port value that caused the fire.
    /// Carried for cycle 3 value-diffing; `payload_summary` already covers serialization.
    pub payload: Option<PortValue>,
}
