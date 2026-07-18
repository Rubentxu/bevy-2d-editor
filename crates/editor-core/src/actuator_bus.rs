//! Actuator Output Bus — typed command bus for logic actuator results.
//!
//! Actuator nodes produce outputs (field=value pairs) that need to be applied
//! back to Bevy entity components. This module provides:
//! - `ActuatorOutput`: the typed output payload
//! - `ACTUATOR_OUTPUT_BUS`: thread-local bus queue
//! - `submit_actuator_output()`: called by actuator evaluators
//! - `apply_actuator_outputs`: Bevy system that drains the bus and writes to components

use bevy::prelude::*;
use std::cell::RefCell;
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
struct ActuatorBus {
    pending: Vec<ActuatorOutput>,
}

impl ActuatorBus {
    fn new() -> Self {
        Self { pending: Vec::new() }
    }

    fn submit(&mut self, output: ActuatorOutput) {
        self.pending.push(output);
    }

    fn drain(&mut self) -> Vec<ActuatorOutput> {
        std::mem::take(&mut self.pending)
    }
}

// Thread-local actuator output bus — matches the codebase `COMMAND_BUS`/`EVENT_BUS` pattern.
thread_local! {
    static ACTUATOR_OUTPUT_BUS: RefCell<Option<ActuatorBus>> = const { RefCell::new(None) };
}

fn actuator_bus() {
    ACTUATOR_OUTPUT_BUS.with(|b| {
        if b.borrow().is_none() {
            *b.borrow_mut() = Some(ActuatorBus::new());
        }
    });
}

/// Submit an actuator output to the thread-local bus.
///
/// Called by actuator evaluators during graph evaluation to queue a component write.
pub fn submit_actuator_output(entity: Entity, field: &str, value: PortValue) {
    let output = ActuatorOutput {
        entity_bits: entity.to_bits(),
        field: field.to_string(),
        value,
    };
    actuator_bus();
    ACTUATOR_OUTPUT_BUS.with(|b| {
        if let Some(ref mut bus) = *b.borrow_mut() {
            bus.submit(output);
        }
    });
}

/// Drain all pending actuator outputs from the thread-local bus.
///
/// Returns the collected outputs and leaves the bus empty.
/// Call this from `apply_actuator_outputs` at the start of each frame.
pub fn drain_actuator_outputs() -> Vec<ActuatorOutput> {
    actuator_bus();
    ACTUATOR_OUTPUT_BUS.with(|b| {
        if let Some(ref mut bus) = *b.borrow_mut() {
            bus.drain()
        } else {
            Vec::new()
        }
    })
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

    // §T-apply1: drain_actuator_outputs returns submitted outputs
    #[test]
    fn test_submit_and_drain_roundtrip() {
        use bevy::prelude::Entity;
        use crate::logic_evaluator::PortValue;

        // Drain any pre-existing state
        let _ = drain_actuator_outputs();

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
        use bevy::prelude::Entity;
        use crate::logic_evaluator::PortValue;

        let entity = Entity::from_bits(1);
        submit_actuator_output(entity, "translation", PortValue::Vec2 { x: 1.0, y: 2.0 });

        let first = drain_actuator_outputs();
        assert_eq!(first.len(), 1);

        let second = drain_actuator_outputs();
        assert_eq!(second.len(), 0, "bus should be empty after drain");
    }
}
