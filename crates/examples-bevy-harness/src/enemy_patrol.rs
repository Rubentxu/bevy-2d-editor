//! Enemy patrol plugin for the v1.0 canonical sample harness.
//!
//! This module demonstrates Bevy play_mode runtime behaviour for the
//! `examples/platformer-minimal/` sample: the `Enemy` entity
//! oscillates between `-patrol_range` and `+patrol_range` along the
//! x axis at the speed declared by its `EnemyPatrol` component (which
//! round-trips from the editor-authored `game.EnemyPatrol` schema).
//!
//! This is a **witness** — we prove the JSON round-trips and the
//! runtime behaves as expected; we do not implement full gameplay
//! (patrol state machine, vertical patrol, contact death).
//!
//! ## Boundary detection
//!
//! The patrol is **reflective**: when the entity reaches
//! `±patrol_range`, direction flips. The position does not overshoot
//! the boundary because:
//!
//! 1. On the frame the entity reaches `patrol_range`, direction
//!    flips to `-1.0` BEFORE the move is applied.
//! 2. The subsequent `translation.x += -1.0 * speed * dt` adds a
//!    negative delta, sending the entity back.
//!
//! This guarantees `|translation.x| <= patrol_range + speed * dt` at
//! all times.
//!
//! ## Per-entity state
//!
//! Each enemy carries an `EnemyDirection` component (initialised to
//! `+1.0` by the loader). A `Local<>` resource would force a single
//! global direction shared across all enemies — we want independent
//! state per enemy, so a component is the right shape.
//!
//! See `crates/examples-bevy-harness/tests/enemy_patrol.rs` for the
//! integration tests that prove this behaviour.

use bevy::prelude::*;

use crate::components::{EnemyDirection, EnemyPatrol};

/// Bevy plugin that wires `enemy_patrol_system` into the `Update`
/// schedule. Add this alongside `PlayerMovementPlugin` to enable
/// the full arrow-keys + enemy-patrol gameplay witness.
pub struct EnemyPatrolPlugin;

impl Plugin for EnemyPatrolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, enemy_patrol_system);
    }
}

/// Moves every entity carrying `EnemyPatrol` along the x axis at
/// the speed declared by the component. Reflects at the
/// `±patrol_range` boundaries.
///
/// The query is gated by `With<EnemyPatrol>`, so the player (which
/// carries `PlayerController`, not `EnemyPatrol`) is unaffected.
pub fn enemy_patrol_system(
    time: Res<Time>,
    mut q: Query<(&mut Transform, &EnemyPatrol, &mut EnemyDirection)>,
) {
    let dt = time.delta_secs();
    for (mut t, ctrl, mut dir) in &mut q {
        if t.translation.x >= ctrl.patrol_range {
            dir.0 = -1.0;
        } else if t.translation.x <= -ctrl.patrol_range {
            dir.0 = 1.0;
        }
        t.translation.x += dir.0 * ctrl.speed * dt;
    }
}
