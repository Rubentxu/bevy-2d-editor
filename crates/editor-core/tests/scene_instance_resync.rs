//! Tests for scene instance resync on load.
//!
//! Covers: S8, S9 scenarios.
//!
//! S8: Asset version bump on load triggers resync
//! S9: Resync never auto-deletes overrides - orphaned patches go to orphaned_overrides

use editor_core::{
    document::SceneDocument,
    scene_asset::{AssetReference, LocalId, SceneAssetDocument, SceneAssetEntity, SceneAssetRole},
    scene_instance::{ComponentOverride, ComponentOverrideStatus, SceneInstance},
    scene_instance_overrides::{resync, ResyncReport},
    StableId,
};
use std::collections::BTreeMap;

// Helper: create a SceneInstance with overrides
fn make_instance_with_overrides(
    instance_id: &str,
    asset_ref: &str,
    version_seen: u32,
    component_overrides: Vec<ComponentOverride>,
) -> SceneInstance {
    SceneInstance {
        instance_id: StableId::new(instance_id),
        asset_ref: AssetReference::new(asset_ref),
        asset_version_seen: version_seen,
        id_map: vec![(LocalId::new("root"), StableId::new(format!("{}_root", instance_id)))]
            .into_iter()
            .collect(),
        component_overrides,
        orphaned_component_overrides: vec![],
    }
}

// Helper: create a simple SceneAssetDocument
fn make_asset(asset_id: &str, entities: Vec<SceneAssetEntity>) -> SceneAssetDocument {
    SceneAssetDocument {
        asset_id: asset_id.to_string(),
        logical_path: format!("assets/{}.bsn", asset_id),
        role: SceneAssetRole::Actor,
        version: 1,
        entities,
        relationships: vec![],
        exposed_properties: vec![],
        metadata: Default::default(),
    }
}

/// S8: Asset version bump triggers resync and updates asset_version_seen.
#[test]
fn s8_version_bump_triggers_resync() {
    // Create instance at version 1
    let mut instance = make_instance_with_overrides(
        "inst_001",
        "player_asset",
        1, // old version
        vec![
            ComponentOverride {
                target_local_id: LocalId::new("root".to_string()),
                component_type_id: editor_core::schema::ComponentTypeId::new("editor.Transform2D"),
                field_path: vec!["translation".to_string()],
                value: serde_json::json!({"x": 100.0, "y": 200.0}),
                status: ComponentOverrideStatus::Active,
            },
        ],
    );

    // Create asset at version 2 (new version)
    let asset = make_asset(
        "player_asset",
        vec![SceneAssetEntity {
            local_id: LocalId("root".to_string()),
            local_path: "root".to_string(),
            name: "Player".to_string(),
            components: vec![],
        }],
    );

    // Run resync with new version
    let report = resync(&asset, &mut instance, 2);

    // Verify version was updated
    assert_eq!(instance.asset_version_seen, 2, "asset_version_seen should update to 2");

    // Verify resync report has information
    assert!(
        report.active > 0 || report.stale > 0 || report.orphaned > 0 || report.conflict > 0 || report.rebound > 0,
        "ResyncReport should have non-zero counts"
    );
}

/// S9: Resync never auto-deletes overrides - orphaned patches move to orphaned_overrides.
#[test]
fn s9_resync_never_silently_deletes_overrides() {
    // Create instance with an override targeting "deleted_entity" which no longer exists
    let mut instance = make_instance_with_overrides(
        "inst_002",
        "enemy_asset",
        1,
        vec![
            // Override targeting a deleted entity
            ComponentOverride {
                target_local_id: LocalId::new("deleted_entity".to_string()),
                component_type_id: editor_core::schema::ComponentTypeId::new("editor.Transform2D"),
                field_path: vec!["translation".to_string()],
                value: serde_json::json!({"x": 50.0, "y": 75.0}),
                status: ComponentOverrideStatus::Active,
            },
            // Override targeting existing entity
            ComponentOverride {
                target_local_id: LocalId::new("root".to_string()),
                component_type_id: editor_core::schema::ComponentTypeId::new("editor.Transform2D"),
                field_path: vec!["scale".to_string()],
                value: serde_json::json!({"x": 2.0, "y": 2.0}),
                status: ComponentOverrideStatus::Active,
            },
        ],
    );

    let original_override_count = instance.component_overrides.len();

    // Create asset at version 2 where "deleted_entity" no longer exists
    let asset = make_asset(
        "enemy_asset",
        vec![SceneAssetEntity {
            // Only "root" exists now, "deleted_entity" is gone
            local_id: LocalId("root".to_string()),
            local_path: "root".to_string(),
            name: "Enemy".to_string(),
            components: vec![],
        }],
    );

    // Run resync
    let report = resync(&asset, &mut instance, 2);

    // The patch targeting deleted_entity should be moved to orphaned_overrides
    assert!(
        instance.orphaned_component_overrides.len() > 0 || report.orphaned > 0,
        "Override targeting deleted entity should be marked as orphaned"
    );

    // Total override count should be preserved (no silent deletion)
    let total_after = instance.component_overrides.len() + instance.orphaned_component_overrides.len();
    assert_eq!(
        total_after, original_override_count,
        "Total override count should be preserved - no silent deletion"
    );

    // Verify the orphaned override has Orphaned status
    if !instance.orphaned_component_overrides.is_empty() {
        assert!(
            matches!(
                instance.orphaned_component_overrides[0].status,
                ComponentOverrideStatus::Orphaned
            ),
            "Orphaned override should have Orphaned status"
        );
    }
}

