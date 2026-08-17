//! Integration tests for scene instance overrides per ADR-0005 §Overrides/§Versioning.

use std::collections::BTreeMap;

use editor_bevy::StableId;
use editor_bevy::scene_asset::{
    AssetReference, LocalId, SceneAssetDocument, SceneAssetEntity, SceneAssetRole,
};
use editor_bevy::scene_instance::{ComponentOverride, ComponentOverrideStatus, SceneInstance};
use editor_bevy::scene_instance_overrides::{
    classify_overrides, effective_values, mint_id_map, reconcile_id_map, resync, try_rebind,
    validate_overrides,
};
use editor_model::ComponentInstance;

// ---------------------------------------------------------------------------
// Test Helpers
// ---------------------------------------------------------------------------

fn make_asset(entities: Vec<SceneAssetEntity>, version: u32) -> SceneAssetDocument {
    SceneAssetDocument {
        layers: vec![],
        asset_id: "asset_test".to_string(),
        logical_path: "assets/test".to_string(),
        role: SceneAssetRole::Actor,
        version,
        entities,
        relationships: vec![],
        exposed_properties: vec![],
        metadata: Default::default(),
    }
}

fn make_instance(
    component_overrides: Vec<ComponentOverride>,
    orphaned_component_overrides: Vec<ComponentOverride>,
    id_map: BTreeMap<LocalId, StableId>,
    asset_version_seen: u32,
) -> SceneInstance {
    SceneInstance {
        instance_components: vec![],

        instance_id: StableId::new("inst_1"),
        asset_ref: AssetReference::new("assets/test"),
        asset_version_seen,
        id_map,
        component_overrides,
        orphaned_component_overrides,
    }
}

// ---------------------------------------------------------------------------
// S1: classify_overrides — full type_id segment-0 → Active
// ---------------------------------------------------------------------------

#[test]
fn classify_overrides_namespaced_active() {
    let asset = make_asset(
        vec![SceneAssetEntity {
            local_id: LocalId::new("root"),
            local_path: "root".to_string(),
            name: "Root".to_string(),
            components: vec![ComponentInstance {
                type_id: "editor.Sprite2D".to_string(),
                values: serde_json::json!({"asset": "player.png"}),
            }],
        }],
        1,
    );

    let patch = ComponentOverride {
        target_local_id: LocalId::new("root"),
        component_type_id: editor_bevy::schema::ComponentTypeId::new("editor.Sprite2D"),
        field_path: vec!["asset".to_string()],
        value: serde_json::Value::String("cannon.png".to_string()),
        status: ComponentOverrideStatus::Active,
    };

    let result = classify_overrides(&asset, std::slice::from_ref(&patch));
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].status, ComponentOverrideStatus::Active);
}

// ---------------------------------------------------------------------------
// S2: classify_overrides — short form does NOT match (Orphaned)
// ---------------------------------------------------------------------------

#[test]
fn classify_overrides_short_form_orphans() {
    let asset = make_asset(
        vec![SceneAssetEntity {
            local_id: LocalId::new("root"),
            local_path: "root".to_string(),
            name: "Root".to_string(),
            components: vec![ComponentInstance {
                type_id: "editor.Sprite2D".to_string(),
                values: serde_json::json!({"asset": "player.png"}),
            }],
        }],
        1,
    );

    // Short form: "Sprite2D" instead of "editor.Sprite2D"
    let patch = ComponentOverride {
        target_local_id: LocalId::new("root"),
        component_type_id: editor_bevy::schema::ComponentTypeId::new("Sprite2D"),
        field_path: vec!["asset".to_string()],
        value: serde_json::Value::String("cannon.png".to_string()),
        status: ComponentOverrideStatus::Active,
    };

    let result = classify_overrides(&asset, std::slice::from_ref(&patch));
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].status, ComponentOverrideStatus::Orphaned);
}

// ---------------------------------------------------------------------------
// S3 + S4: resync — entity rename preserves override (Active)
// ---------------------------------------------------------------------------

