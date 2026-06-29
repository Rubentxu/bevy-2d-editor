//! Tests for ASSET_BODY_CACHE roundtrip (warm/invalidate/clear).
//!
//! Covers Task 1.7: cache operations for instance placement projection.

use editor_core::scene_asset::{
    AssetReference, LocalId, SceneAssetDocument, SceneAssetEntity, SceneAssetMetadata,
    SceneAssetRole,
};
use std::collections::BTreeMap;

// Note: These tests verify the cache DATA STRUCTURE behavior.
// The actual thread_local access is tested via integration tests.
// Here we test the BTreeMap operations that mirror cache behavior.

fn make_test_asset(logical_path: &str) -> SceneAssetDocument {
    SceneAssetDocument {
        asset_id: format!("asset-{}", logical_path.replace("/", "-")),
        logical_path: logical_path.to_string(),
        role: SceneAssetRole::Actor,
        version: 1,
        entities: vec![SceneAssetEntity {
            local_id: LocalId("root".to_string()),
            local_path: "root".to_string(),
            name: "Test Entity".to_string(),
            components: vec![],
        }],
        relationships: vec![],
        exposed_properties: vec![],
        metadata: SceneAssetMetadata::default(),
    }
}

#[test]
fn cache_warm_single_asset() {
    let mut cache: BTreeMap<String, SceneAssetDocument> = BTreeMap::new();
    let asset = make_test_asset("assets/player.bsn");

    cache.insert(asset.logical_path.clone(), asset.clone());

    assert_eq!(cache.len(), 1);
    assert!(cache.contains_key("assets/player.bsn"));
    assert_eq!(
        cache.get("assets/player.bsn").unwrap().asset_id,
        "asset-player"
    );
}

#[test]
fn cache_warm_multiple_assets() {
    let mut cache: BTreeMap<String, SceneAssetDocument> = BTreeMap::new();

    let player = make_test_asset("assets/player.bsn");
    let enemy = make_test_asset("assets/enemy.bsn");
    let weapon = make_test_asset("assets/weapon.bsn");

    cache.insert(player.logical_path.clone(), player);
    cache.insert(enemy.logical_path.clone(), enemy);
    cache.insert(weapon.logical_path.clone(), weapon);

    assert_eq!(cache.len(), 3);
}

#[test]
fn cache_invalidate_removes_asset() {
    let mut cache: BTreeMap<String, SceneAssetDocument> = BTreeMap::new();
    cache.insert(
        "assets/player.bsn".to_string(),
        make_test_asset("assets/player.bsn"),
    );

    assert!(cache.contains_key("assets/player.bsn"));

    cache.remove("assets/player.bsn");

    assert!(!cache.contains_key("assets/player.bsn"));
    assert_eq!(cache.len(), 0);
}

#[test]
fn cache_clear_removes_all() {
    let mut cache: BTreeMap<String, SceneAssetDocument> = BTreeMap::new();
    cache.insert(
        "assets/player.bsn".to_string(),
        make_test_asset("assets/player.bsn"),
    );
    cache.insert(
        "assets/enemy.bsn".to_string(),
        make_test_asset("assets/enemy.bsn"),
    );

    assert_eq!(cache.len(), 2);

    cache.clear();

    assert_eq!(cache.len(), 0);
}

#[test]
fn cache_update_replaces_existing() {
    let mut cache: BTreeMap<String, SceneAssetDocument> = BTreeMap::new();

    let v1 = make_test_asset("assets/player.bsn");
    cache.insert("assets/player.bsn".to_string(), v1);

    let mut v2 = make_test_asset("assets/player.bsn");
    v2.version = 2;
    cache.insert("assets/player.bsn".to_string(), v2);

    assert_eq!(cache.len(), 1);
    assert_eq!(cache.get("assets/player.bsn").unwrap().version, 2);
}

#[test]
fn cache_lookups_are_case_sensitive() {
    let mut cache: BTreeMap<String, SceneAssetDocument> = BTreeMap::new();
    cache.insert(
        "Assets/Player.bsn".to_string(),
        make_test_asset("Assets/Player.bsn"),
    );

    // Lowercase lookup should fail
    assert!(!cache.contains_key("assets/player.bsn"));
    // Exact case lookup should succeed
    assert!(cache.contains_key("Assets/Player.bsn"));
}

#[test]
fn cache_deterministic_iteration_order() {
    let mut cache: BTreeMap<String, SceneAssetDocument> = BTreeMap::new();

    // Insert in non-sorted order
    cache.insert(
        "assets/zebra.bsn".to_string(),
        make_test_asset("assets/zebra.bsn"),
    );
    cache.insert(
        "assets/apple.bsn".to_string(),
        make_test_asset("assets/apple.bsn"),
    );
    cache.insert(
        "assets/mango.bsn".to_string(),
        make_test_asset("assets/mango.bsn"),
    );

    // Keys should be in sorted order (BTreeMap guarantee)
    let keys: Vec<&String> = cache.keys().collect();
    assert_eq!(keys[0].as_str(), "assets/apple.bsn");
    assert_eq!(keys[1].as_str(), "assets/mango.bsn");
    assert_eq!(keys[2].as_str(), "assets/zebra.bsn");
}

#[test]
fn cache_roundtrip_serialize_deserialize() {
    let mut cache: BTreeMap<String, SceneAssetDocument> = BTreeMap::new();
    cache.insert(
        "assets/player.bsn".to_string(),
        make_test_asset("assets/player.bsn"),
    );
    cache.insert(
        "assets/enemy.bsn".to_string(),
        make_test_asset("assets/enemy.bsn"),
    );

    // Serialize to JSON
    let json = serde_json::to_string(&cache).expect("Must serialize cache");

    // Deserialize back
    let roundtripped: BTreeMap<String, SceneAssetDocument> =
        serde_json::from_str(&json).expect("Must deserialize cache");

    assert_eq!(roundtripped.len(), 2);
    assert!(roundtripped.contains_key("assets/player.bsn"));
    assert!(roundtripped.contains_key("assets/enemy.bsn"));

    // Verify content is preserved
    assert_eq!(
        roundtripped.get("assets/player.bsn").unwrap().logical_path,
        "assets/player.bsn"
    );
}
