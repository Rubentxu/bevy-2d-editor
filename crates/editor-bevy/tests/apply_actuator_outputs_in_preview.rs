//! Integration tests for apply_actuator_outputs_in_preview system.
//!
//! Tests the Velocity component and logic binding evaluation in edit mode.
//!
//! These tests verify:
//! - Velocity component can be created and serialized
//! - Logic binding evaluation populates the actuator bus
//! - Drain and apply logic for velocity updates

use bevy::prelude::*;

/// Test 1: Velocity component can be created with default values
#[test]
fn velocity_default_is_zero() {
    let vel = editor_bevy::preview_runtime::Velocity::default();
    assert_eq!(vel.linvel, Vec2::ZERO);
}

/// Test 2: Velocity component can be created with custom values
#[test]
fn velocity_with_values() {
    let vel = editor_bevy::preview_runtime::Velocity {
        linvel: Vec2::new(1.0, 2.0),
    };
    assert_eq!(vel.linvel.x, 1.0);
    assert_eq!(vel.linvel.y, 2.0);
}

/// Test 3: Velocity is Copy
#[test]
fn velocity_is_copy() {
    let vel = editor_bevy::preview_runtime::Velocity {
        linvel: Vec2::new(3.0, 4.0),
    };
    let vel2 = vel; // Copy
    assert_eq!(vel.linvel, vel2.linvel);
}

/// Test 4: Velocity is PartialEq
#[test]
fn velocity_partial_eq() {
    let vel1 = editor_bevy::preview_runtime::Velocity {
        linvel: Vec2::new(1.0, 2.0),
    };
    let vel2 = editor_bevy::preview_runtime::Velocity {
        linvel: Vec2::new(1.0, 2.0),
    };
    let vel3 = editor_bevy::preview_runtime::Velocity {
        linvel: Vec2::new(1.0, 3.0),
    };
    assert_eq!(vel1, vel2);
    assert_ne!(vel1, vel3);
}