#[test]
fn resync_preserves_override_on_rename() {
    // Asset v2: same local_id "abc" but name changed from "Weapon" to "Cannon"
    let asset = make_asset(
        vec![SceneAssetEntity {
            local_id: LocalId::new("abc"),
            local_path: "abc".to_string(),
            name: "Cannon".to_string(),
            components: vec![ComponentInstance {
                type_id: "editor.Sprite2D".to_string(),
                values: serde_json::json!({"asset": "player.png"}),
            }],
        }],
        2,
    );

    let id_map: BTreeMap<LocalId, StableId> = vec![(LocalId::new("abc"), StableId::new("ent_a"))]
        .into_iter()
        .collect();

    let instance = make_instance(
        vec![ComponentOverride {
            target_local_id: LocalId::new("abc"),
            component_type_id: editor_bevy::schema::ComponentTypeId::new("editor.Sprite2D"),
            field_path: vec!["asset".to_string()],
            value: serde_json::Value::String("cannon.png".to_string()),
            status: ComponentOverrideStatus::Active,
        }],
        vec![],
        id_map,
        1,
    );

    let mut instance = instance;
    let report = resync(&asset, &mut instance, 2);

    assert_eq!(report.active, 1);
    assert_eq!(report.orphaned, 0);
    assert_eq!(report.stale, 0);
    assert_eq!(report.conflict, 0);
    assert_eq!(report.rebound, 0);
    assert_eq!(instance.asset_version_seen, 2);
    assert_eq!(instance.component_overrides.len(), 1);
    assert_eq!(
        instance.component_overrides[0].status,
        ComponentOverrideStatus::Active
    );
}

// ---------------------------------------------------------------------------
// S5: resync — entity removed routes patch to orphaned_overrides
// ---------------------------------------------------------------------------

#[test]
fn resync_moves_to_orphaned_on_entity_removed() {
    let asset_v2 = make_asset(vec![], 2);

    let instance = make_instance(
        vec![ComponentOverride {
            target_local_id: LocalId::new("abc"),
            component_type_id: editor_bevy::schema::ComponentTypeId::new("editor.Sprite2D"),
            field_path: vec!["asset".to_string()],
            value: serde_json::Value::String("cannon.png".to_string()),
            status: ComponentOverrideStatus::Active,
        }],
        vec![],
        BTreeMap::new(),
        1,
    );

    let mut instance = instance;
    let report = resync(&asset_v2, &mut instance, 2);

    assert_eq!(report.active, 0);
    assert_eq!(report.orphaned, 1);
    assert_eq!(instance.component_overrides.len(), 0);
    assert_eq!(instance.orphaned_component_overrides.len(), 1);
    assert_eq!(
        instance.orphaned_component_overrides[0].status,
        ComponentOverrideStatus::Orphaned
    );
    assert_eq!(
        instance.orphaned_component_overrides[0].target_local_id,
        LocalId::new("abc")
    );
}

// ---------------------------------------------------------------------------
// S6: resync — field renamed marks patch Stale
// ---------------------------------------------------------------------------

#[test]
fn resync_marks_stale_on_field_rename() {
    // Asset v1: has "asset" field
    let asset_v1 = make_asset(
        vec![SceneAssetEntity {
            local_id: LocalId::new("root"),
            local_path: "root".to_string(),
            name: "Root".to_string(),
            components: vec![ComponentInstance {
                type_id: "editor.Sprite2D".to_string(),
                values: serde_json::json!({"asset": "player.png"}),
            }],
        }],
        1,
    );

    let instance = make_instance(
        vec![ComponentOverride {
            target_local_id: LocalId::new("root"),
            component_type_id: editor_bevy::schema::ComponentTypeId::new("editor.Sprite2D"),
            field_path: vec!["asset".to_string()],
            value: serde_json::Value::String("cannon.png".to_string()),
            status: ComponentOverrideStatus::Active,
        }],
        vec![],
        BTreeMap::new(),
        1,
    );

    let mut instance = instance;

    // Asset v2: "asset" renamed to "image"
    let asset_v2 = make_asset(
        vec![SceneAssetEntity {
            local_id: LocalId::new("root"),
            local_path: "root".to_string(),
            name: "Root".to_string(),
            components: vec![ComponentInstance {
                type_id: "editor.Sprite2D".to_string(),
                values: serde_json::json!({"image": "player.png"}),
            }],
        }],
        2,
    );

    let report = resync(&asset_v2, &mut instance, 2);

    assert_eq!(report.stale, 1);
    assert_eq!(report.active, 0);
    assert_eq!(instance.component_overrides.len(), 1);
    assert_eq!(
        instance.component_overrides[0].status,
        ComponentOverrideStatus::Stale
    );
}

// ---------------------------------------------------------------------------
// S7: resync — type change marks patch Conflict
// ---------------------------------------------------------------------------

