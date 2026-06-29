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
