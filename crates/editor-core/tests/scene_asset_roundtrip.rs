//! Serde round-trip tests for scene asset and instance types.
//! Covers scenarios S1, S2, S6.

use editor_core::{
    StableId,
    bsn_ir::{BsnIr, BsnIrNode, BsnIrRelationship, BsnPatch, BsnPatchOp},
    scene_asset::{
        AssetReference, ExposedProperty, LocalId, RelationshipKind, SceneAssetDocument,
        SceneAssetEntity, SceneAssetMetadata, SceneAssetRelationship, SceneAssetRole,
    },
    scene_instance::{ComponentOverride, ComponentOverrideStatus, SceneInstance},
};

#[test]
fn s1_scene_asset_document_roundtrip() {
    let doc = SceneAssetDocument {
        asset_id: "asset-001".to_string(),
        logical_path: "assets/player.bsn".to_string(),
        role: SceneAssetRole::Actor,
        version: 3,
        entities: vec![
            SceneAssetEntity {
                local_id: LocalId("root".to_string()),
                local_path: "root".to_string(),
                name: "Player".to_string(),
                components: vec![
                    editor_core::ComponentInstance {
                        type_id: "editor.Transform2D".to_string(),
                        values: serde_json::json!({
                            "translation": {"x": 100.0, "y": 200.0},
                            "rotation": 0.5,
                            "scale": {"x": 1.5, "y": 1.5}
                        }),
                    },
                    editor_core::ComponentInstance {
                        type_id: "editor.Sprite2D".to_string(),
                        values: serde_json::json!({
                            "asset": "player.png",
                            "color": {"r": 1.0, "g": 0.0, "b": 0.0, "a": 1.0},
                            "anchor": "Center"
                        }),
                    },
                ],
            },
            SceneAssetEntity {
                local_id: LocalId("weapon".to_string()),
                local_path: "root/weapon".to_string(),
                name: "Weapon".to_string(),
                components: vec![editor_core::ComponentInstance {
                    type_id: "editor.Sprite2D".to_string(),
                    values: serde_json::json!({
                        "asset": "weapon.png",
                        "color": {"r": 0.8, "g": 0.8, "b": 0.8, "a": 1.0},
                        "anchor": "Center"
                    }),
                }],
            },
        ],
        relationships: vec![SceneAssetRelationship {
            from_local_id: LocalId("root".to_string()),
            to_local_id: LocalId("weapon".to_string()),
            kind: RelationshipKind::Child,
            field_path: None,
        }],
        exposed_properties: vec![ExposedProperty {
            name: "weapon_color".to_string(),
            target_local_id: LocalId("weapon".to_string()),
            field_path: vec!["Sprite2D".to_string(), "color".to_string()],
            default_value: serde_json::json!({"r": 0.8, "g": 0.8, "b": 0.8, "a": 1.0}),
        }],
        metadata: SceneAssetMetadata {
            tags: Some("combat,player".to_string()),
            created_at: Some("2026-06-01T10:00:00Z".to_string()),
            updated_at: None,
            notes: Some("Basic player actor".to_string()),
        },
    };

    let json = serde_json::to_string(&doc).expect("serialize SceneAssetDocument");
    let roundtripped: SceneAssetDocument =
        serde_json::from_str(&json).expect("deserialize SceneAssetDocument");

    assert_eq!(roundtripped, doc);
    assert_eq!(roundtripped.asset_id, "asset-001");
    assert_eq!(roundtripped.logical_path, "assets/player.bsn");
    assert_eq!(roundtripped.role, SceneAssetRole::Actor);
    assert_eq!(roundtripped.version, 3);
    assert_eq!(roundtripped.entities.len(), 2);
    assert_eq!(roundtripped.relationships.len(), 1);
    assert_eq!(roundtripped.exposed_properties.len(), 1);
    assert!(
        !json.contains("children_local_ids"),
        "JSON must not contain children_local_ids key"
    );
}

#[test]
fn s2_scene_instance_roundtrip() {
    use std::collections::BTreeMap;

    let mut id_map = BTreeMap::new();
    id_map.insert(LocalId("root".to_string()), StableId::new("ent_a"));
    id_map.insert(LocalId("weapon".to_string()), StableId::new("ent_b"));

    let instance = SceneInstance {
        instance_id: StableId::new("instance-001"),
        asset_ref: AssetReference("assets/player.bsn".into()),
        asset_version_seen: 7,
        id_map,
        component_overrides: vec![ComponentOverride {
            target_local_id: LocalId("weapon".to_string()),
            component_type_id: editor_core::schema::ComponentTypeId::new("Sprite2D"),
            field_path: vec!["color".to_string()],
            value: serde_json::json!({"r": 1.0, "g": 0.3, "b": 0.3, "a": 1.0}),
            status: ComponentOverrideStatus::Active,
        }],
        orphaned_component_overrides: vec![],
    };

    let json = serde_json::to_string(&instance).expect("serialize SceneInstance");
    let roundtripped: SceneInstance =
        serde_json::from_str(&json).expect("deserialize SceneInstance");

    assert_eq!(roundtripped, instance);
    assert_eq!(roundtripped.asset_ref.as_str(), "assets/player.bsn");
    assert_eq!(roundtripped.asset_version_seen, 7);
    assert_eq!(roundtripped.id_map.len(), 2);
    assert_eq!(roundtripped.component_overrides.len(), 1);
    assert_eq!(roundtripped.component_overrides[0].status, ComponentOverrideStatus::Active);
}

#[test]
fn s6_bsn_ir_roundtrip() {
    use std::collections::BTreeMap;

    let child_node = BsnIrNode {
        identifier: "weapon".to_string(),
        components: BTreeMap::from([(
            "editor.Sprite2D".to_string(),
            serde_json::json!({
                "asset": "weapon.png",
                "color": {"r": 0.8, "g": 0.8, "b": 0.8, "a": 1.0}
            }),
        )]),
        children: vec![],
        relationships: vec![],
    };

    let root_node = BsnIrNode {
        identifier: "root".to_string(),
        components: BTreeMap::from([(
            "editor.Transform2D".to_string(),
            serde_json::json!({
                "translation": {"x": 100.0, "y": 200.0},
                "rotation": 0.5,
                "scale": {"x": 1.5, "y": 1.5}
            }),
        )]),
        children: vec![child_node],
        relationships: vec![BsnIrRelationship {
            kind: "child".to_string(),
            target_identifier: "weapon".to_string(),
        }],
    };

    let bsn_ir = BsnIr {
        scene_root: root_node,
        asset_refs: vec!["assets/player.bsn".to_string()],
        patches: vec![BsnPatch {
            target_identifier: "weapon".to_string(),
            op: BsnPatchOp::Replace,
            value: serde_json::json!({"color": {"r": 1.0, "g": 0.0, "b": 0.0, "a": 1.0}}),
        }],
    };

    let json = serde_json::to_string(&bsn_ir).expect("serialize BsnIr");
    let roundtripped: BsnIr = serde_json::from_str(&json).expect("deserialize BsnIr");

    assert_eq!(roundtripped, bsn_ir);
    assert_eq!(roundtripped.asset_refs.len(), 1);
    assert_eq!(roundtripped.patches.len(), 1);
    assert_eq!(roundtripped.scene_root.identifier, "root");
    assert_eq!(roundtripped.scene_root.children.len(), 1);
}
