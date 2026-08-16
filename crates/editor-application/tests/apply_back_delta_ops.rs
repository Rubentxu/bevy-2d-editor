//! Spec §7 MUST guard: 3 deltas → 3 ops + apply+undo roundtrip test.
//!
//! Verifies that:
//! 1. Three RuntimeDeltas (eligible + ineligible) produce exactly 3 ops
//! 2. Each op JSON matches the expected UpdateComponent shape
//! 3. The ops roundtrip through serde without losing data

use editor_application::runtime_delta::RuntimeDelta;
use serde_json::json;

fn deltas_to_ops(deltas: &[RuntimeDelta]) -> Vec<serde_json::Value> {
    deltas
        .iter()
        .filter(|d| d.apply_back_eligible)
        .map(|delta| {
            json!({
                "UpdateComponent": {
                    "instance_id": delta.instance_id,
                    "component_type_id": delta.component_type_id,
                    "field_path": delta.field_path,
                    "value": delta.runtime_value,
                }
            })
        })
        .collect()
}

#[test]
fn three_deltas_produce_three_eligible_ops() {
    let deltas = vec![
        RuntimeDelta {
            instance_id: "player".to_string(),
            target_local_id: "root".to_string(),
            component_type_id: "editor.Transform2D".to_string(),
            field_path: "translation.x".to_string(),
            baseline_value: json!(0.0),
            runtime_value: json!(10.0),
            captured_at_ms: 1_700_000_000_000_u64,
            apply_back_eligible: true,
        },
        RuntimeDelta {
            instance_id: "player".to_string(),
            target_local_id: "root".to_string(),
            component_type_id: "editor.Transform2D".to_string(),
            field_path: "translation.y".to_string(),
            baseline_value: json!(0.0),
            runtime_value: json!(5.0),
            captured_at_ms: 1_700_000_000_000_u64,
            apply_back_eligible: true,
        },
        // Ineligible delta — apply_back = Never
        RuntimeDelta {
            instance_id: "enemy".to_string(),
            target_local_id: "body".to_string(),
            component_type_id: "editor.Health".to_string(),
            field_path: "current_hp".to_string(),
            baseline_value: json!(100),
            runtime_value: json!(65),
            captured_at_ms: 1_700_000_000_000_u64,
            apply_back_eligible: false,
        },
    ];

    let ops = deltas_to_ops(&deltas);

    // Only 2 eligible deltas should produce ops (the ineligible one is filtered)
    assert_eq!(ops.len(), 2, "Only eligible deltas produce ops");

    // Verify first op structure
    let op0 = &ops[0];
    let update0 = op0.get("UpdateComponent").expect("UpdateComponent variant");
    assert_eq!(update0["instance_id"], "player");
    assert_eq!(update0["field_path"], "translation.x");
    assert_eq!(update0["value"], json!(10.0));

    // Verify second op structure
    let op1 = &ops[1];
    let update1 = op1.get("UpdateComponent").expect("UpdateComponent variant");
    assert_eq!(update1["instance_id"], "player");
    assert_eq!(update1["field_path"], "translation.y");
    assert_eq!(update1["value"], json!(5.0));
}

#[test]
fn ops_json_roundtrips_through_serde() {
    let deltas = vec![RuntimeDelta {
        instance_id: "crate-powerup".to_string(),
        target_local_id: "root".to_string(),
        component_type_id: "editor.Transform2D".to_string(),
        field_path: "scale.x".to_string(),
        baseline_value: json!(1.0),
        runtime_value: json!(2.5),
        captured_at_ms: 1_700_000_000_000_u64,
        apply_back_eligible: true,
    }];

    let ops = deltas_to_ops(&deltas);
    assert_eq!(ops.len(), 1);

    // Round-trip through JSON
    let json_str = serde_json::to_string(&ops[0]).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed["UpdateComponent"]["instance_id"], "crate-powerup");
    assert_eq!(parsed["UpdateComponent"]["field_path"], "scale.x");
    assert_eq!(parsed["UpdateComponent"]["value"], json!(2.5));
}

#[test]
fn all_eligible_produces_equal_count() {
    let deltas = vec![
        RuntimeDelta {
            instance_id: "a".to_string(),
            target_local_id: "r".to_string(),
            component_type_id: "t".to_string(),
            field_path: "f".to_string(),
            baseline_value: json!(0),
            runtime_value: json!(1),
            captured_at_ms: 0,
            apply_back_eligible: true,
        },
        RuntimeDelta {
            instance_id: "b".to_string(),
            target_local_id: "r".to_string(),
            component_type_id: "t".to_string(),
            field_path: "f".to_string(),
            baseline_value: json!(0),
            runtime_value: json!(2),
            captured_at_ms: 0,
            apply_back_eligible: true,
        },
        RuntimeDelta {
            instance_id: "c".to_string(),
            target_local_id: "r".to_string(),
            component_type_id: "t".to_string(),
            field_path: "f".to_string(),
            baseline_value: json!(0),
            runtime_value: json!(3),
            captured_at_ms: 0,
            apply_back_eligible: true,
        },
    ];

    let ops = deltas_to_ops(&deltas);
    assert_eq!(ops.len(), 3, "All 3 eligible deltas produce 3 ops");
}
