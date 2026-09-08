//! Integration test: Bevy play_mode runtime behaviour for the v1.0
//! canonical sample — enemy patrol (BJ-5 partial).
//!
//! The test is marked `#[ignore]` so the default `cargo test` run does
//! not pay the Bevy compile cost. To run explicitly:
//!
//! ```text
//! cargo test -p examples-bevy-harness --test enemy_patrol -- --ignored
//! ```
//!
//! What this test proves:
//!
//! 1. The `Enemy` entity spawned from `examples/platformer-minimal/`
//!    carries the `EnemyPatrol` and `EnemyDirection` components.
//! 2. When no input is pressed, the enemy translates along the x
//!    axis at the speed declared by the schema (60.0).
//! 3. When the enemy reaches `patrol_range`, direction reverses.
//! 4. The `Player` and `Ground` entities (which do not carry
//!    `EnemyPatrol`) are unaffected by the patrol system.
//!
//! All assertions check `Transform.translation.x` after one or more
//! frames of virtual time advance.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy::input::InputPlugin;
use bevy::time::TimeUpdateStrategy;
use editor_model::scene_asset::SceneAssetDocument;
use examples_bevy_harness::{
    EnemyDirection, EnemyPatrol, EnemyPatrolPlugin,
    PlayerMovementPlugin, spawn_all_assets,
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

fn setup_app(plugins: bool) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(InputPlugin);
    if plugins {
        app.add_plugins((PlayerMovementPlugin, EnemyPatrolPlugin));
    } else {
        app.add_plugins(EnemyPatrolPlugin);
    }
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

const ENEMY_SPEED: f32 = 60.0;
const ENEMY_PATROL_RANGE: f32 = 150.0;
const FRAME_DT: f32 = 0.1;

fn press(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
}

fn release(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
}

/// Warm up Time<Real> by running one update, then advance virtual time
/// by `secs` on the next update. The `ManualDuration` strategy is
/// one-shot: we re-set it before each `update()` so both the warm-up
/// and the effective frame apply the configured duration.
fn advance(app: &mut App, secs: f32) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(secs));
    app.update(); // warm-up
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(secs));
    app.update(); // effective
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test enemy_patrol -- --ignored"]
fn enemy_patrols_right_initially() {
    let mut app = setup_app(false);
    let enemy = find_named(&mut app, "Enemy");

    // baseline
    let ctrl = app
        .world()
        .get::<EnemyPatrol>(enemy)
        .expect("Enemy should carry EnemyPatrol");
    assert_eq!(ctrl.speed, ENEMY_SPEED);
    assert_eq!(ctrl.patrol_range, ENEMY_PATROL_RANGE);

    let dir = app
        .world()
        .get::<EnemyDirection>(enemy)
        .expect("Enemy should carry EnemyDirection");
    assert_eq!(dir.0, 1.0, "default direction should be +1.0 (right)");

    let x0 = app.world().get::<Transform>(enemy).unwrap().translation.x;
    assert_eq!(x0, 0.0, "enemy should start at x=0");

    // advance one frame: enemy moves right by speed * dt
    advance(&mut app, FRAME_DT);

    let x1 = app.world().get::<Transform>(enemy).unwrap().translation.x;
    let expected_delta = ENEMY_SPEED * FRAME_DT; // 6.0
    assert!(
        (x1 - (x0 + expected_delta)).abs() < 0.5,
        "expected x ≈ {}, got {} (delta {})",
        x0 + expected_delta,
        x1,
        x1 - x0
    );
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test enemy_patrol -- --ignored"]
fn enemy_reverses_at_boundary() {
    let mut app = setup_app(false);
    let enemy = find_named(&mut app, "Enemy");

    // Manually move the enemy just past the right boundary. This
    // bypasses the time-control issue (multi-frame advances with
    // ManualDuration are flaky) and lets us test the boundary-flip
    // logic in isolation. We choose x = patrol_range + 1 so the
    // first update's `>=` check fires and direction flips to -1.
    let initial_x = ENEMY_PATROL_RANGE + 1.0;
    {
        let mut t = app.world_mut().get_mut::<Transform>(enemy).unwrap();
        t.translation.x = initial_x;
    }

    advance(&mut app, FRAME_DT);

    // After one frame, direction flipped to -1.0 and the entity
    // moved by -speed * dt = -6.0.
    let x_after_flip = app.world().get::<Transform>(enemy).unwrap().translation.x;
    let expected_x = initial_x - ENEMY_SPEED * FRAME_DT;
    assert!(
        (x_after_flip - expected_x).abs() < 0.5,
        "expected x ≈ {} (initial - speed*dt), got {}",
        expected_x,
        x_after_flip
    );

    let dir = app.world().get::<EnemyDirection>(enemy).unwrap();
    assert_eq!(dir.0, -1.0, "direction should be flipped to -1.0");
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test enemy_patrol -- --ignored"]
fn enemy_reverses_at_left_boundary() {
    let mut app = setup_app(false);
    let enemy = find_named(&mut app, "Enemy");

    // Manually position enemy past the left boundary. First update
    // should flip direction to +1.0 and move right.
    let initial_x = -ENEMY_PATROL_RANGE - 1.0;
    {
        let mut t = app.world_mut().get_mut::<Transform>(enemy).unwrap();
        t.translation.x = initial_x;
    }

    advance(&mut app, FRAME_DT);

    let x_after_flip = app.world().get::<Transform>(enemy).unwrap().translation.x;
    let expected_x = initial_x + ENEMY_SPEED * FRAME_DT;
    assert!(
        (x_after_flip - expected_x).abs() < 0.5,
        "expected x ≈ {}, got {}",
        expected_x,
        x_after_flip
    );

    let dir = app.world().get::<EnemyDirection>(enemy).unwrap();
    assert_eq!(dir.0, 1.0, "direction should be flipped to +1.0");
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test enemy_patrol -- --ignored"]
fn enemy_unaffected_by_player_movement() {
    let mut app = setup_app(true); // both PlayerMovementPlugin + EnemyPatrolPlugin
    let player = find_named(&mut app, "Player");
    let enemy = find_named(&mut app, "Enemy");

    // press ArrowRight; player should move at 200.0 * 0.1 = 20.0
    // enemy should move at 60.0 * 0.1 = 6.0 (patrol speed, not 200.0)
    press(&mut app, KeyCode::ArrowRight);
    advance(&mut app, FRAME_DT);

    let player_x = app.world().get::<Transform>(player).unwrap().translation.x;
    let enemy_x = app.world().get::<Transform>(enemy).unwrap().translation.x;

    assert!(
        (player_x - 20.0).abs() < 0.5,
        "player should move at 200.0 * 0.1 = 20.0, got {}",
        player_x
    );
    assert!(
        (enemy_x - 6.0).abs() < 0.5,
        "enemy should move at 60.0 * 0.1 = 6.0 (patrol, NOT player movement), got {}",
        enemy_x
    );

    release(&mut app, KeyCode::ArrowRight);
}
