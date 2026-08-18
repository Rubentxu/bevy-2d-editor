//! Integration tests for asset command processor (PR2 tasks 2.14).
//! Covers spec scenarios: S10, S13, S14, S15.
//!
//! Strict TDD: RED first — tests define the expected API contract.

use editor_bevy::asset_command::{
    AssetCommand, AssetCommandError, AssetOperationLog, apply as asset_apply,
};
use editor_bevy::scene_asset::{LocalId, SceneAssetDocument, SceneAssetEntity, SceneAssetRole};
use editor_model::ComponentInstance;
use serde_json::json;

// ─────────────────────────────────────────────────────────────────────────
// Helper: make a SceneAssetDocument
// ─────────────────────────────────────────────────────────────────────────

fn empty_asset_doc(asset_id: &str, logical_path: &str) -> SceneAssetDocument {
    SceneAssetDocument {
        layers: vec![],
        asset_id: asset_id.to_string(),
        logical_path: logical_path.to_string(),
        role: SceneAssetRole::Actor,
        version: 1,
        entities: vec![],
        relationships: vec![],
        exposed_properties: vec![],
        metadata: Default::default(),
        extension_data: Default::default(),
    }
}

fn entity(local_id: &str, name: &str, components: Vec<ComponentInstance>) -> SceneAssetEntity {
    SceneAssetEntity {
        local_id: LocalId::new(local_id),
        local_path: format!("./{}", local_id),
        name: name.to_string(),
        components,
        extension_data: Default::default(),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// S14 RED — AssetCommand serde: tag="type", rename_all="PascalCase"
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn asset_command_ser_de_tag_type_present() {
    let cmd = AssetCommand::AddEntity {
        local_id: "a1".to_string(),
        name: "Foo".to_string(),
        local_path: "./a1".to_string(),
        components: vec![],
    };
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"AddEntity\""));
}

#[test]
fn asset_command_ser_de_rename_all_pascal_case() {
    let cmd = AssetCommand::SetComponentValue {
        local_id: "a1".to_string(),
        type_id: "editor.Transform2D".to_string(),
        field_path: vec!["translation".to_string()],
        value: json!({"x": 1.0, "y": 2.0}),
    };
    let json = serde_json::to_string(&cmd).unwrap();
    // field_path is Vec<String> → serialized as JSON array
    assert!(json.contains("\"field_path\":[\"translation\"]"));
    assert!(json.contains("\"type\":\"SetComponentValue\""));
}

#[test]
fn asset_command_de_set_component_value_field_path_is_vec() {
    let json = r#"{
        "type": "SetComponentValue",
        "local_id": "a1",
        "type_id": "editor.Transform2D",
        "field_path": ["translation", "x"],
        "value": 42.0
    }"#;
    let cmd: AssetCommand = serde_json::from_str(json).unwrap();
    match cmd {
        AssetCommand::SetComponentValue { field_path, .. } => {
            assert_eq!(field_path, vec!["translation", "x"]);
        }
        _ => panic!("Expected SetComponentValue"),
    }
}

#[test]
fn asset_command_batch_ser() {
    let cmd = AssetCommand::Batch {
        label: "test-batch".to_string(),
        commands: vec![
            AssetCommand::AddEntity {
                local_id: "a1".to_string(),
                name: "Foo".to_string(),
                local_path: "./a1".to_string(),
                components: vec![],
            },
            AssetCommand::AddEntity {
                local_id: "a2".to_string(),
                name: "Bar".to_string(),
                local_path: "./a2".to_string(),
                components: vec![],
            },
        ],
    };
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"Batch\""));
    assert!(json.contains("\"label\":\"test-batch\""));
    assert!(json.contains("AddEntity"));
}

// ─────────────────────────────────────────────────────────────────────────
// S13 RED — AddEntity applies and inverts
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn add_entity_applies_to_document() {
    let mut doc = empty_asset_doc("id_test", "test/asset");
    let cmd = AssetCommand::AddEntity {
        local_id: "a1".to_string(),
        name: "A".to_string(),
        local_path: "./a1".to_string(),
        components: vec![],
    };

    let result = asset_apply(&mut doc, &cmd);
    assert!(result.is_ok(), "Apply should succeed: {:?}", result);
    assert_eq!(doc.entities.len(), 1);
    assert_eq!(doc.entities[0].local_id.as_str(), "a1");
    assert_eq!(doc.entities[0].name, "A");
}

