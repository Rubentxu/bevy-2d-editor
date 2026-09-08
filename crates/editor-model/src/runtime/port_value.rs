//! Typed value boundary for logic evaluator ports.
//!
//! NO `serde_json::Value` inside — this is the strict contract.

use serde::{Deserialize, Serialize};

/// Typed value for logic evaluator input/output ports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortValue {
    /// Boolean value.
    Bool(bool),
    /// 32-bit float.
    Float(f32),
    /// 2D vector.
    Vec2 {
        /// X component.
        x: f32,
        /// Y component.
        y: f32,
    },
    /// Entity reference by stable ID string.
    EntityRef(String),
    /// Named action signal.
    Action(String),
}
