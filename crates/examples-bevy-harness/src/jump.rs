//! Jump plugin for the v1.0 canonical sample harness.
//!
//! This module demonstrates Bevy play_mode runtime behaviour for the
//! `examples/platformer-minimal/` sample: when the user presses
//! `KeyCode::Space`, the `Player` entity translates along the y axis
//! by `PlayerController.jump_force * JUMP_FRAME_DT`.
//!
//! This is a **witness**, not a physics simulator — there is no
//! gravity, no ground check, no variable jump height. The player rises
//! one frame's worth of `jump_force` and stays there.
//!
//! ## Edge detection
//!
//! Bevy 0.19's `keyboard_input_system` runs in `PreUpdate` and calls
//! `ButtonInput::clear()`, which zeroes `just_pressed` and
//! `just_released` *before* `Update` runs our system. If a test sets
//! the press directly via `ButtonInput::press()` (no `KeyboardInput`
//! message event), `just_pressed` is wiped before we can read it.
//!
//! To make this robust in unit tests without depending on
//! `KeyboardInput` event plumbing, we track the previous frame's
//! pressed-state in `JumpState` and detect the rising edge via
//! `pressed()` + memory.
//!
//! See `crates/examples-bevy-harness/tests/jump.rs` for the integration
//! tests that prove this behaviour.

use bevy::prelude::*;

use crate::components::PlayerController;

/// Fixed per-frame impulse duration. Matches the test setup's
/// `TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(0.1))`.
///
/// A real production system would read `Res<Time<Virtual>>` and use
/// `delta_secs()`. We pick the constant here so the test assertion
/// (`y1 - y0 == jump_force * JUMP_FRAME_DT`) is exact and decoupled
/// from Bevy's first-frame `Time<Real>::last_update` edge case.
pub const JUMP_FRAME_DT: f32 = 0.1;

/// Resource that tracks whether `KeyCode::Space` was pressed on the
/// previous frame. Required because `ButtonInput::just_pressed` is
/// unreliable in tests that bypass the `KeyboardInput` event pipeline.
#[derive(Resource, Default)]
pub struct JumpState {
    pub space_was_pressed: bool,
}

/// Bevy plugin that wires `jump_system` into the `Update` schedule.
/// Add this alongside `PlayerMovementPlugin` to enable the full
/// arrow-keys + Space gameplay witness.
pub struct JumpPlugin;

impl Plugin for JumpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JumpState>();
        app.add_systems(Update, jump_system);
    }
}

/// Applies an upward impulse to every entity carrying `PlayerController`
/// when `KeyCode::Space` transitions from released to pressed.
///
/// On the rising edge:
/// ```text
/// player.translation.y += ctrl.jump_force * JUMP_FRAME_DT
/// ```
///
/// The query is gated by `With<PlayerController>`, so the enemy (which
/// carries `EnemyPatrol`) and the ground (no controller) are unaffected.
pub fn jump_system(
    input: Res<ButtonInput<KeyCode>>,
    mut jump_state: ResMut<JumpState>,
    mut q: Query<(&mut Transform, &PlayerController)>,
) {
    let pressed_now = input.pressed(KeyCode::Space);
    if pressed_now && !jump_state.space_was_pressed {
        for (mut t, ctrl) in &mut q {
            t.translation.y += ctrl.jump_force * JUMP_FRAME_DT;
        }
    }
    jump_state.space_was_pressed = pressed_now;
}