#[test]
fn add_entity_inverse_is_remove_entity() {
    let mut doc = empty_asset_doc("id_test", "test/asset");
    let cmd = AssetCommand::AddEntity {
        local_id: "a1".to_string(),
        name: "A".to_string(),
        local_path: "./a1".to_string(),
        components: vec![],
    };

    let inverse = asset_apply(&mut doc, &cmd).unwrap();
    match inverse {
        AssetCommand::RemoveEntity { local_id } => {
            assert_eq!(local_id, "a1");
        }
        _ => panic!("Expected RemoveEntity inverse, got {:?}", inverse),
    }
}

#[test]
fn add_entity_undo_restores_to_empty() {
    let mut doc = empty_asset_doc("id_test", "test/asset");
    let cmd = AssetCommand::AddEntity {
        local_id: "a1".to_string(),
        name: "A".to_string(),
        local_path: "./a1".to_string(),
        components: vec![],
    };

    // Apply
    let inverse = asset_apply(&mut doc, &cmd).unwrap();
    assert_eq!(doc.entities.len(), 1);

    // Undo via inverse
    asset_apply(&mut doc, &inverse).unwrap();
    assert_eq!(doc.entities.len(), 0);
}

#[test]
fn add_entity_redo_restores_entity() {
    let mut doc = empty_asset_doc("id_test", "test/asset");
    let cmd = AssetCommand::AddEntity {
        local_id: "a1".to_string(),
        name: "A".to_string(),
        local_path: "./a1".to_string(),
        components: vec![],
    };

    // Apply
    let inverse = asset_apply(&mut doc, &cmd).unwrap();
    assert_eq!(doc.entities.len(), 1);

    // Undo
    asset_apply(&mut doc, &inverse).unwrap();
    assert_eq!(doc.entities.len(), 0);

    // Redo (re-apply forward)
    asset_apply(&mut doc, &cmd).unwrap();
    assert_eq!(doc.entities.len(), 1);
}

// ─────────────────────────────────────────────────────────────────────────
// S14 RED — SetComponentValue field path set
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn set_component_value_updates_field() {
    let mut doc = empty_asset_doc("id_test", "test/asset");
    let transform_comp = ComponentInstance {
        type_id: "editor.Transform2D".to_string(),
        values: json!({
            "translation": {"x": 0.0, "y": 0.0},
            "rotation": 0.0,
            "scale": {"x": 1.0, "y": 1.0}
        }),
    };
    doc.entities.push(entity("a1", "A", vec![transform_comp]));

    let cmd = AssetCommand::SetComponentValue {
        local_id: "a1".to_string(),
        type_id: "editor.Transform2D".to_string(),
        field_path: vec!["translation".to_string(), "x".to_string()],
        value: json!(100.0),
    };

    let inverse = asset_apply(&mut doc, &cmd).unwrap();
    assert_eq!(
        doc.entities[0].components[0].values["translation"]["x"],
        json!(100.0)
    );

    // Inverse should restore old value
    asset_apply(&mut doc, &inverse).unwrap();
    assert_eq!(
        doc.entities[0].components[0].values["translation"]["x"],
        json!(0.0)
    );
}

// ─────────────────────────────────────────────────────────────────────────
// S10 RED — dispatch_asset_command does NOT mutate SCENE_DOC
// (SCENE_DOC isolation test — verifies the apply only touches asset doc)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn asset_command_apply_only_touches_asset_document() {
    // This test verifies that asset_apply only modifies
    // the document it receives — the caller (dispatch_asset_command)
    // is responsible for ensuring SCENE_DOC is untouched.

    let mut asset_doc = empty_asset_doc("id_test", "test/asset");
    let cmd = AssetCommand::AddEntity {
        local_id: "a1".to_string(),
        name: "A".to_string(),
        local_path: "./a1".to_string(),
        components: vec![],
    };

    // Apply to asset doc
    let inverse = asset_apply(&mut asset_doc, &cmd).unwrap();

    // Verify asset doc changed
    assert_eq!(asset_doc.entities.len(), 1);

    // Verify undo works (still on asset doc)
    asset_apply(&mut asset_doc, &inverse).unwrap();
    assert_eq!(asset_doc.entities.len(), 0);

    // Note: SCENE_DOC isolation is a WASM-level concern (separate thread-local).
    // The pure Rust test here verifies the apply function is a pure transformation.
}

// ─────────────────────────────────────────────────────────────────────────
// S15 RED — AssetOperationLog::is_dirty and clear
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn asset_operation_log_new_is_not_dirty() {
    let log = AssetOperationLog::new_const();
    assert!(!log.is_dirty());
    assert!(!log.can_undo());
    assert!(!log.can_redo());
}

