//! Integration test: the v1.0 canonical sample game
//! (`examples/platformer-minimal/`) round-trips through the harness
//! into a Bevy `World`.
//!
//! The test is marked `#[ignore]` so the default `cargo test` run
//! does not pay the Bevy compile cost. To run explicitly:
//!
//! ```text
//! cargo test -p examples-bevy-harness -- --ignored
//! ```
//!
//! The test runs all of the following:
//! 1. Load `examples/platformer-minimal/project.json` and assert it
//!    declares 4 scene assets.
//! 2. Load each `scene-assets/**/*.actor.json` into a
//!    `SceneAssetDocument` and assert each has exactly 1 entity.
//! 3. Build a Bevy `World` and spawn the 4 assets via
//!    `spawn_all_assets`. Assert the world contains 4 entities with
//!    `Name` matching the asset's root entity name.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use bevy::prelude::*;
use editor_model::scene_asset::SceneAssetDocument;
use examples_bevy_harness::spawn_all_assets;

/// Resolves `<repo>/examples/platformer-minimal/` regardless of where
/// cargo invokes the test from.
fn sample_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join("platformer-minimal")
}

#[test]
#[ignore = "Run explicitly with: cargo test -p examples-bevy-harness -- --ignored"]
fn sample_round_trips_into_bevy_world() {
    let dir = sample_dir();
    assert!(
        dir.is_dir(),
        "Sample directory must exist: {}",
        dir.display()
    );

    // 1. Load project.json — minimal validation; the full project
    // schema is verified by the editor's own integration tests.
    let project_path = dir.join("project.json");
    let project_raw = fs::read_to_string(&project_path)
        .unwrap_or_else(|e| panic!("read {}: {}", project_path.display(), e));
    let project: serde_json::Value = serde_json::from_str(&project_raw)
        .unwrap_or_else(|e| panic!("parse project.json: {}", e));
    let declared_assets = project
        .get("scene_assets")
        .and_then(serde_json::Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(
        declared_assets, 4,
        "project.json should declare 4 scene assets, found {}",
        declared_assets
    );

    // 2. Load each scene asset and verify entity count.
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
        assert_eq!(
            doc.entities.len(),
            1,
            "{} should declare 1 root entity",
            rel
        );
        assert_eq!(
            doc.logical_path, *logical_path,
            "{} logical_path mismatch",
            rel
        );
        assets_by_path.insert(logical_path.to_string(), doc);
    }

    // 3. Build a Bevy World and spawn all assets.
    let mut world = World::new();
    let spawned = spawn_all_assets(&mut world, &assets_by_path);
    assert_eq!(spawned, 4, "harness should spawn 4 entities");

    // 4. Verify the world holds at least 4 entities carrying a
    // `Name` component from the `editor.Name` of each scene asset.
    // (`World::new()` may spawn internal entities for resource
    // bookkeeping; the harness only cares about its 4 spawned assets.)
    let all_entities: Vec<Entity> = world.iter_entities().map(|e| e.id()).collect();
    let entities: Vec<Entity> = all_entities
        .iter()
        .copied()
        .filter(|e| world.get::<Name>(*e).is_some())
        .collect();
    assert_eq!(
        entities.len(),
        4,
        "world should hold 4 entities with Name from spawned assets"
    );

    let names: BTreeSet<String> = entities
        .iter()
        .filter_map(|e| world.get::<Name>(*e).map(|n| n.as_str().to_string()))
        .collect();
    let expected: BTreeSet<String> = ["Enemy", "Ground", "Pickup", "Player"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        names, expected,
        "expected the 4 sample asset root entity names"
    );

    // 5. Spot-check that the player carries the custom Bevy
    // component for `game.PlayerController`.
    let player_entity = entities
        .iter()
        .find(|e| {
            world
                .get::<Name>(**e)
                .map(|n| n.as_str() == "Player")
                .unwrap_or(false)
        })
        .copied()
        .expect("Player entity should exist");
    let player = world
        .get::<examples_bevy_harness::PlayerController>(player_entity)
        .expect("Player should carry the harness PlayerController component");
    assert_eq!(player.speed, 200.0);
    assert_eq!(player.jump_force, 400.0);
}
