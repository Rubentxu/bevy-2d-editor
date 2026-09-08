//! Bevy 2D runtime harness that consumes editor-authored scene asset
//! JSON as a witness for the v1.0 canonical sample game (G1).
//!
//! See `docs/v1.0-stabilization-evidence-map.md` §6.1 / §7-P1 and
//! `docs/sddk/v1-g1-bevy-harness/{explore-report,spec}.md` for the
//! design rationale and scope.

pub mod collision;
pub mod components;
pub mod loader;
pub mod movement;

pub use collision::{PICKUP_HITBOX_HALF, PickupCollisionPlugin, pickup_collision_system};
pub use components::{EditorSpriteAsset, EnemyPatrol, Pickup, PlayerController, Visible};
pub use loader::SpawnReport;
pub use movement::{PlayerMovementPlugin, player_movement_system};

use bevy::prelude::*;
use editor_model::scene_asset::SceneAssetDocument;
use std::collections::BTreeMap;

/// Spawn every asset in `assets_by_path` into `world`. Returns the
/// total entity count across all spawned assets.
///
/// `assets_by_path` is keyed by `SceneAssetDocument.logical_path`,
/// matching the `asset_ref` values used in `SceneInstance.asset_ref`.
pub fn spawn_all_assets(world: &mut World, assets_by_path: &BTreeMap<String, SceneAssetDocument>) -> usize {
    let mut total = 0;
    for doc in assets_by_path.values() {
        let report = loader::spawn_scene_asset(world, doc);
        total += report.entity_count;
    }
    total
}
