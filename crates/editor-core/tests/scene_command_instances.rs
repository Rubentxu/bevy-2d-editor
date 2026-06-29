//! Tests for scene instance command variants.
//!
//! Covers: S3 (RemoveInstance), S15 (PlaceInstance), S16 (RemoveInstance inverse),
//! S17 (ReplaceInstanceAsset).

use editor_core::scene_asset::{AssetReference, LocalId};
use editor_core::scene_instance::SceneInstance;
use editor_core::{Command, SceneDocument, StableId};
use std::collections::BTreeMap;

fn empty_doc() -> SceneDocument {
    SceneDocument {
        version: "0.1".to_string(),
        scene_id: "test".to_string(),
        name: "Test".to_string(),
        entities: vec![],
        instances: BTreeMap::new(),
    }
}

/// S15: PlaceInstance serializes with type tag and all fields.
#[test]
fn test_place_instance_serializes_pascal_case() {
    let id_map: BTreeMap<LocalId, StableId> = vec![
        (LocalId::new("local_1"), StableId::new("inst_test_1")),
    ]
    .into_iter()
    .collect();

    let cmd = Command::PlaceInstance {
        instance_id: StableId::new("inst_test"),
        asset_ref: AssetReference::new("characters/player"),
        asset_version: 1,
        id_map,
        overrides: vec![],
        orphaned_overrides: vec![],
    };

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(
        json.contains("\"type\":\"PlaceInstance\""),
        "Should have PlaceInstance type tag"
    );
    assert!(
        json.contains("\"instance_id\":\"inst_test\""),
        "Should have instance_id"
    );
    assert!(
        json.contains("\"asset_ref\":\"characters/player\""),
        "Should have asset_ref"
    );
    assert!(json.contains("\"asset_version\":1"), "Should have asset_version");
    assert!(
        json.contains("\"id_map\":"),
        "Should have id_map field"
    );
}

/// S16: RemoveInstance serializes with type tag.
#[test]
fn test_remove_instance_serializes_pascal_case() {
    let cmd = Command::RemoveInstance {
        instance_id: StableId::new("inst_test"),
    };

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(
        json.contains("\"type\":\"RemoveInstance\""),
        "Should have RemoveInstance type tag"
    );
    assert!(
        json.contains("\"instance_id\":\"inst_test\""),
        "Should have instance_id"
    );
}

/// S17: ReplaceInstanceAsset serializes with type tag and captured_old.
#[test]
fn test_replace_instance_asset_serializes_pascal_case() {
    let captured_old = Some(SceneInstance {
        instance_id: StableId::new("inst_test"),
        asset_ref: AssetReference::new("old_player"),
        asset_version_seen: 1,
        id_map: vec![(LocalId::new("local_1"), StableId::new("inst_test_1"))]
            .into_iter()
            .collect(),
        overrides: vec![],
        orphaned_overrides: vec![],
    });

    let cmd = Command::ReplaceInstanceAsset {
        instance_id: StableId::new("inst_test"),
        new_asset_ref: AssetReference::new("new_player"),
        new_asset_version: 2,
        captured_old,
    };

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(
        json.contains("\"type\":\"ReplaceInstanceAsset\""),
        "Should have ReplaceInstanceAsset type tag"
    );
    assert!(
        json.contains("\"instance_id\":\"inst_test\""),
        "Should have instance_id"
    );
    assert!(
        json.contains("\"new_asset_ref\":\"new_player\""),
        "Should have new_asset_ref"
    );
    assert!(
        json.contains("\"new_asset_version\":2"),
        "Should have new_asset_version"
    );
}

/// Deserialize PlaceInstance from JSON.
#[test]
fn test_place_instance_deserializes() {
    let json = r#"{
        "type": "PlaceInstance",
        "instance_id": "inst_abc123",
        "asset_ref": "assets/hero",
        "asset_version": 3,
        "id_map": {"local_x": "inst_abc123_local_x"},
        "overrides": [],
        "orphaned_overrides": []
    }"#;

    let cmd: Command = serde_json::from_str(json).unwrap();
    match cmd {
        Command::PlaceInstance {
            instance_id,
            asset_ref,
            asset_version,
            id_map,
            overrides,
            orphaned_overrides,
        } => {
            assert_eq!(instance_id.as_str(), "inst_abc123");
            assert_eq!(asset_ref.as_str(), "assets/hero");
            assert_eq!(asset_version, 3);
            assert_eq!(id_map.len(), 1);
            assert!(overrides.is_empty());
            assert!(orphaned_overrides.is_empty());
        }
        _ => panic!("Expected PlaceInstance variant"),
    }
}