#[test]
fn resync_marks_conflict_on_type_change() {
    let asset_v2 = make_asset(
        vec![SceneAssetEntity {
            local_id: LocalId::new("player"),
            local_path: "player".to_string(),
            name: "Player".to_string(),
            components: vec![ComponentInstance {
                type_id: "editor.Health".to_string(),
                values: serde_json::json!({"current": "full"}),
            }],
        }],
        2,
    );

    let instance = make_instance(
        vec![ComponentOverride {
            target_local_id: LocalId::new("player"),
            component_type_id: editor_bevy::schema::ComponentTypeId::new("editor.Health"),
            field_path: vec!["current".to_string()],
            value: serde_json::json!(42),
            status: ComponentOverrideStatus::Active,
        }],
        vec![],
        BTreeMap::new(),
        1,
    );

    let mut instance = instance;
    let report = resync(&asset_v2, &mut instance, 2);

    assert_eq!(report.conflict, 1);
    assert_eq!(instance.component_overrides.len(), 1);
    assert_eq!(
        instance.component_overrides[0].status,
        ComponentOverrideStatus::Conflict
    );
}

// ---------------------------------------------------------------------------
// S8: resync — orphaned rebind via local_id
// ---------------------------------------------------------------------------

#[test]
fn resync_rebinds_via_local_path() {
    // Asset v2: entity removed
    let asset_v2 = make_asset(vec![], 2);

    let id_map: BTreeMap<LocalId, StableId> = vec![(LocalId::new("abc"), StableId::new("ent_a"))]
        .into_iter()
        .collect();

    let instance = make_instance(
        vec![ComponentOverride {
            target_local_id: LocalId::new("abc"),
            component_type_id: editor_bevy::schema::ComponentTypeId::new("editor.Sprite2D"),
            field_path: vec!["asset".to_string()],
            value: serde_json::Value::String("cannon.png".to_string()),
            status: ComponentOverrideStatus::Active,
        }],
        vec![],
        id_map,
        1,
    );

    let mut instance = instance;
    resync(&asset_v2, &mut instance, 2);
    assert_eq!(instance.orphaned_component_overrides.len(), 1);

    // Asset v3: same local_id reappears
    let asset_v3 = make_asset(
        vec![SceneAssetEntity {
            local_id: LocalId::new("abc"),
            local_path: "root/player/weapons/cannon".to_string(),
            name: "Cannon".to_string(),
            components: vec![ComponentInstance {
                type_id: "editor.Sprite2D".to_string(),
                values: serde_json::json!({"asset": "player.png"}),
            }],
        }],
        3,
    );

    let report = resync(&asset_v3, &mut instance, 3);

    assert_eq!(report.rebound, 1);
    assert_eq!(instance.component_overrides.len(), 1);
    assert_eq!(
        instance.component_overrides[0].status,
        ComponentOverrideStatus::Active
    );
    assert_eq!(
        instance.component_overrides[0].target_local_id,
        LocalId::new("abc")
    );
}

// ---------------------------------------------------------------------------
// S9: effective_values — no overrides returns asset unchanged
// ---------------------------------------------------------------------------

#[test]
fn effective_values_with_no_overrides_returns_asset_unchanged() {
    let asset = make_asset(
        vec![
            SceneAssetEntity {
                local_id: LocalId::new("a"),
                local_path: "a".to_string(),
                name: "A".to_string(),
                components: vec![ComponentInstance {
                    type_id: "editor.Sprite2D".to_string(),
                    values: serde_json::json!({"asset": "a.png"}),
                }],
            },
            SceneAssetEntity {
                local_id: LocalId::new("b"),
                local_path: "b".to_string(),
                name: "B".to_string(),
                components: vec![ComponentInstance {
                    type_id: "editor.Sprite2D".to_string(),
                    values: serde_json::json!({"asset": "b.png"}),
                }],
            },
        ],
        1,
    );

    let instance = make_instance(vec![], vec![], BTreeMap::new(), 1);

    let resolved = {
        let mut counter = 0u32;
        let mut mint = move || {
            counter += 1;
            StableId::new(format!("sid_{}", counter))
        };
        effective_values(&asset, &instance, &mut mint).unwrap()
    };

    assert_eq!(resolved.entities.len(), 2);
    assert!(resolved.unresolved.is_empty());
    assert_eq!(resolved.id_map.len(), 2);
}

