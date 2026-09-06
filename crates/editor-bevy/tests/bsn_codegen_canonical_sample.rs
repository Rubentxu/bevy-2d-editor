//! Integration test: round-trip the v1.0 canonical sample (`examples/platformer-minimal/`)
//! through `emit_bsn_source_from_document`. Each scene asset JSON committed under
//! `examples/platformer-minimal/scene-assets/` is loaded via `include_str!`,
//! deserialized to a `SceneAssetDocument`, and the resulting BSN is compared
//! against the committed `.bsn` reference under `examples/platformer-minimal/export/`.
//!
//! If this test fails, regenerate the references:
//!   1. Update the scene asset JSON.
//!   2. Run `cargo test -p editor-bevy --test bsn_codegen_canonical_sample -- --ignored`
//!      (the `regenerate` test writes the new output).
//!   3. Inspect the diff, commit if correct.

use editor_bevy::{
    bsn_codegen::emit_bsn_source_from_document,
    scene_asset::SceneAssetDocument,
};

const PLAYER_JSON: &str = include_str!("../../../examples/platformer-minimal/scene-assets/characters/player.actor.json");
const ENEMY_JSON: &str = include_str!("../../../examples/platformer-minimal/scene-assets/characters/enemy.actor.json");
const GROUND_JSON: &str = include_str!("../../../examples/platformer-minimal/scene-assets/environment/ground.fragment.json");
const PICKUP_JSON: &str = include_str!("../../../examples/platformer-minimal/scene-assets/effects/pickup.actor.json");

const PLAYER_BSN_REF: &str = include_str!("../../../examples/platformer-minimal/export/player.bsn");
const ENEMY_BSN_REF: &str = include_str!("../../../examples/platformer-minimal/export/enemy.bsn");
const GROUND_BSN_REF: &str = include_str!("../../../examples/platformer-minimal/export/ground.bsn");
const PICKUP_BSN_REF: &str = include_str!("../../../examples/platformer-minimal/export/pickup.bsn");

fn assert_bsn_round_trip(label: &str, json: &str, reference_bsn: &str, scene_name: &str) {
    let doc: SceneAssetDocument =
        serde_json::from_str(json).unwrap_or_else(|e| panic!("{label}: failed to parse JSON — {e}"));

    let result = emit_bsn_source_from_document(&doc, scene_name);

    assert!(
        result.warnings.is_empty(),
        "{label}: expected no export warnings, got: {:?}",
        result.warnings
    );

    let actual = result.source.trim_end();
    let expected = reference_bsn.trim_end();

    assert_eq!(
        actual, expected,
        "{label}: BSN output drifted from committed reference.\n\
         If this is intentional, run the `regenerate_*` tests to update the .bsn files.\n\
         --- actual ---\n{actual}\n--- expected ---\n{expected}\n"
    );
}

#[test]
fn bsn_round_trip_player() {
    assert_bsn_round_trip(
        "player",
        PLAYER_JSON,
        PLAYER_BSN_REF,
        "platformer_minimal_player",
    );
}

#[test]
fn bsn_round_trip_enemy() {
    assert_bsn_round_trip(
        "enemy",
        ENEMY_JSON,
        ENEMY_BSN_REF,
        "platformer_minimal_enemy",
    );
}

#[test]
fn bsn_round_trip_ground() {
    assert_bsn_round_trip(
        "ground",
        GROUND_JSON,
        GROUND_BSN_REF,
        "platformer_minimal_ground",
    );
}

#[test]
fn bsn_round_trip_pickup() {
    assert_bsn_round_trip(
        "pickup",
        PICKUP_JSON,
        PICKUP_BSN_REF,
        "platformer_minimal_pickup",
    );
}

// ─── Regenerators (ignored by default; run with --ignored to overwrite references) ─

#[test]
#[ignore = "regenerator — run with --ignored to overwrite the committed .bsn references"]
fn regenerate_player_bsn() {
    regenerate("player", PLAYER_JSON, "platformer_minimal_player");
}

#[test]
#[ignore = "regenerator — run with --ignored to overwrite the committed .bsn references"]
fn regenerate_enemy_bsn() {
    regenerate("enemy", ENEMY_JSON, "platformer_minimal_enemy");
}

#[test]
#[ignore = "regenerator — run with --ignored to overwrite the committed .bsn references"]
fn regenerate_ground_bsn() {
    regenerate("ground", GROUND_JSON, "platformer_minimal_ground");
}

#[test]
#[ignore = "regenerator — run with --ignored to overwrite the committed .bsn references"]
fn regenerate_pickup_bsn() {
    regenerate("pickup", PICKUP_JSON, "platformer_minimal_pickup");
}

fn regenerate(label: &str, json: &str, scene_name: &str) {
    let doc: SceneAssetDocument = serde_json::from_str(json).expect("parse JSON");
    let result = emit_bsn_source_from_document(&doc, scene_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let out_path = match label {
        "player" => format!("{manifest_dir}/../../examples/platformer-minimal/export/player.bsn"),
        "enemy" => format!("{manifest_dir}/../../examples/platformer-minimal/export/enemy.bsn"),
        "ground" => format!("{manifest_dir}/../../examples/platformer-minimal/export/ground.bsn"),
        "pickup" => format!("{manifest_dir}/../../examples/platformer-minimal/export/pickup.bsn"),
        _ => panic!("unknown label: {label}"),
    };
    std::fs::write(&out_path, &result.source).expect("write .bsn");
    println!("regenerated {out_path}");
}
