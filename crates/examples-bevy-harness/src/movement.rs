//! Player movement plugin for the v1.0 canonical sample harness.
//!
//! This module demonstrates Bevy play_mode runtime behaviour for the
//! `examples/platformer-minimal/` sample: when the user presses the
//! arrow keys, the `Player` entity translates along the x axis at the
//! speed declared in its `PlayerController` component (which round-trips
//! from the editor-authored `game.PlayerController` schema).
//!
//! This is the BJ-4 deliverable. The harness remains a **witness** —
//! we prove the JSON round-trips and the runtime behaves as expected;
//! we do not implement full gameplay (jump, enemy patrol, pickup
//! collision). Those are tracked as BJ-5 follow-ups.

use bevy::prelude::*;

use crate::components::PlayerController;

/// Bevy plugin that wires `player_movement_system` into the `Update`
/// schedule. Add this to any Bevy `App` whose world has been populated
/// by `spawn_all_assets` to enable player movement.
pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_movement_system);
    }
}

/// Moves every entity carrying `PlayerController` along the x axis at
/// the speed declared by the component. Reads `ButtonInput<KeyCode>`
/// for `ArrowLeft` and `ArrowRight`; if neither is pressed, the system
/// is a no-op.
pub fn player_movement_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut Transform, &PlayerController)>,
) {
    let mut dir = 0.0;
    if input.pressed(KeyCode::ArrowRight) {
        dir += 1.0;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        dir -= 1.0;
    }
    if dir == 0.0 {
        return;
    }
    let dt = time.delta_secs();
    for (mut t, ctrl) in &mut q {
        t.translation.x += dir * ctrl.speed * dt;
    }
}
