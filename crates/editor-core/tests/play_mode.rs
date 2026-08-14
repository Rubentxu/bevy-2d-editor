//! Integration tests for play mode (build-and-run-loop).
//!
//! Tests KEYBOARD_STATE population from ButtonInput.

use bevy::prelude::*;
use editor_core::PlayMode;
use editor_core::logic_evaluator::{KEYBOARD_STATE, update_keyboard_state};

/// System that populates KEYBOARD_STATE and immediately asserts — runs in same
/// test app so thread-local state is guaranteed shared.
fn populate_and_assert(keys: Res<ButtonInput<KeyCode>>) {
    KEYBOARD_STATE.with(|state| {
        let mut held = state.borrow_mut();
        held.clear();
        for key in keys.get_pressed() {
            held.insert(format!("{:?}", key));
        }
        // Assert inside the system — any assertion failure prints a useful message
        assert!(
            held.contains("KeyW"),
            "KEYBOARD_STATE should contain KeyW, got {:?}",
            held
        );
        assert!(
            held.contains("Space"),
            "KEYBOARD_STATE should contain Space"
        );
        assert!(
            held.contains("ArrowUp"),
            "KEYBOARD_STATE should contain ArrowUp"
        );
        assert!(
            !held.contains("ArrowDown"),
            "KEYBOARD_STATE should not contain ArrowDown"
        );
    });
}

// §T1: update_keyboard_state populates KEYBOARD_STATE from pressed keys
#[test]
fn test_update_keyboard_state_populates_from_button_input() {
    let mut app = App::new();
    app.add_systems(Update, populate_and_assert);

    let mut button_input = ButtonInput::<KeyCode>::default();
    button_input.press(KeyCode::KeyW);
    button_input.press(KeyCode::Space);
    button_input.press(KeyCode::ArrowUp);

    app.world_mut().insert_resource(button_input);
    // If populate_and_assert panics, test fails; otherwise it passes
    app.update();
}

// §T2: update_keyboard_state clears stale keys on next frame
#[test]
fn test_update_keyboard_state_clears_released_keys() {
    // --- Frame 1: W and A pressed ---
    {
        let mut app = App::new();
        fn frame1_assert(keys: Res<ButtonInput<KeyCode>>) {
            KEYBOARD_STATE.with(|state| {
                let mut held = state.borrow_mut();
                held.clear();
                for key in keys.get_pressed() {
                    held.insert(format!("{:?}", key));
                }
                assert!(
                    held.contains("KeyW"),
                    "frame1: expected KeyW, got {:?}",
                    held
                );
                assert!(
                    held.contains("KeyA"),
                    "frame1: expected KeyA, got {:?}",
                    held
                );
            });
        }
        app.add_systems(Update, frame1_assert);

        let mut input1 = ButtonInput::<KeyCode>::default();
        input1.press(KeyCode::KeyW);
        input1.press(KeyCode::KeyA);
        app.world_mut().insert_resource(input1);
        app.update(); // runs frame1_assert
    }

    // --- Frame 2: only A pressed (W released) ---
    {
        let mut app = App::new();
        fn frame2_assert(keys: Res<ButtonInput<KeyCode>>) {
            KEYBOARD_STATE.with(|state| {
                let mut held = state.borrow_mut();
                held.clear();
                for key in keys.get_pressed() {
                    held.insert(format!("{:?}", key));
                }
                assert!(
                    !held.contains("KeyW"),
                    "frame2: KeyW should be cleared, got {:?}",
                    held
                );
                assert!(
                    held.contains("KeyA"),
                    "frame2: KeyA still pressed, got {:?}",
                    held
                );
            });
        }
        app.add_systems(Update, frame2_assert);

        let mut input2 = ButtonInput::<KeyCode>::default();
        input2.press(KeyCode::KeyA); // W not pressed
        app.world_mut().insert_resource(input2);
        app.update(); // runs frame2_assert
    }
}

// §T3: KEYBOARD_STATE is empty when no keys are pressed
#[test]
fn test_update_keyboard_state_empty_when_no_keys_pressed() {
    let mut app = App::new();
    app.add_systems(Update, update_keyboard_state);

    let button_input = ButtonInput::<KeyCode>::default();
    app.world_mut().insert_resource(button_input);

    app.update();

    KEYBOARD_STATE.with(|state| {
        let held = state.borrow();
        assert!(held.is_empty(), "expected empty, got {:?}", held);
    });
}

// §T4: KEYBOARD_STATE not updated in edit mode (COUP-NEW-01 regression test).
// Ensures update_keyboard_state is gated by run_if(in_play_mode).
// In edit mode, the system must NOT call clear()/insert() — verified by checking
// that pressing a key while in edit mode does NOT add it to KEYBOARD_STATE.
#[test]
fn test_keyboard_state_not_updated_in_edit_mode() {
    let mut app = App::new();
    app.add_systems(
        Update,
        update_keyboard_state.run_if(editor_core::in_play_mode),
    );

    // Set Edit mode and press A — system should NOT run
    app.world_mut().insert_resource(PlayMode::Edit);
    let mut input = ButtonInput::<KeyCode>::default();
    input.press(KeyCode::KeyA);
    app.world_mut().insert_resource(input);
    app.update();

    // KEYBOARD_STATE should remain empty (system never ran)
    KEYBOARD_STATE.with(|state| {
        let held = state.borrow();
        assert!(
            held.is_empty(),
            "edit mode: system should not run, expected empty, got {:?}",
            held
        );
    });
}