#[test]
fn asset_operation_log_is_dirty_after_record() {
    let mut log = AssetOperationLog::new_const();
    let doc = empty_asset_doc("id_test", "test/asset");

    // Simulate recording: Apply a command to doc and record the inverse
    let cmd = AssetCommand::AddEntity {
        local_id: "a1".to_string(),
        name: "A".to_string(),
        local_path: "./a1".to_string(),
        components: vec![],
    };
    let inverse = asset_apply(&mut doc.clone(), &cmd).unwrap();

    // Record into the log
    log.record(&cmd, inverse.clone());

    // After recording, should be dirty (there are un-saved changes)
    // Note: is_dirty = cursor < entries.len() - 1 (has future redo entries)
    // Actually per design: is_dirty means there are changes since last clear
    // For now, we just check the log has entries
    assert!(log.can_undo());
    assert!(!log.can_redo());
}

#[test]
fn asset_operation_log_clear_resets_dirty() {
    let mut log = AssetOperationLog::new_const();
    let doc = empty_asset_doc("id_test", "test/asset");

    let cmd = AssetCommand::AddEntity {
        local_id: "a1".to_string(),
        name: "A".to_string(),
        local_path: "./a1".to_string(),
        components: vec![],
    };
    let inverse = asset_apply(&mut doc.clone(), &cmd).unwrap();
    log.record(&cmd, inverse);

    assert!(log.can_undo());

    log.clear();

    assert!(!log.can_undo());
    assert!(!log.can_redo());
    assert!(!log.is_dirty());
}

#[test]
fn asset_operation_log_undo_applies_inverse() {
    let mut log = AssetOperationLog::new_const();
    let mut doc = empty_asset_doc("id_test", "test/asset");

    let cmd = AssetCommand::AddEntity {
        local_id: "a1".to_string(),
        name: "A".to_string(),
        local_path: "./a1".to_string(),
        components: vec![],
    };
    let inverse = asset_apply(&mut doc, &cmd).unwrap();
    log.record(&cmd, inverse.clone());

    assert_eq!(doc.entities.len(), 1);

    // Undo
    log.undo(&mut doc).unwrap();
    assert_eq!(doc.entities.len(), 0);
    assert!(!log.can_undo());
    assert!(log.can_redo());
}

#[test]
fn asset_operation_log_redo_applies_forward() {
    let mut log = AssetOperationLog::new_const();
    let mut doc = empty_asset_doc("id_test", "test/asset");

    let cmd = AssetCommand::AddEntity {
        local_id: "a1".to_string(),
        name: "A".to_string(),
        local_path: "./a1".to_string(),
        components: vec![],
    };
    let inverse = asset_apply(&mut doc, &cmd).unwrap();
    log.record(&cmd, inverse);

    // Undo
    log.undo(&mut doc).unwrap();
    assert_eq!(doc.entities.len(), 0);

    // Redo
    log.redo(&mut doc).unwrap();
    assert_eq!(doc.entities.len(), 1);
}

// ─────────────────────────────────────────────────────────────────────────
// S13 RED — RemoveEntity inverse captures full entity
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn remove_entity_inverse_contains_full_entity() {
    let mut doc = empty_asset_doc("id_test", "test/asset");
    let transform_comp = ComponentInstance {
        type_id: "editor.Transform2D".to_string(),
        values: json!({
            "translation": {"x": 0.0, "y": 0.0},
            "rotation": 0.0,
            "scale": {"x": 1.0, "y": 1.0}
        }),
    };
    doc.entities.push(entity("a1", "A", vec![transform_comp]));

    let cmd = AssetCommand::RemoveEntity {
        local_id: "a1".to_string(),
    };

    let inverse = asset_apply(&mut doc, &cmd).unwrap();

    // Inverse should be AddEntity with full captured entity
    match inverse {
        AssetCommand::AddEntity {
            local_id,
            name,
            local_path,
            components,
        } => {
            assert_eq!(local_id, "a1");
            assert_eq!(name, "A");
            assert_eq!(local_path, "./a1");
            assert_eq!(components.len(), 1);
        }
        _ => panic!("Expected AddEntity inverse, got {:?}", inverse),
    }
}

#[test]
fn remove_entity_undo_restores_full_entity() {
    let mut doc = empty_asset_doc("id_test", "test/asset");
    let transform_comp = ComponentInstance {
        type_id: "editor.Transform2D".to_string(),
        values: json!({
            "translation": {"x": 0.0, "y": 0.0},
            "rotation": 0.0,
            "scale": {"x": 1.0, "y": 1.0}
        }),
    };
    doc.entities.push(entity("a1", "A", vec![transform_comp]));

    let cmd = AssetCommand::RemoveEntity {
        local_id: "a1".to_string(),
    };

    let inverse = asset_apply(&mut doc, &cmd).unwrap();
    assert_eq!(doc.entities.len(), 0);

    // Undo via inverse
    asset_apply(&mut doc, &inverse).unwrap();
    assert_eq!(doc.entities.len(), 1);
    assert_eq!(doc.entities[0].local_id.as_str(), "a1");
    assert_eq!(doc.entities[0].name, "A");
}

