//! Integration test: Bevy play_mode runtime behaviour for the v1.0
//! canonical sample — jump mechanic (BJ-5 partial).
//!
//! The test is marked `#[ignore]` so the default `cargo test` run does
//! not pay the Bevy compile cost. To run explicitly:
//!
//! ```text
//! cargo test -p examples-bevy-harness --test jump -- --ignored
//! ```
//!
//! What this test proves:
//!
//! 1. When `KeyCode::Space` is pressed, the player translates along
//!    the y axis by `PlayerController.jump_force * JUMP_FRAME_DT`
//!    (default 40.0 units).
//! 2. When no key is pressed, the player stays at y=0.
//! 3. The `Enemy` and `Ground` entities (which do not carry
//!    `PlayerController`) are unaffected by the Space press.
//!
//! ## Edge detection caveat
//!
//! Bevy 0.19's `keyboard_input_system` runs in `PreUpdate` and calls
//! `ButtonInput::clear()`, which zeroes `just_pressed` *before* our
//! `Update` system runs. The harness works around this by tracking
//! the previous frame's pressed-state in `JumpState` (rising-edge
//! detection via `pressed()` + memory). See `src/jump.rs` for the
//! rationale.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy::input::InputPlugin;
use bevy::time::TimeUpdateStrategy;
use editor_model::scene_asset::SceneAssetDocument;
use examples_bevy_harness::{
    PlayerController, JumpPlugin, spawn_all_assets,
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
    app.add_plugins(JumpPlugin);
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

const PLAYER_JUMP_FORCE: f32 = 400.0;
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
/// by `secs` on the next update. After this call the app has consumed
/// one frame of `secs` seconds of virtual time.
fn advance(app: &mut App, secs: f32) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(secs));
    app.update(); // warm-up
    app.update(); // effective
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test jump -- --ignored"]
fn player_jumps_on_space_press() {
    let mut app = setup_app();
    let player = find_named(&mut app, "Player");

    // baseline
    let ctrl = app
        .world()
        .get::<PlayerController>(player)
        .expect("Player should carry PlayerController");
    assert_eq!(ctrl.jump_force, PLAYER_JUMP_FORCE);

    let y0 = app.world().get::<Transform>(player).unwrap().translation.y;
    assert_eq!(y0, 0.0, "player should start at y=0");

    // press Space; update once; jump fires on the rising edge
    press(&mut app, KeyCode::Space);
    advance(&mut app, FRAME_DT);

    let y1 = app.world().get::<Transform>(player).unwrap().translation.y;
    let expected_delta = PLAYER_JUMP_FORCE * FRAME_DT; // 40.0
    assert!(
        (y1 - (y0 + expected_delta)).abs() < 0.5,
        "expected y ≈ {}, got {} (delta {})",
        y0 + expected_delta,
        y1,
        y1 - y0
    );

    release(&mut app, KeyCode::Space);
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test jump -- --ignored"]
fn player_does_not_jump_without_input() {
    let mut app = setup_app();
    let player = find_named(&mut app, "Player");

    advance(&mut app, FRAME_DT);

    let t = app.world().get::<Transform>(player).unwrap();
    assert_eq!(
        t.translation.y, 0.0,
        "player should not translate with no input"
    );
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness --test jump -- --ignored"]
fn jump_only_affects_player() {
    let mut app = setup_app();
    let enemy = find_named(&mut app, "Enemy");

    let enemy_y0 = app.world().get::<Transform>(enemy).unwrap().translation.y;
    assert_eq!(enemy_y0, 0.0, "enemy should start at y=0");

    press(&mut app, KeyCode::Space);
    advance(&mut app, FRAME_DT);

    let enemy_t = app.world().get::<Transform>(enemy).unwrap();
    assert_eq!(
        enemy_t.translation.y, enemy_y0,
        "enemy should not be affected by jump (filter is With<PlayerController>)"
    );

    release(&mut app, KeyCode::Space);
}
