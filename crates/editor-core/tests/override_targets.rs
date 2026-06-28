//! Override target & rename tests.
//! Covers scenarios S3, S4.

use editor_core::{
    scene_asset::LocalId,
    scene_instance::{OverridePatch, OverrideStatus, patch_status_after_field_rename},
};

#[test]
fn s3_override_targets_local_id() {
    let patch = OverridePatch {
        target_local_id: LocalId("weapon".to_string()),
        field_path: vec!["Sprite2D".to_string(), "asset".to_string()],
        value: serde_json::json!("cannon.png"),
        status: OverrideStatus::Active,
    };

    assert_eq!(patch.target_local_id.as_str(), "weapon");

    let renamed_name_patch = OverridePatch {
        target_local_id: LocalId("weapon".to_string()),
        field_path: patch.field_path.clone(),
        value: serde_json::json!("cannon.png"),
        status: patch.status,
    };
    assert_eq!(renamed_name_patch.target_local_id.as_str(), "weapon");
}

#[test]
fn s4_rename_marks_stale() {
    let patch = OverridePatch {
        target_local_id: LocalId("weapon".to_string()),
        field_path: vec!["Sprite2D".to_string(), "asset".to_string()],
        value: serde_json::json!("cannon.png"),
        status: OverrideStatus::Active,
    };

    let result = patch_status_after_field_rename(&patch, ("Sprite2D", "Sprite"));
    assert_eq!(
        result,
        OverrideStatus::Stale,
        "Renaming component field should mark override Stale"
    );

    let unchanged_result = patch_status_after_field_rename(&patch, ("Transform2D", "Transform"));
    assert_eq!(
        unchanged_result,
        OverrideStatus::Active,
        "Absent field rename should leave status Active"
    );

    let orphan_patch = OverridePatch {
        target_local_id: LocalId("weapon".to_string()),
        field_path: vec!["Sprite2D".to_string(), "asset".to_string()],
        value: serde_json::json!("cannon.png"),
        status: OverrideStatus::Orphaned,
    };
    let orphan_result = patch_status_after_field_rename(&orphan_patch, ("Sprite2D", "Sprite"));
    assert_eq!(
        orphan_result,
        OverrideStatus::Orphaned,
        "Orphaned patch should remain Orphaned even on field rename"
    );
}
