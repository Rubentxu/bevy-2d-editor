//! Spec §9 NFR-1: RuntimeDelta MUST NOT contain Bevy entity identifiers.
//!
//! Verifies that `RuntimeDelta` has no `bevy_entity_id` or similar leak field.

use editor_application::runtime_delta::RuntimeDelta;
use serde_json::json;

#[test]
fn runtime_delta_contains_no_bevy_entity_fields() {
    let delta = RuntimeDelta {
        instance_id: "player-entity".to_string(),
        target_local_id: "root".to_string(),
        component_type_id: "editor.Transform2D".to_string(),
        field_path: "translation.x".to_string(),
        baseline_value: json!(1.0),
        runtime_value: json!(2.0),
        captured_at_ms: 1_700_000_000_000_u64,
        apply_back_eligible: true,
    };

    let json = serde_json::to_value(&delta).unwrap();

    // NFR-1: Must not contain any Bevy Entity ID fields.
    // These would leak runtime-only Bevy identifiers to the editor model.
    assert!(
        !json.as_object().unwrap().contains_key("bevy_entity"),
        "RuntimeDelta must NOT contain bevy_entity field"
    );
    assert!(
        !json.as_object().unwrap().contains_key("entity_id"),
        "RuntimeDelta must NOT contain entity_id field"
    );
    assert!(
        !json.as_object().unwrap().contains_key("bevy_entity_id"),
        "RuntimeDelta must NOT contain bevy_entity_id field"
    );
    // StableId-based fields are editor-owned — those are allowed.
    assert!(json.as_object().unwrap().contains_key("instance_id"));
    assert!(json.as_object().unwrap().contains_key("target_local_id"));
    assert!(json.as_object().unwrap().contains_key("component_type_id"));
    assert!(json.as_object().unwrap().contains_key("field_path"));
}

#[test]
fn runtime_delta_roundtrips_through_json() {
    let delta = RuntimeDelta {
        instance_id: "enemy-spawn".to_string(),
        target_local_id: "body".to_string(),
        component_type_id: "editor.Health".to_string(),
        field_path: "current_hp".to_string(),
        baseline_value: json!(100),
        runtime_value: json!(65),
        captured_at_ms: 1_700_000_000_000_u64,
        apply_back_eligible: true,
    };

    let json = serde_json::to_value(&delta).unwrap();
    let roundtrip: RuntimeDelta = serde_json::from_value(json).unwrap();

    assert_eq!(roundtrip.instance_id, delta.instance_id);
    assert_eq!(roundtrip.target_local_id, delta.target_local_id);
    assert_eq!(roundtrip.component_type_id, delta.component_type_id);
    assert_eq!(roundtrip.field_path, delta.field_path);
    assert_eq!(roundtrip.baseline_value, delta.baseline_value);
    assert_eq!(roundtrip.runtime_value, delta.runtime_value);
    assert_eq!(roundtrip.apply_back_eligible, delta.apply_back_eligible);
}
