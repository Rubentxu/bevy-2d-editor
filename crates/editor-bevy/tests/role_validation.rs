//! Role validation & hierarchy-via-relationships tests.
//! Covers scenarios S7, S9.

use editor_bevy::scene_asset::{
    LocalId, RelationshipKind, SceneAssetDocument, SceneAssetEntity, SceneAssetMetadata,
    SceneAssetRelationship, SceneAssetRole, validate_role,
};

#[test]
fn s7_fragment_standalone_warning() {
    let doc = SceneAssetDocument {
        layers: vec![],
        asset_id: "frag-001".to_string(),
        logical_path: "assets/fragments/health_pickup.bsn".to_string(),
        role: SceneAssetRole::Fragment,
        version: 1,
        entities: vec![SceneAssetEntity {
            local_id: LocalId::new("health".to_string()),
            local_path: "health".to_string(),
            name: "Health Pickup".to_string(),
            components: vec![],
        }],
        relationships: vec![],
        exposed_properties: vec![],
        metadata: SceneAssetMetadata::default(),
    };

    let warnings = validate_role(SceneAssetRole::Fragment, &doc);
    assert!(
        !warnings.is_empty(),
        "Fragment with no Child relationships should produce at least one warning"
    );
    let codes: Vec<&str> = warnings.iter().map(|w| w.code.as_str()).collect();
    assert!(
        codes.contains(&"fragment_standalone"),
        "Expected 'fragment_standalone' warning code, got: {:?}",
        codes
    );
}

#[test]
fn s9_hierarchy_via_relationships_only() {
    let doc = SceneAssetDocument {
        layers: vec![],
        asset_id: "actor-001".to_string(),
        logical_path: "assets/player.bsn".to_string(),
        role: SceneAssetRole::Actor,
        version: 1,
        entities: vec![
            SceneAssetEntity {
                local_id: LocalId::new("root".to_string()),
                local_path: "root".to_string(),
                name: "Player".to_string(),
                components: vec![],
            },
            SceneAssetEntity {
                local_id: LocalId::new("weapon".to_string()),
                local_path: "root/weapon".to_string(),
                name: "Weapon".to_string(),
                components: vec![],
            },
        ],
        relationships: vec![SceneAssetRelationship {
            from_local_id: LocalId::new("root".to_string()),
            to_local_id: LocalId::new("weapon".to_string()),
            kind: RelationshipKind::Child,
            field_path: None,
        }],
        exposed_properties: vec![],
        metadata: SceneAssetMetadata::default(),
    };

    let json = serde_json::to_string(&doc).expect("serialize doc");

    assert!(
        json.contains("\"relationships\""),
        "JSON must contain 'relationships' key"
    );
    assert!(
        json.contains("\"kind\":\"child\""),
        "JSON must contain kind 'child'"
    );
    assert!(
        !json.contains("children_local_ids"),
        "JSON must NOT contain 'children_local_ids' (hierarchy is via relationships only)"
    );

    let roundtripped: SceneAssetDocument = serde_json::from_str(&json).expect("deserialize doc");
    assert_eq!(roundtripped.relationships.len(), 1);
    assert!(matches!(
        roundtripped.relationships[0].kind,
        RelationshipKind::Child
    ));

    let json_with_children_ids = serde_json::json!({
        "asset_id": "test",
        "logical_path": "test.bsn",
        "role": "actor",
        "version": 1,
        "entities": [{
            "local_id": "root",
            "local_path": "root",
            "name": "Root",
            "components": [],
            "children_local_ids": ["child1"]
        }],
        "relationships": [],
        "exposed_properties": [],
        "metadata": {}
    })
    .to_string();

    let result: Result<SceneAssetDocument, _> = serde_json::from_str(&json_with_children_ids);
    assert!(
        result.is_err(),
        "Deserializing a doc with children_local_ids should fail"
    );
}
