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
            extension_data: Default::default(),
        }],
        relationships: vec![],
        exposed_properties: vec![],
        metadata: SceneAssetMetadata::default(),
        extension_data: Default::default(),
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
                extension_data: Default::default(),
            },
            SceneAssetEntity {
                local_id: LocalId::new("weapon".to_string()),
                local_path: "root/weapon".to_string(),
                name: "Weapon".to_string(),
                components: vec![],
                extension_data: Default::default(),
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
        extension_data: Default::default(),
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

    // Post-S4 contract (SDD-0046 S4, commit a24c523, v0.99.0):
    // `SceneAssetEntity` uses `#[serde(default, flatten)] extension_data`
    // (ADR-0046 rule 2 / SEM-3). Unknown fields are preserved in
    // `extension_data` rather than rejected. So a doc with the legacy
    // `children_local_ids` field now deserializes successfully, with
    // `children_local_ids` landing in the entity's `extension_data`.
    //
    // Hierarchy is still relationships-only — the assertion above
    // (`!json.contains("children_local_ids")`) covers the serialised
    // form. Here we verify the deserialise-preserves-foreign-fields
    // contract.
    let result: SceneAssetDocument = serde_json::from_str(&json_with_children_ids)
        .expect("Post-S4: deserialization must succeed (extension_data preserves unknown fields)");
    assert_eq!(result.entities.len(), 1, "entity preserved");
    let root_entity = &result.entities[0];
    assert!(
        root_entity
            .extension_data
            .contains_key("children_local_ids"),
        "Post-S4: children_local_ids must land in extension_data, got: {:?}",
        root_entity.extension_data
    );
    let preserved = &root_entity.extension_data["children_local_ids"];
    let arr = preserved
        .as_array()
        .expect("children_local_ids must be an array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0], serde_json::json!("child1"));
}
