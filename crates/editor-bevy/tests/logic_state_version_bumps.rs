//! Regression test: apply_set_field_override bumps binding_version and sets dirty.
//!
//! R3: when apply_set_field_override commits, binding_version increments and dirty=true.
//! This verifies the registry-level state is updated correctly.

use editor_bevy::with_binding_registry_for_tests;
use editor_bevy::logic_command::BindingId;
use editor_model::ids::StableId;
use std::collections::BTreeMap;

// Test helper: create a binding record in the registry for testing.
fn create_test_binding(binding_id: BindingId, scene_instance_id: &str) {
    use editor_bevy::with_binding_registry_mut_for_tests;

    let record = editor_bevy::state::BindingRecord {
        binding_id,
        recipe_id: "test_recipe".to_string(),
        version: 1,
        field_overrides: BTreeMap::new(),
        dirty: false,
        binding_version: 0,
    };

    with_binding_registry_mut_for_tests(|reg| {
        reg.insert(StableId::new(scene_instance_id.to_string()), record);
    });
}

#[test]
fn apply_set_field_override_bumps_version_and_sets_dirty() {
    // Create a binding with binding_version=0, dirty=false
    let binding_id = BindingId::new("test_binding_1".to_string());
    create_test_binding(binding_id.clone(), "instance_1");

    // Apply a field override
    let field_path = "jump_impulse".to_string();
    let value = serde_json::json!(10.0);
    let result =
        editor_bevy::logic_state::apply_set_binding_field_override(binding_id.clone(), field_path, value);

    // Verify the operation succeeded
    assert!(
        result.is_ok(),
        "apply_set_field_override must succeed: {:?}",
        result
    );

    // Verify the registry has dirty=true and binding_version > 0
    let mut check_passed = false;
    with_binding_registry_for_tests(|reg| {
        for (_sid, record) in reg.iter() {
            if record.binding_id == binding_id {
                assert!(
                    record.dirty,
                    "dirty must be true after apply_set_field_override"
                );
                assert!(
                    record.binding_version > 0,
                    "binding_version must be > 0 after apply_set_field_override, got {}",
                    record.binding_version
                );
                check_passed = true;
            }
        }
    });

    assert!(
        check_passed,
        "binding record must exist in registry after apply_set_field_override"
    );
}
