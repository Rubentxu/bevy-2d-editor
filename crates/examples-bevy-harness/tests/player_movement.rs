//! Integration test: Bevy play_mode runtime behaviour for the v1.0
//! canonical sample (BJ-4 deliverable).
//!
//! The test is marked `#[ignore]` so the default `cargo test` run does
//! not pay the Bevy compile cost. To run explicitly:
//!
//! ```text
//! cargo test -p examples-bevy-harness --test player_movement -- --ignored
//! ```
//!
//! What this test proves:
//!
//! 1. The `Player` entity spawned from `examples/platformer-minimal/`
//!    carries the `PlayerController` component (already proven by
//!    `load_sample.rs`; re-asserted here).
//! 2. When `KeyCode::ArrowRight` is pressed, the player translates
//!    along the x axis at the speed declared by the schema (200.0).
//! 3. When no key is pressed, the player stays still.
//! 4. When `KeyCode::ArrowLeft` is pressed, the player translates
//!    negatively.
//! 5. The `Enemy` entity (which carries `EnemyPatrol`, not
//!    `PlayerController`) is unaffected by player input.
//!
//! All assertions check `Transform.translation.x` after one frame
//! of virtual time advance.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy::input::InputPlugin;
use bevy::time::TimeUpdateStrategy;
use editor_model::scene_asset::SceneAssetDocument;
use examples_bevy_harness::{
    PlayerController, PlayerMovementPlugin, spawn_all_assets,
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

/// Load the 4 sample assets into a `BTreeMap<String, SceneAssetDocument>`.
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

/// Build a Bevy `App` with `MinimalPlugins` + `InputPlugin` +
/// `PlayerMovementPlugin` and spawn the 4 sample assets. Returns the app.
fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(InputPlugin);
    app.add_plugins(PlayerMovementPlugin);
    let assets = load_sample_assets();
    spawn_all_assets(app.world_mut(), &assets);
    app
}

/// Find the entity whose `Name` matches the given string.
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

const PLAYER_SPEED: f32 = 200.0;
const FRAME_DT: f32 = 0.1;

/// Press a key (set the input to "pressed" without releasing others).
fn press(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
}

/// Release a key (only used to clear state between sub-tests).
fn release(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
}

/// Advance virtual time by `secs` and run one update cycle.
///
/// Bevy 0.19's `time_system` runs every frame and overwrites
/// `Time<Virtual>` with the real-time delta. To control `delta_secs`
/// deterministically we switch to `ManualDuration` for the frame,
/// run one update. The strategy is left as `ManualDuration` after
/// the call (subsequent frames advance by the same delta until the
/// caller sets a different one).
///
/// The first call also "warms up" `Time<Real>` (its `last_update` is
/// `None` by default, so the very first `time_system` run sets it
/// without applying a delta). The second call applies the configured
/// duration. To make the API predictable, we always run two updates
/// and discard the first one's state.
fn advance(app: &mut App, secs: f32) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(secs));
    // Warm-up: first run establishes `last_update` for `Time<Real>`.
    app.update();
    // Effective run: applies the configured duration as `delta_secs`.
    app.update();
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test player_movement -- --ignored"]
fn player_translates_right_on_arrow_right() {
    let mut app = setup_app();

    let player = find_named(&mut app, "Player");
    // baseline: Player must carry PlayerController with the schema-declared speed
    let ctrl = app
        .world()
        .get::<PlayerController>(player)
        .expect("Player should carry PlayerController")
        .clone();
    assert_eq!(ctrl.speed, PLAYER_SPEED);
    assert_eq!(ctrl.jump_force, 400.0);

    // baseline x position is 0
    let x0 = app.world().get::<Transform>(player).unwrap().translation.x;
    assert_eq!(x0, 0.0, "player should start at x=0");

    // press ArrowRight, advance one frame
    press(&mut app, KeyCode::ArrowRight);
    advance(&mut app, FRAME_DT);

    let x1 = app.world().get::<Transform>(player).unwrap().translation.x;
    let expected = PLAYER_SPEED * FRAME_DT; // 20.0
    assert!(
        (x1 - expected).abs() < 0.5,
        "expected x ≈ {}, got {} (delta {})",
        expected,
        x1,
        x1 - expected
    );

    release(&mut app, KeyCode::ArrowRight);
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test player_movement -- --ignored"]
fn player_stays_still_with_no_input() {
    let mut app = setup_app();
    let player = find_named(&mut app, "Player");
    advance(&mut app, FRAME_DT);

    let t = app.world().get::<Transform>(player).unwrap();
    assert_eq!(
        t.translation.x, 0.0,
        "player should not translate with no input"
    );
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test player_movement -- --ignored"]
fn player_translates_left_on_arrow_left() {
    let mut app = setup_app();
    let player = find_named(&mut app, "Player");

    press(&mut app, KeyCode::ArrowLeft);
    advance(&mut app, FRAME_DT);

    let x1 = app.world().get::<Transform>(player).unwrap().translation.x;
    let expected = -PLAYER_SPEED * FRAME_DT; // -20.0
    assert!(
        (x1 - expected).abs() < 0.5,
        "expected x ≈ {}, got {} (delta {})",
        expected,
        x1,
        x1 - expected
    );

    release(&mut app, KeyCode::ArrowLeft);
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test player_movement -- --ignored"]
fn enemy_is_unaffected_by_player_input() {
    let mut app = setup_app();
    let enemy = find_named(&mut app, "Enemy");

    press(&mut app, KeyCode::ArrowRight);
    advance(&mut app, FRAME_DT);

    let t = app.world().get::<Transform>(enemy).unwrap();
    assert_eq!(
        t.translation.x, 0.0,
        "enemy should not be affected by player input (filter is With<PlayerController>)"
    );

    release(&mut app, KeyCode::ArrowRight);
}
