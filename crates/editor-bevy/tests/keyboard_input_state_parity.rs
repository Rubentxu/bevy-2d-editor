//! Parity tests for the Bevy Resource `InputState` (H2.5 Block H).
//!
//! Block H adds a Bevy `Resource InputState` as the canonical owner
//! of keyboard state, with `KEYBOARD_STATE_FALLBACK` as the
//! dual-write fallback (mirrors Blocks A2/D/E/F/G).
//!
//! These tests prove that:
//! - `InputState` is auto-initialized by Bevy (default empty set).
//! - `update_keyboard_state` populates the `InputState` Resource from
//!   `ButtonInput<KeyCode>`.
//! - Cleared/released keys are removed from the Resource.
//! - The system uses `Option<ResMut<InputState>>` so legacy tests
//!   that don't initialize the Resource still work.

use bevy::prelude::*;
use editor_bevy::keyboard_state::{InputState, update_keyboard_state};

fn fresh_app_with_resource() -> App {
    let mut app = App::new();
    app.init_resource::<InputState>();
    app.add_systems(Update, update_keyboard_state);
    app
}

// §S-H-1: InputState is auto-initialized with empty set
#[test]
fn parity_input_state_default_empty() {
    let mut app = App::new();
    app.init_resource::<InputState>();
    let input_state = app.world_mut().resource::<InputState>();
    assert!(
        input_state.held.is_empty(),
        "InputState should default to empty set"
    );
    assert!(!input_state.contains("KeyW"));
}

// §S-H-2: update_keyboard_state populates the Resource
#[test]
fn parity_dual_write_resource() {
    let mut app = fresh_app_with_resource();
    let mut input = ButtonInput::<KeyCode>::default();
    input.press(KeyCode::KeyW);
    input.press(KeyCode::Space);
    app.world_mut().insert_resource(input);

    app.update();

    let input_state = app.world_mut().resource::<InputState>();
    assert!(input_state.contains("KeyW"), "Resource should contain KeyW");
    assert!(input_state.contains("Space"), "Resource should contain Space");
}

// §S-H-2b: cleared/released keys are removed from Resource
#[test]
fn parity_released_keys_removed_from_resource() {
    let mut app = fresh_app_with_resource();
    let mut input = ButtonInput::<KeyCode>::default();
    input.press(KeyCode::KeyW);
    input.press(KeyCode::Space);
    app.world_mut().insert_resource(input);
    app.update();

    // Release KeyW
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.release(KeyCode::KeyW);
    }

    app.update();

    let input_state = app.world_mut().resource::<InputState>();
    assert!(
        !input_state.contains("KeyW"),
        "Resource should NOT contain KeyW after release"
    );
    assert!(
        input_state.contains("Space"),
        "Resource should still contain Space"
    );
}

// §S-H-2c: empty input → empty Resource
#[test]
fn parity_no_keys_pressed_clears_resource() {
    let mut app = fresh_app_with_resource();
    let input = ButtonInput::<KeyCode>::default();
    app.world_mut().insert_resource(input);

    app.update();

    let input_state = app.world_mut().resource::<InputState>();
    assert!(
        input_state.held.is_empty(),
        "Resource should be empty when no keys pressed"
    );
}

// §S-H-2d: backward compat — no Resource, system still runs (no panic)
#[test]
fn parity_legacy_fallback_only_when_resource_missing() {
    // App WITHOUT init_resource::<InputState>()
    let mut app = App::new();
    app.add_systems(Update, update_keyboard_state);
    let mut input = ButtonInput::<KeyCode>::default();
    input.press(KeyCode::KeyA);
    app.world_mut().insert_resource(input);

    app.update(); // Must NOT panic

    // After app.update(), the system ran. No Resource was set, so no
    // assert on Resource. The test passes if no panic occurred.
}