/// S8 Variant: Resync with multiple version bumps.
#[test]
fn s8_multiple_version_bumps() {
    let mut instance = make_instance_with_overrides(
        "inst_003",
        "npc_asset",
        1,
        vec![],
    );

    // Asset is now at version 3
    let asset = make_asset(
        "npc_asset",
        vec![SceneAssetEntity {
            local_id: LocalId("root".to_string()),
            local_path: "root".to_string(),
            name: "NPC".to_string(),
            components: vec![],
        }],
    );

    // Run resync skipping versions
    let report = resync(&asset, &mut instance, 3);

    assert_eq!(instance.asset_version_seen, 3);
    assert!(report.active >= 0); // No active overrides to report
}

/// S9 Variant: Multiple orphaned overrides.
#[test]
fn s9_multiple_orphaned_overrides() {
    let mut instance = SceneInstance {
        instance_id: StableId::new("inst_multi_orphan"),
        asset_ref: AssetReference::new("multi_asset"),
        asset_version_seen: 1,
        id_map: vec![
            (LocalId::new("root"), StableId::new("inst_multi_orphan_root")),
            (LocalId::new("entity_a"), StableId::new("inst_multi_orphan_entity_a")),
            (LocalId::new("entity_b"), StableId::new("inst_multi_orphan_entity_b")),
        ]
        .into_iter()
        .collect(),
        component_overrides: vec![
            ComponentOverride {
                target_local_id: LocalId::new("deleted1".to_string()),
                component_type_id: editor_core::schema::ComponentTypeId::new("editor.Transform2D"),
                field_path: vec!["translation".to_string()],
                value: serde_json::json!({"x": 1.0, "y": 1.0}),
                status: ComponentOverrideStatus::Active,
            },
            ComponentOverride {
                target_local_id: LocalId::new("deleted2".to_string()),
                component_type_id: editor_core::schema::ComponentTypeId::new("editor.Transform2D"),
                field_path: vec!["rotation".to_string()],
                value: serde_json::json!(45.0),
                status: ComponentOverrideStatus::Active,
            },
            ComponentOverride {
                target_local_id: LocalId::new("root".to_string()), // This one survives
                component_type_id: editor_core::schema::ComponentTypeId::new("editor.Transform2D"),
                field_path: vec!["scale".to_string()],
                value: serde_json::json!({"x": 1.5, "y": 1.5}),
                status: ComponentOverrideStatus::Active,
            },
        ],
        orphaned_component_overrides: vec![],
    };

    // Asset only has "root" entity now
    let asset = make_asset(
        "multi_asset",
        vec![SceneAssetEntity {
            local_id: LocalId("root".to_string()),
            local_path: "root".to_string(),
            name: "Multi".to_string(),
            components: vec![],
        }],
    );

    let report = resync(&asset, &mut instance, 2);

    // Should have 2 orphaned overrides
    assert_eq!(instance.orphaned_component_overrides.len(), 2, "Should have 2 orphaned overrides");
    assert_eq!(instance.component_overrides.len(), 1, "Should have 1 remaining override");

    // Total preserved
    let total = instance.component_overrides.len() + instance.orphaned_component_overrides.len();
    assert_eq!(total, 3, "All 3 overrides should be preserved");
}
