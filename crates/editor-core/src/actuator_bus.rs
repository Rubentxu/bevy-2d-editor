//! Actuator Output Bus — typed command bus for logic actuator results.
//!
//! Actuator nodes produce outputs (field=value pairs) that need to be applied
//! back to Bevy entity components. This module provides:
//! - `ActuatorOutput`: the typed output payload
//! - `ACTUATOR_OUTPUT_BUS`: process-global OnceLock bus queue
//! - `submit_actuator_output()`: called by actuator evaluators
//! - `apply_actuator_outputs`: Bevy system that drains the bus and writes to components

use bevy::prelude::*;
use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::logic_evaluator::PortValue;

/// The output produced by an actuator node evaluation.
/// Carries the entity identifier (as u64 bits), the target field name,
/// and the typed value to write.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorOutput {
    /// Bevy Entity encoded as u64 bits (from `Entity::to_bits()`).
    pub entity_bits: u64,
    /// The field name on the target component to write (e.g., "translation", "color").
    pub field: String,
    /// The typed value to write to the field.
    pub value: PortValue,
}

/// Internal bus queue for actuator outputs.
/// Uses a simple Vec-based ring buffer pattern; `drain` atomically takes all pending outputs.
struct ActuatorBus {
    pending: Vec<ActuatorOutput>,
}

impl ActuatorBus {
    fn new() -> Self {
        Self { pending: Vec::new() }
    }

    /// Submit a single output to the bus.
    fn submit(&mut self, output: ActuatorOutput) {
        self.pending.push(output);
    }

    /// Atomically drain all pending outputs, leaving the bus empty.
    fn drain(&mut self) -> Vec<ActuatorOutput> {
        std::mem::take(&mut self.pending)
    }
}

/// Process-global actuator output bus — initialized on first use.
/// Uses Mutex for interior mutability (same pattern as `std::sync::OnceLock`).
static ACTUATOR_OUTPUT_BUS: OnceLock<Mutex<ActuatorBus>> = OnceLock::new();

fn actuator_bus() -> &'static Mutex<ActuatorBus> {
    ACTUATOR_OUTPUT_BUS.get_or_init(|| Mutex::new(ActuatorBus::new()))
}

/// Submit an actuator output to the global bus.
///
/// Called by actuator evaluators during graph evaluation to queue a component write.
pub fn submit_actuator_output(entity: Entity, field: &str, value: PortValue) {
    let output = ActuatorOutput {
        entity_bits: entity.to_bits(),
        field: field.to_string(),
        value,
    };
    if let Ok(mut bus) = actuator_bus().lock() {
        bus.submit(output);
    }
}

/// Drain all pending actuator outputs from the global bus.
///
/// Returns the collected outputs and leaves the bus empty.
/// Call this from `apply_actuator_outputs` at the start of each frame.
pub fn drain_actuator_outputs() -> Vec<ActuatorOutput> {
    match actuator_bus().lock() {
        Ok(mut bus) => bus.drain(),
        Err(_) => Vec::new(), // Poisoned — treat as empty
    }
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
