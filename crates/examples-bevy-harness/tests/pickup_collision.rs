//! Integration test: pickup collision Bevy play_mode behaviour
//! (BJ-5 partial deliverable).
//!
//! The test is marked `#[ignore]` so the default `cargo test` run does
//! not pay the Bevy compile cost. To run explicitly:
//!
//! ```text
//! cargo test -p examples-bevy-harness --test pickup_collision -- --ignored
//! ```
//!
//! What this test proves:
//!
//! 1. When the player overlaps a pickup, the pickup entity is despawned
//!    by `pickup_collision_system`.
//! 2. When the player is far from a pickup, the pickup entity remains.
//! 3. The collision system only affects entities carrying the `Pickup`
//!    marker — Player, Enemy, and Ground entities are untouched.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy::input::InputPlugin;
use bevy::time::TimeUpdateStrategy;
use editor_model::scene_asset::SceneAssetDocument;
use examples_bevy_harness::{
    Pickup, PickupCollisionPlugin, PlayerController, PlayerMovementPlugin,
    spawn_all_assets,
};
use std::time::Duration;

fn sample_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join("platformer-minimal")
}

fn load_sample_assets() -> BTreeMap<String, SceneAssetDocument> {
    let dir = sample_dir();
    let asset_paths = [
        ("characters/player", "scene-assets/characters/player.actor.json"),
        ("characters/enemy", "scene-assets/characters/enemy.actor.json"),
        ("environment/ground", "scene-assets/environment/ground.fragment.json"),
        ("effects/pickup", "scene-assets/effects/pickup.actor.json"),
    ];
    let mut assets_by_path: BTreeMap<String, SceneAssetDocument> = BTreeMap::new();
    for (logical_path, rel) in asset_paths.iter() {
        let abs = dir.join(rel);
        let raw = fs::read_to_string(&abs)
            .unwrap_or_else(|e| panic!("read {}: {}", abs.display(), e));
        let doc: SceneAssetDocument = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("parse {}: {}", abs.display(), e));
        assets_by_path.insert(logical_path.to_string(), doc);
    }
    assets_by_path
}

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(InputPlugin);
    app.add_plugins(PlayerMovementPlugin);
    app.add_plugins(PickupCollisionPlugin);
    let assets = load_sample_assets();
    spawn_all_assets(app.world_mut(), &assets);
    app
}

fn find_named(app: &mut App, name: &str) -> Entity {
    let world = app.world_mut();
    let mut found = None;
    let mut q = world.query::<(Entity, &Name)>();
    for (e, n) in q.iter(world) {
        if n.as_str() == name {
            found = Some(e);
            break;
        }
    }
    found.unwrap_or_else(|| panic!("entity with Name={} not found", name))
}

fn find_pickup(app: &mut App) -> Entity {
    let world = app.world_mut();
    let mut q = world.query_filtered::<Entity, With<Pickup>>();
    let mut found = None;
    for e in q.iter(world) {
        found = Some(e);
        break;
    }
    found.expect("Pickup entity should exist after spawn_all_assets")
}

/// Bevy 0.19 test-time helper: `Time<Real>::last_update` is None on
/// first frame; `time_system` sets it without applying a delta. We run
/// two updates per advance so `delta_secs` is meaningful.
fn advance(app: &mut App, secs: f32) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(secs));
    app.update();
    app.update();
}

fn entity_exists(world: &World, entity: Entity) -> bool {
    world.get_entity(entity).is_ok()
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test pickup_collision -- --ignored"]
fn pickup_despawns_when_player_overlaps() {
    let mut app = setup_app();
    let player = find_named(&mut app, "Player");
    let pickup = find_pickup(&mut app);

    // Move player and pickup close together (well within hitbox).
    {
        let world = app.world_mut();
        let mut p_t = world.get_mut::<Transform>(player).unwrap();
        p_t.translation = Vec3::new(0.0, 0.0, 0.0);
        let mut pu_t = world.get_mut::<Transform>(pickup).unwrap();
        pu_t.translation = Vec3::new(5.0, 5.0, 0.0);
    }

    assert!(entity_exists(app.world(), pickup), "pickup should exist before");

    advance(&mut app, 0.1);

    assert!(
        !entity_exists(app.world(), pickup),
        "pickup should be despawned after player overlaps it"
    );
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test pickup_collision -- --ignored"]
fn pickup_unchanged_when_player_far() {
    let mut app = setup_app();
    let player = find_named(&mut app, "Player");
    let pickup = find_pickup(&mut app);

    // Place player and pickup far apart.
    {
        let world = app.world_mut();
        let mut p_t = world.get_mut::<Transform>(player).unwrap();
        p_t.translation = Vec3::new(0.0, 0.0, 0.0);
        let mut pu_t = world.get_mut::<Transform>(pickup).unwrap();
        pu_t.translation = Vec3::new(1000.0, 1000.0, 0.0);
    }

    advance(&mut app, 0.1);

    assert!(
        entity_exists(app.world(), pickup),
        "pickup should remain when player is far away"
    );
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test pickup_collision -- --ignored"]
fn pickup_collision_only_affects_pickup_marker() {
    let mut app = setup_app();
    let player = find_named(&mut app, "Player");
    let enemy = find_named(&mut app, "Enemy");
    let ground = find_named(&mut app, "Ground");
    let pickup = find_pickup(&mut app);

    // Move player close to pickup, then verify enemy and ground remain.
    {
        let world = app.world_mut();
        let mut p_t = world.get_mut::<Transform>(player).unwrap();
        p_t.translation = Vec3::new(0.0, 0.0, 0.0);
        let mut pu_t = world.get_mut::<Transform>(pickup).unwrap();
        pu_t.translation = Vec3::new(5.0, 5.0, 0.0);
    }

    advance(&mut app, 0.1);

    assert!(!entity_exists(app.world(), pickup), "pickup should despawn");
    assert!(entity_exists(app.world(), player), "player should remain");
    assert!(entity_exists(app.world(), enemy), "enemy should remain (no Pickup marker)");
    assert!(entity_exists(app.world(), ground), "ground should remain (no Pickup marker)");
}