/// Deserialize RemoveInstance from JSON.
#[test]
fn test_remove_instance_deserializes() {
    let json = r#"{
        "type": "RemoveInstance",
        "instance_id": "inst_xyz"
    }"#;

    let cmd: Command = serde_json::from_str(json).unwrap();
    match cmd {
        Command::RemoveInstance { instance_id } => {
            assert_eq!(instance_id.as_str(), "inst_xyz");
        }
        _ => panic!("Expected RemoveInstance variant"),
    }
}

/// Deserialize ReplaceInstanceAsset from JSON.
#[test]
fn test_replace_instance_asset_deserializes() {
    let json = r#"{
        "type": "ReplaceInstanceAsset",
        "instance_id": "inst_test",
        "new_asset_ref": "assets/enemy",
        "new_asset_version": 5,
        "captured_old": null
    }"#;

    let cmd: Command = serde_json::from_str(json).unwrap();
    match cmd {
        Command::ReplaceInstanceAsset {
            instance_id,
            new_asset_ref,
            new_asset_version,
            captured_old,
        } => {
            assert_eq!(instance_id.as_str(), "inst_test");
            assert_eq!(new_asset_ref.as_str(), "assets/enemy");
            assert_eq!(new_asset_version, 5);
            assert!(captured_old.is_none());
        }
        _ => panic!("Expected ReplaceInstanceAsset variant"),
    }
}

// =============================================================================
// S15, S16, S17: Apply and Inverse Tests
// =============================================================================

use editor_core::processor;

/// S15: PlaceInstance applies and produces RemoveInstance as inverse.
#[test]
fn s15_place_instance_apply_and_inverse() {
    let mut doc = empty_doc();

    let id_map: BTreeMap<LocalId, StableId> = vec![
        (LocalId::new("local_1"), StableId::new("inst_test_1")),
    ]
    .into_iter()
    .collect();

    let cmd = Command::PlaceInstance {
        instance_id: StableId::new("inst_test"),
        asset_ref: AssetReference::new("characters/player"),
        asset_version: 1,
        id_map,
        overrides: vec![],
        orphaned_overrides: vec![],
    };

    // Apply PlaceInstance
    let inverse = processor::apply(&mut doc, &cmd).expect("apply should succeed");

    assert_eq!(doc.instances.len(), 1, "Should have 1 instance");
    assert!(
        doc.instances.contains_key(&StableId::new("inst_test")),
        "Instance should be in instances"
    );

    // Inverse should be RemoveInstance
    match inverse {
        Command::RemoveInstance { instance_id } => {
            assert_eq!(instance_id.as_str(), "inst_test");
        }
        _ => panic!("Inverse should be RemoveInstance"),
    }

    // Apply inverse (RemoveInstance)
    processor::apply(&mut doc, &inverse).expect("inverse apply should succeed");

    assert!(doc.instances.is_empty(), "Instances should be empty after undo");
}

/// S16: RemoveInstance applies and produces PlaceInstance as inverse.
#[test]
fn s16_remove_instance_apply_and_inverse() {
    let mut doc = empty_doc();

    // Pre-populate with an instance
    let instance = SceneInstance {
        instance_id: StableId::new("inst_test"),
        asset_ref: AssetReference::new("characters/player"),
        asset_version_seen: 1,
        id_map: vec![(LocalId::new("local_1"), StableId::new("inst_test_1"))]
            .into_iter()
            .collect(),
        overrides: vec![],
        orphaned_overrides: vec![],
    };
    doc.instances.insert(StableId::new("inst_test"), instance);

    let cmd = Command::RemoveInstance {
        instance_id: StableId::new("inst_test"),
    };

    // Apply RemoveInstance
    let inverse = processor::apply(&mut doc, &cmd).expect("apply should succeed");

    assert!(doc.instances.is_empty(), "Instance should be removed");

    // Inverse should be PlaceInstance restoring the captured state
    match inverse {
        Command::PlaceInstance {
            instance_id,
            asset_ref,
            asset_version,
            ..
        } => {
            assert_eq!(instance_id.as_str(), "inst_test");
            assert_eq!(asset_ref.as_str(), "characters/player");
            assert_eq!(asset_version, 1);
        }
        _ => panic!("Inverse should be PlaceInstance"),
    }

    // Apply inverse (PlaceInstance) to restore
    processor::apply(&mut doc, &inverse).expect("inverse apply should succeed");

    assert_eq!(doc.instances.len(), 1, "Instance should be restored");
    assert_eq!(
        doc.instances.get(&StableId::new("inst_test")).unwrap().asset_ref.as_str(),
        "characters/player"
    );
}

