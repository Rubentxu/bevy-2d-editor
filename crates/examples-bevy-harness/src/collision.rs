//! Pickup collision plugin for the v1.0 canonical sample harness.
//!
//! Despawns any entity carrying the `Pickup` marker when the player
//! overlaps it within a fixed hitbox. Demonstrates a Bevy play_mode
//! gameplay mechanic with deterministic test invariants.
//!
//! This is the BJ-5 (partial) deliverable: pickup collision only.
//! Enemy patrol, jump, and contact-death are tracked as separate
//! debt items for future cycles.

use bevy::prelude::*;

use crate::components::{Pickup, PlayerController};

/// Bevy plugin that wires `pickup_collision_system` into the `Update`
/// schedule. Add this to any Bevy `App` whose world has been populated
/// by `spawn_all_assets` to enable pickup collection.
pub struct PickupCollisionPlugin;

impl Plugin for PickupCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pickup_collision_system);
    }
}

/// Hitbox half-extent (px). The sample's sprites are 32×32, so 16 px
/// covers the central half. Generous for testing, narrow enough for
/// precision in normal play.
pub const PICKUP_HITBOX_HALF: f32 = 16.0;

/// Despawns every `Pickup` entity within `PICKUP_HITBOX_HALF` of the
/// player in both axes.
///
/// The system tolerates zero or multiple players (uses `single()`,
/// which returns Err if the query doesn't match exactly one). The
/// sample declares exactly one Player entity, so this is the natural
/// shape. If the count diverges, the system is a no-op.
pub fn pickup_collision_system(
    mut commands: Commands,
    q_player: Query<&Transform, With<PlayerController>>,
    q_pickups: Query<(Entity, &Transform), With<Pickup>>,
) {
    let Ok(player_t) = q_player.single() else {
        return;
    };
    let px = player_t.translation.x;
    let py = player_t.translation.y;
    for (pickup_e, pickup_t) in &q_pickups {
        let dx = (px - pickup_t.translation.x).abs();
        let dy = (py - pickup_t.translation.y).abs();
        if dx < PICKUP_HITBOX_HALF && dy < PICKUP_HITBOX_HALF {
            commands.entity(pickup_e).despawn();
        }
    }
}
