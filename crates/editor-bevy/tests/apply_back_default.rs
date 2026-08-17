//! Spec §7: apply-back-policy-default-never (MUST, D4).
//!
//! Verifies legacy v0.88 fixtures lacking `apply_back` deserialize to `Never`.

use editor_bevy::ApplyBackPolicy;
use editor_bevy::schema::ComponentSchema;
use serde_json::json;

#[test]
fn legacy_schema_without_apply_back_field_deserialises_to_never() {
    // A v0.88-era fixture that does NOT have the `apply_back` field.
    // Required fields: type_id, display_name, fields, exports_to_bevy.
    let legacy_fixture = json!({
        "type_id": "editor.Transform2D",
        "display_name": "Transform",
        "fields": [],
        "exports_to_bevy": true
    });
    let schema: ComponentSchema =
        serde_json::from_value(legacy_fixture).expect("legacy schema must deserialize");
    assert_eq!(
        schema.apply_back,
        ApplyBackPolicy::Never,
        "v0.88 fixture must default apply_back to Never"
    );
}

#[test]
fn explicit_apply_back_tunable_round_trips() {
    let fixture = json!({
        "type_id": "editor.Health",
        "display_name": "Health",
        "fields": [],
        "exports_to_bevy": true,
        "apply_back": "tunable",
    });
    let schema: ComponentSchema = serde_json::from_value(fixture).unwrap();
    assert_eq!(schema.apply_back, ApplyBackPolicy::Tunable);
    // Round-trip
    let json = serde_json::to_value(&schema).unwrap();
    assert_eq!(json["apply_back"], "tunable");
}