// ---------------------------------------------------------------------------
// S10: resync — id_map extends when asset gains a new entity
// ---------------------------------------------------------------------------

#[test]
fn resync_extends_id_map_on_new_entity() {
    // Asset v1: 2 entities
    let asset_v1 = make_asset(
        vec![
            SceneAssetEntity {
                local_id: LocalId::new("a"),
                local_path: "a".to_string(),
                name: "A".to_string(),
                components: vec![],
            },
            SceneAssetEntity {
                local_id: LocalId::new("b"),
                local_path: "b".to_string(),
                name: "B".to_string(),
                components: vec![],
            },
        ],
        1,
    );

    let id_map: BTreeMap<LocalId, StableId> = vec![
        (LocalId::new("a"), StableId::new("sid_0")),
        (LocalId::new("b"), StableId::new("sid_1")),
    ]
    .into_iter()
    .collect();

    let instance = make_instance(vec![], vec![], id_map, 1);
    let mut instance = instance;

    // Asset v2: 3 entities (added "c")
    let asset_v2 = make_asset(
        vec![
            SceneAssetEntity {
                local_id: LocalId::new("a"),
                local_path: "a".to_string(),
                name: "A".to_string(),
                components: vec![],
            },
            SceneAssetEntity {
                local_id: LocalId::new("b"),
                local_path: "b".to_string(),
                name: "B".to_string(),
                components: vec![],
            },
            SceneAssetEntity {
                local_id: LocalId::new("c"),
                local_path: "c".to_string(),
                name: "C".to_string(),
                components: vec![],
            },
        ],
        2,
    );

    resync(&asset_v2, &mut instance, 2);

    assert_eq!(instance.id_map.len(), 3);
    assert_eq!(
        instance.id_map.get(&LocalId::new("a")).map(|s| s.as_str()),
        Some("sid_0")
    );
    assert_eq!(
        instance.id_map.get(&LocalId::new("b")).map(|s| s.as_str()),
        Some("sid_1")
    );
    assert!(instance.id_map.contains_key(&LocalId::new("c")));
}

// ---------------------------------------------------------------------------
// Additional: validate_overrides returns issues for each failure
// ---------------------------------------------------------------------------

#[test]
fn validate_overrides_returns_issues_for_each_failure() {
    let asset = make_asset(
        vec![SceneAssetEntity {
            local_id: LocalId::new("root"),
            local_path: "root".to_string(),
            name: "Root".to_string(),
            components: vec![ComponentInstance {
                type_id: "editor.Sprite2D".to_string(),
                values: serde_json::json!({"asset": "player.png"}),
            }],
        }],
        1,
    );

    // Synthetic doc with 3 issues:
    // 1. missing_entity (target doesn't exist)
    // 2. missing_component (field_path[0] doesn't match)
    // 3. type_conflict (kind mismatch)
    let instance = make_instance(
        vec![
            // Issue 2: missing_component (short form)
            ComponentOverride {
                target_local_id: LocalId::new("root"),
                component_type_id: editor_bevy::schema::ComponentTypeId::new("WrongComponent"),
                field_path: vec!["field".to_string()],
                value: serde_json::Value::Null,
                status: ComponentOverrideStatus::Active,
            },
            // Issue 3: type_conflict
            ComponentOverride {
                target_local_id: LocalId::new("root"),
                component_type_id: editor_bevy::schema::ComponentTypeId::new("editor.Sprite2D"),
                field_path: vec!["asset".to_string()],
                value: serde_json::json!(123),
                status: ComponentOverrideStatus::Active,
            },
        ],
        vec![
            // Issue 1: missing_entity
            ComponentOverride {
                target_local_id: LocalId::new("nonexistent"),
                component_type_id: editor_bevy::schema::ComponentTypeId::new("editor.Sprite2D"),
                field_path: vec!["asset".to_string()],
                value: serde_json::Value::String("cannon.png".to_string()),
                status: ComponentOverrideStatus::Orphaned,
            },
        ],
        BTreeMap::new(),
        1,
    );

    let issues = validate_overrides(&asset, &instance);
    assert_eq!(issues.len(), 3);

    let codes: Vec<&str> = issues.iter().map(|i| i.code.as_str()).collect();
    assert!(codes.contains(&"missing_entity"));
    assert!(codes.contains(&"missing_component"));
    assert!(codes.contains(&"type_conflict"));
}
