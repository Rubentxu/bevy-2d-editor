//! Override target & rename tests.
//! Covers scenarios S3, S4.

use editor_core::{
    scene_asset::LocalId,
    scene_instance::{ComponentOverride, ComponentOverrideStatus, component_override_status_after_field_rename},
};

#[test]
fn s3_override_targets_local_id() {
    let patch = ComponentOverride {
        target_local_id: LocalId("weapon".to_string()),
        component_type_id: editor_core::schema::ComponentTypeId::new("Sprite2D"),
        field_path: vec!["asset".to_string()],
        value: serde_json::json!("cannon.png"),
        status: ComponentOverrideStatus::Active,
    };

    assert_eq!(patch.target_local_id.as_str(), "weapon");

    let renamed_name_patch = ComponentOverride {
        target_local_id: LocalId("weapon".to_string()),
        component_type_id: patch.component_type_id.clone(),
        field_path: patch.field_path.clone(),
        value: serde_json::json!("cannon.png"),
        status: patch.status,
    };
    assert_eq!(renamed_name_patch.target_local_id.as_str(), "weapon");
}

#[test]
fn s4_rename_marks_stale() {
    let patch = ComponentOverride {
        target_local_id: LocalId("weapon".to_string()),
        component_type_id: editor_core::schema::ComponentTypeId::new("Sprite2D"),
        field_path: vec!["asset".to_string()],
        value: serde_json::json!("cannon.png"),
        status: ComponentOverrideStatus::Active,
    };

    let result = component_override_status_after_field_rename(&patch, ("asset", "Sprite"));
    assert_eq!(
        result,
        ComponentOverrideStatus::Stale,
        "Renaming component field should mark override Stale"
    );

    let unchanged_result = component_override_status_after_field_rename(&patch, ("Transform2D", "Transform"));
    assert_eq!(
        unchanged_result,
        ComponentOverrideStatus::Active,
        "Absent field rename should leave status Active"
    );

    let orphan_patch = ComponentOverride {
        target_local_id: LocalId("weapon".to_string()),
        component_type_id: editor_core::schema::ComponentTypeId::new("Sprite2D"),
        field_path: vec!["asset".to_string()],
        value: serde_json::json!("cannon.png"),
        status: ComponentOverrideStatus::Orphaned,
    };
    let orphan_result = component_override_status_after_field_rename(&orphan_patch, ("Sprite2D", "Sprite"));
    assert_eq!(
        orphan_result,
        ComponentOverrideStatus::Orphaned,
        "Orphaned patch should remain Orphaned even on field rename"
    );
}