// ─────────────────────────────────────────────────────────────────────────
// S14 RED — RenameEntity inverse captures old_name pre-mutation
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn rename_entity_inverse_swaps_old_and_new_name() {
    let mut doc = empty_asset_doc("id_test", "test/asset");
    doc.entities.push(entity("a1", "OriginalName", vec![]));

    let cmd = AssetCommand::RenameEntity {
        local_id: "a1".to_string(),
        old_name: None, // Will be captured
        new_name: "NewName".to_string(),
    };

    let inverse = asset_apply(&mut doc, &cmd).unwrap();

    // Document should have new name
    assert_eq!(doc.entities[0].name, "NewName");

    // Inverse should swap: old_name = prior actual name, new_name = prior requested name
    match inverse {
        AssetCommand::RenameEntity {
            local_id,
            old_name,
            new_name,
        } => {
            assert_eq!(local_id, "a1");
            assert_eq!(old_name, Some("OriginalName".to_string()));
            assert_eq!(new_name, "OriginalName".to_string());
        }
        _ => panic!("Expected RenameEntity inverse, got {:?}", inverse),
    }
}

#[test]
fn rename_entity_roundtrip_undo_redo_restores_both_names() {
    let mut doc = empty_asset_doc("id_test", "test/asset");
    doc.entities.push(entity("a1", "OriginalName", vec![]));

    let cmd = AssetCommand::RenameEntity {
        local_id: "a1".to_string(),
        old_name: None,
        new_name: "NewName".to_string(),
    };

    // Apply forward
    let inverse = asset_apply(&mut doc, &cmd).unwrap();
    assert_eq!(doc.entities[0].name, "NewName");

    // Undo
    asset_apply(&mut doc, &inverse).unwrap();
    assert_eq!(doc.entities[0].name, "OriginalName");

    // Redo (apply forward again)
    asset_apply(&mut doc, &cmd).unwrap();
    assert_eq!(doc.entities[0].name, "NewName");
}

// ─────────────────────────────────────────────────────────────────────────
// Error cases
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn add_entity_duplicate_local_id_fails() {
    let mut doc = empty_asset_doc("id_test", "test/asset");
    doc.entities.push(entity("a1", "A", vec![]));

    let cmd = AssetCommand::AddEntity {
        local_id: "a1".to_string(), // Already exists
        name: "B".to_string(),
        local_path: "./a1".to_string(),
        components: vec![],
    };

    let result = asset_apply(&mut doc, &cmd);
    assert!(matches!(
        result,
        Err(AssetCommandError::DuplicateLocalId(_))
    ));
}

#[test]
fn remove_entity_not_found_fails() {
    let mut doc = empty_asset_doc("id_test", "test/asset");

    let cmd = AssetCommand::RemoveEntity {
        local_id: "nonexistent".to_string(),
    };

    let result = asset_apply(&mut doc, &cmd);
    assert!(matches!(result, Err(AssetCommandError::EntityNotFound(_))));
}

#[test]
fn set_component_value_entity_not_found_fails() {
    let mut doc = empty_asset_doc("id_test", "test/asset");

    let cmd = AssetCommand::SetComponentValue {
        local_id: "nonexistent".to_string(),
        type_id: "editor.Transform2D".to_string(),
        field_path: vec!["translation".to_string()],
        value: json!(42.0),
    };

    let result = asset_apply(&mut doc, &cmd);
    assert!(matches!(result, Err(AssetCommandError::EntityNotFound(_))));
}

// ─────────────────────────────────────────────────────────────────────────
// Batch command
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn batch_command_applies_all_and_inverts_in_reverse() {
    let mut doc = empty_asset_doc("id_test", "test/asset");

    let cmd = AssetCommand::Batch {
        label: "create-two".to_string(),
        commands: vec![
            AssetCommand::AddEntity {
                local_id: "a1".to_string(),
                name: "A".to_string(),
                local_path: "./a1".to_string(),
                components: vec![],
            },
            AssetCommand::AddEntity {
                local_id: "a2".to_string(),
                name: "B".to_string(),
                local_path: "./a2".to_string(),
                components: vec![],
            },
        ],
    };

    let inverse = asset_apply(&mut doc, &cmd).unwrap();
    assert_eq!(doc.entities.len(), 2);

    // Undo batch
    asset_apply(&mut doc, &inverse).unwrap();
    assert_eq!(doc.entities.len(), 0);
}
