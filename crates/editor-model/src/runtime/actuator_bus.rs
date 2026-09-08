//!
//! Actuator output bus — pure (no Bevy, no WASM) types.
//!
//! Actuator nodes produce outputs (field=value pairs) that need to be applied
//! back to Bevy entity components. The pure ActuatorBus lives here in
//! editor-model so EditorSession can own it without depending on editor-bevy.

use super::port_value::PortValue;
use serde::{Deserialize, Serialize};

/// The output produced by an actuator node evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorOutput {
    /// Bevy Entity encoded as u64 bits (from Entity::to_bits()).
    pub entity_bits: u64,
    /// The field name on the target component to write.
    pub field: String,
    /// The typed value to write to the field.
    pub value: PortValue,
}

/// Internal bus queue for actuator outputs.
#[derive(Debug, Clone, Default)]
pub struct ActuatorBus {
    pending: Vec<ActuatorOutput>,
}

impl ActuatorBus {
    /// Construct a new empty actuator output bus.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    /// Push one actuator output onto the pending queue.
    pub fn submit(&mut self, output: ActuatorOutput) {
        self.pending.push(output);
    }

    /// Drain all pending outputs and return them.
    pub fn drain(&mut self) -> Vec<ActuatorOutput> {
        std::mem::take(&mut self.pending)
    }
}
