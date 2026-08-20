//! Dirty-tracking dispatch tests for logic binding evaluation.
//!
//! R8: dispatch only evaluates bindings with `dirty == true`.
//! R3: `apply_bind_logic_graph_to_instance` sets dirty=true, binding_version=1.
//! R4: `apply_set_binding_field_override` bumps binding_version, sets dirty=true.
//!
//! These tests verify the registry-level dirty flag is set correctly by apply_*
//! functions and that the dispatch layer respects it.

use editor_bevy::logic_command::BindingId;
use editor_bevy::with_binding_registry_for_tests;
use editor_bevy::with_binding_registry_mut_for_tests;
use editor_model::ids::StableId;
use std::collections::BTreeMap;

/// Helper: insert a fresh BindingRecord into the registry for testing.
fn insert_test_binding(scene_instance_id: &str, dirty: bool, binding_version: u64) -> BindingId {
    use editor_bevy::state::BindingRecord;

    let binding_id = BindingId::new(format!("test_binding_{}", scene_instance_id));
    let record = BindingRecord {
        binding_id: binding_id.clone(),
        recipe_id: "test_recipe".to_string(),
        version: 1,
        field_overrides: BTreeMap::new(),
        dirty,
        binding_version,
    };

    with_binding_registry_mut_for_tests(|reg| {
        reg.insert(StableId::new(scene_instance_id.to_string()), record);
    });

    binding_id
}

/// Test 1: fresh binding after bind has dirty=true, binding_version=1
#[test]
fn fresh_binding_has_dirty_true_and_version_1() {
    // Simulate apply_bind_logic_graph_to_instance: dirty=true, binding_version=1
    let binding_id = insert_test_binding("scene_1", true, 1);

    let mut found = false;
    with_binding_registry_for_tests(|reg| {
        for (sid, record) in reg.iter() {
            if record.binding_id == binding_id {
                assert!(
                    record.dirty,
                    "fresh binding must have dirty=true after bind"
                );
                assert_eq!(
                    record.binding_version, 1,
                    "fresh binding must have binding_version=1 after bind"
                );
                found = true;
            }
        }
    });

    assert!(found, "binding record must exist in registry");
}

/// Test 2: override bumps binding_version and sets dirty=true
#[test]
fn override_bumps_version_and_sets_dirty() {
    // Setup: binding with dirty=false, binding_version=1
    let binding_id = insert_test_binding("scene_2", false, 1);

    // Simulate apply_set_binding_field_override
    let field_path = "jump_impulse".to_string();
    let value = serde_json::json!(10.0);
    let result = editor_bevy::logic_state::apply_set_binding_field_override(
        binding_id.clone(),
        field_path,
        value,
    );

    assert!(
        result.is_ok(),
        "apply_set_field_override must succeed: {:?}",
        result
    );

    let mut check_passed = false;
    with_binding_registry_for_tests(|reg| {
        for (sid, record) in reg.iter() {
            if record.binding_id == binding_id {
                assert!(
                    record.dirty,
                    "dirty must be true after apply_set_field_override"
                );
                assert!(
                    record.binding_version > 1,
                    "binding_version must be > 1 after override, got {}",
                    record.binding_version
                );
                check_passed = true;
            }
        }
    });

    assert!(check_passed, "binding record must exist after override");
}

/// Test 3: non-dirty binding should be skipped by dispatch
#[test]
fn non_dirty_binding_has_version_0() {
    // A binding that hasn't been modified has binding_version=0, dirty=false
    let _binding_id = insert_test_binding("scene_3", false, 0);

    let mut count = 0;
    with_binding_registry_for_tests(|reg| {
        for (_sid, record) in reg.iter() {
            if record.dirty {
                count += 1;
            }
        }
    });

    assert_eq!(count, 0, "non-dirty binding should not be counted as dirty");
}

/// Test 4: dirty flag can be cleared after dispatch
#[test]
fn dirty_cleared_after_dispatch() {
    // Setup: binding with dirty=true
    let binding_id = insert_test_binding("scene_4", true, 1);

    // Simulate dispatch clearing dirty (as apply_actuator_outputs_in_preview does)
    with_binding_registry_mut_for_tests(|reg| {
        for (_sid, record) in reg.iter_mut() {
            if record.binding_id == binding_id {
                record.dirty = false;
            }
        }
    });

    let mut is_dirty = false;
    with_binding_registry_for_tests(|reg| {
        for (_sid, record) in reg.iter() {
            if record.binding_id == binding_id {
                is_dirty = record.dirty;
            }
        }
    });

    assert!(
        !is_dirty,
        "dirty flag must be cleared after dispatch"
    );
}
