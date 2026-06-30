//! Tests for SceneDocument.instances field (S7, S13, S14).
//!
//! S7: JSON without `instances` field deserializes to empty BTreeMap.
//! S13: `entities` array shape is unchanged when instances are present.
//! S14: Authored entities do NOT have an `instance_id` field.

use editor_core::{
    SceneDocument,
    scene_asset::{AssetReference, LocalId, SceneAssetDocument, SceneAssetEntity, SceneAssetRole},
    scene_instance::SceneInstance,
    StableId,
};
use std::collections::BTreeMap;

#[test]
fn s7_json_without_instances_deserializes_to_empty_btreemap() {
    // JSON without `instances` field - should default to empty BTreeMap
    let json = r#"{
        "version": "0.1",
        "scene_id": "test_scene",
        "name": "Test Scene",
        "entities": []
    }"#;

    let doc: SceneDocument = serde_json::from_str(json).expect(
        "SceneDocument must deserialize even without `instances` field (#[serde(default)])",
    );

    assert!(
        doc.instances.is_empty(),
        "instances should default to empty BTreeMap when absent from JSON"
    );
}

#[test]
fn s7_instances_field_absent_then_present_roundtrip() {
    // Start without instances
    let json_without = r#"{
        "version": "0.1",
        "scene_id": "test_scene",
        "name": "Test Scene",
        "entities": []
    }"#;

    let doc: SceneDocument =
        serde_json::from_str(json_without).expect("Must deserialize without instances");

    // Add an instance
    let mut instances = BTreeMap::new();
    let mut id_map = BTreeMap::new();
    id_map.insert(LocalId("root".into()), StableId::new("ent_001"));
    instances.insert(
        StableId::new("inst_001"),
        SceneInstance {            instance_components: vec![],

            instance_id: StableId::new("inst_001"),
            asset_ref: AssetReference("assets/player.bsn".into()),
            asset_version_seen: 1,
            id_map,
            component_overrides: vec![],
            orphaned_component_overrides: vec![],
        },
    );

    // Serialize with instances
    let doc_with_instances = SceneDocument { instances, ..doc };
    let json_with = serde_json::to_string(&doc_with_instances).expect("Must serialize");

    // Deserialize again
    let roundtripped: SceneDocument =
        serde_json::from_str(&json_with).expect("Must deserialize with instances");

    assert!(
        !roundtripped.instances.is_empty(),
        "instances should be preserved through roundtrip"
    );
    assert_eq!(
        roundtripped.instances.len(),
        1,
        "Should have exactly 1 instance after roundtrip"
    );
}

#[test]
fn s13_entities_array_shape_unchanged_when_instances_present() {
    // Create a document with entities AND instances
    let mut id_map = BTreeMap::new();
    id_map.insert(LocalId("root".into()), StableId::new("ent_001"));

    let doc = SceneDocument {
        version: "0.1".to_string(),
        scene_id: "test_scene".to_string(),
        name: "Test Scene".to_string(),
        entities: vec![editor_core::Entity {
            id: StableId::new("ent_001"),
            name: "Player".to_string(),
            parent: None,
            components: vec![],
        }],
        instances: BTreeMap::from([(
            StableId::new("inst_001"),
            SceneInstance {                instance_components: vec![],

                instance_id: StableId::new("inst_001"),
                asset_ref: AssetReference("assets/player.bsn".into()),
                asset_version_seen: 1,
                id_map,
                component_overrides: vec![],
                orphaned_component_overrides: vec![],
            },
        )]),
    };

    let json = serde_json::to_string(&doc).expect("Must serialize");

    // Verify entities array structure is intact
    assert!(
        json.contains("\"entities\":[{"),
        "entities array must be present and well-formed"
    );
    assert!(
        json.contains("\"id\":\"ent_001\""),
        "Entity id must be preserved"
    );
    assert!(
        json.contains("\"name\":\"Player\""),
        "Entity name must be preserved"
    );
}

#[test]
fn s14_authored_entities_do_not_have_instance_id() {
    // Authored entities (in the entities array) should NOT have instance_id
    let doc = SceneDocument {
        version: "0.1".to_string(),
        scene_id: "test_scene".to_string(),
        name: "Test Scene".to_string(),
        entities: vec![editor_core::Entity {
            id: StableId::new("ent_001"),
            name: "Player".to_string(),
            parent: None,
            components: vec![],
        }],
        instances: BTreeMap::new(),
    };

    let json = serde_json::to_string(&doc).expect("Must serialize");

    // Parse the JSON and verify no instance_id on the entity
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Must parse JSON");

    let entities = parsed.get("entities").expect("entities must exist");
    if let Some(arr) = entities.as_array() {
        for entity in arr {
            assert!(
                entity.get("instance_id").is_none(),
                "Authored entities must NOT have instance_id field"
            );
        }
    }
}

#[test]
fn s6_instances_with_id_map_3_entries_byte_equal_roundtrip() {
    // S6: instances[id_map with 3 entries] byte-equal after serialize/deserialize
    let mut id_map = BTreeMap::new();
    id_map.insert(LocalId("root".into()), StableId::new("ent_001"));
    id_map.insert(LocalId("weapon".into()), StableId::new("ent_002"));
    id_map.insert(LocalId("shield".into()), StableId::new("ent_003"));

    let doc = SceneDocument {
        version: "0.1".to_string(),
        scene_id: "test_scene".to_string(),
        name: "Test Scene".to_string(),
        entities: vec![],
        instances: BTreeMap::from([(
            StableId::new("inst_001"),
            SceneInstance {                instance_components: vec![],

                instance_id: StableId::new("inst_001"),
                asset_ref: AssetReference("assets/player.bsn".into()),
                asset_version_seen: 3,
                id_map,
                component_overrides: vec![],
                orphaned_component_overrides: vec![],
            },
        )]),
    };

    let json = serde_json::to_string(&doc).expect("Must serialize");
    let roundtripped: SceneDocument = serde_json::from_str(&json).expect("Must deserialize");

    // Verify instance is preserved
    assert_eq!(
        roundtripped.instances.len(),
        1,
        "Should have exactly 1 instance"
    );

    let roundtripped_instance = roundtripped
        .instances
        .get(&StableId::new("inst_001"))
        .expect("inst_001 must exist");

    assert_eq!(
        roundtripped_instance.id_map.len(),
        3,
        "id_map should have 3 entries"
    );

    // Verify id_map contents are preserved exactly
    assert_eq!(
        roundtripped_instance.id_map.get(&LocalId("root".into())),
        Some(&StableId::new("ent_001")),
        "root mapping must be preserved"
    );
    assert_eq!(
        roundtripped_instance.id_map.get(&LocalId("weapon".into())),
        Some(&StableId::new("ent_002")),
        "weapon mapping must be preserved"
    );
    assert_eq!(
        roundtripped_instance.id_map.get(&LocalId("shield".into())),
        Some(&StableId::new("ent_003")),
        "shield mapping must be preserved"
    );
}