/// S17: ReplaceInstanceAsset applies and inverts correctly.
#[test]
fn s17_replace_instance_asset_apply_and_inverse() {
    let mut doc = empty_doc();

    // Pre-populate with an instance
    let instance = SceneInstance {
        instance_id: StableId::new("inst_test"),
        asset_ref: AssetReference::new("old_player"),
        asset_version_seen: 1,
        id_map: vec![(LocalId::new("local_1"), StableId::new("inst_test_1"))]
            .into_iter()
            .collect(),
        overrides: vec![],
        orphaned_overrides: vec![],
    };
    doc.instances.insert(StableId::new("inst_test"), instance);

    let cmd = Command::ReplaceInstanceAsset {
        instance_id: StableId::new("inst_test"),
        new_asset_ref: AssetReference::new("new_player"),
        new_asset_version: 2,
        captured_old: None, // Processor fills this in
    };

    // Apply ReplaceInstanceAsset
    let inverse = processor::apply(&mut doc, &cmd).expect("apply should succeed");

    let replaced = doc
        .instances
        .get(&StableId::new("inst_test"))
        .expect("instance should exist");
    assert_eq!(replaced.asset_ref.as_str(), "new_player");
    assert_eq!(replaced.asset_version_seen, 2);

    // Inverse should be another ReplaceInstanceAsset restoring old state
    match inverse {
        Command::ReplaceInstanceAsset {
            instance_id,
            new_asset_ref,
            new_asset_version,
            captured_old,
        } => {
            assert_eq!(instance_id.as_str(), "inst_test");
            assert_eq!(new_asset_ref.as_str(), "old_player"); // Swapped back
            assert_eq!(new_asset_version, 1); // Swapped back
            assert!(captured_old.is_some(), "captured_old should be set for next inverse");
        }
        _ => panic!("Inverse should be ReplaceInstanceAsset"),
    }

    // Apply inverse to restore original state
    processor::apply(&mut doc, &inverse).expect("inverse apply should succeed");

    let restored = doc
        .instances
        .get(&StableId::new("inst_test"))
        .expect("instance should exist");
    assert_eq!(restored.asset_ref.as_str(), "old_player");
    assert_eq!(restored.asset_version_seen, 1);
}

/// S3: RemoveInstance drops only that instance, not authored entities.
#[test]
fn s3_remove_instance_only_affects_instance() {
    use editor_core::Entity;

    let mut doc = empty_doc();

    // Add an authored entity
    doc.entities.push(Entity {
        id: StableId::new("authored_entity"),
        name: "Authored Entity".to_string(),
        parent: None,
        components: vec![],
    });

    // Add an instance
    let instance = SceneInstance {
        instance_id: StableId::new("inst_001"),
        asset_ref: AssetReference::new("test_asset"),
        asset_version_seen: 1,
        id_map: vec![(LocalId::new("root"), StableId::new("inst_001_root"))]
            .into_iter()
            .collect(),
        overrides: vec![],
        orphaned_overrides: vec![],
    };
    doc.instances
        .insert(StableId::new("inst_001"), instance);

    let cmd = Command::RemoveInstance {
        instance_id: StableId::new("inst_001"),
    };

    processor::apply(&mut doc, &cmd).expect("remove should succeed");

    // Authored entity should remain
    assert_eq!(doc.entities.len(), 1);
    assert!(
        doc.entities
            .iter()
            .any(|e| e.id.as_str() == "authored_entity"),
        "Authored entity should remain"
    );

    // Instance should be gone
    assert!(doc.instances.is_empty());
}
