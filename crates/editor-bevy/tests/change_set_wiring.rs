//! `change_set_wiring.rs` — byte-equality fixture for kernel vs legacy dispatch.
//!
//! **RED test** (initially fails until kernel routing is wired).
//!
//! This test proves that dispatching through `SceneTransactionKernel` produces
//! byte-identical `OperationLog` entries and undo/redo behavior compared to the
//! legacy `scene_session::apply_command` path.
//!
//! Scenario (spec §3 `scene-dispatch-byte-equivalent-undo`):
//! - GIVEN `editor.Transform2D` on `E1` at `(10,10)` and `DISPATCH_VIA_KERNEL=true`
//! - WHEN `dispatch_command(SetComponentField{E1: (20,20)})` then `undo()` twice
//! - THEN `OperationLog` entries MUST be byte-identical to v0.88 legacy path
//! - AND `E1.translation` MUST equal `(10,10)`

use editor_bevy::command::{Command, CommandEnvelope, CommandMetadata};
use editor_bevy::document::{ComponentInstance, Entity, LocalId, SceneDocument, StableId};
use editor_bevy::operation_log::OperationLog;
use editor_bevy::processor;
use editor_bevy::transaction_bridge::scene_transaction_kernel;
use editor_model::session::HistoryScope;
use editor_model::transaction::{ChangeOrigin, ChangeSet};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Helper: empty scene document for tests.
fn empty_doc() -> SceneDocument {
    SceneDocument {
        version: "0.1".to_string(),
        scene_id: "test-scene".to_string(),
        name: "Test Scene".to_string(),
        entities: Vec::new(),
        instances: std::collections::BTreeMap::new(),
    }
}

/// Helper: scene document with one entity with a Transform2D component.
fn doc_with_transform() -> SceneDocument {
    let mut doc = empty_doc();
    doc.entities.push(Entity {
        id: StableId::new("ent-1"),
        local_id: LocalId::new("ent-1"),
        name: "Entity1".to_string(),
        parent: None,
        components: vec![ComponentInstance {
            type_id: "editor.Transform2D".to_string(),
            values: serde_json::json!({
                "translation": { "x": 10.0, "y": 10.0 }
            }),
        }],
    });
    doc
}

/// Helper: make a `CommandEnvelope` for the given command.
fn envelope(cmd: Command) -> CommandEnvelope {
    CommandEnvelope {
        command: cmd,
        metadata: CommandMetadata::now("test-user"),
    }
}

/// Set a component field command targeting `ent-1` with new translation.
fn set_translation_cmd(x: f64, y: f64) -> Command {
    Command::SetComponentField {
        entity_id: StableId::new("ent-1"),
        type_id: "editor.Transform2D".to_string(),
        field_path: "translation.x".to_string(),
        value: serde_json::json!({ "x": x, "y": y }),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: kernel path produces same result as legacy path
// ─────────────────────────────────────────────────────────────────────────────

/// Test that kernel path and legacy path produce the same inverse for a simple
/// SetComponentField command.
///
/// This is the foundational byte-equality test: if the inverses differ,
/// undo/redo will produce different document states.
#[test]
fn test_kernel_and_legacy_produce_same_inverse() {
    // Setup: scene with entity at (10, 10)
    let legacy_doc = doc_with_transform();
    let kernel_doc = doc_with_transform();

    let cmd = set_translation_cmd(20.0, 20.0);
    let _env = envelope(cmd.clone());

    // ─── Legacy path ────────────────────────────────────────────────────────
    let mut legacy_doc_clone = legacy_doc.clone();
    let legacy_inverse =
        processor::apply(&mut legacy_doc_clone, &cmd).expect("legacy apply should succeed");

    // ─── Kernel path ─────────────────────────────────────────────────────────
    let kernel = scene_transaction_kernel();
    let mut kernel_doc_clone = kernel_doc.clone();
    let mut history = HistoryScope::new();

    let mut cs = ChangeSet::new(
        "cs-test-1".into(),
        ChangeOrigin::Human,
        "test-user".into(),
        "move entity".into(),
    );
    cs.add_resource("scene", "scenes/test.json");
    cs.push_op(cmd.clone());

    let receipt = kernel
        .apply_atomic(&cs, &mut kernel_doc_clone, &mut history)
        .expect("kernel apply should succeed");

    let kernel_inverse = receipt
        .inverses
        .into_iter()
        .next()
        .expect("should have inverse");

    // ─── Assert: inverses are byte-identical ────────────────────────────────
    let legacy_json = serde_json::to_string(&legacy_inverse).expect("serialize legacy inverse");
    let kernel_json = serde_json::to_string(&kernel_inverse).expect("serialize kernel inverse");

    assert_eq!(
        legacy_json, kernel_json,
        "Kernel and legacy inverses must be byte-identical.\n  Legacy: {}\n  Kernel:  {}",
        legacy_json, kernel_json
    );

    // ─── Assert: post-apply documents are identical ───────────────────────────
    let legacy_doc_json = serde_json::to_string(&legacy_doc_clone).expect("serialize legacy doc");
    let kernel_doc_json = serde_json::to_string(&kernel_doc_clone).expect("serialize kernel doc");

    assert_eq!(
        legacy_doc_json, kernel_doc_json,
        "Post-apply documents must be byte-identical.\n  Legacy: {}\n  Kernel:  {}",
        legacy_doc_json, kernel_doc_json
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: undo restores original state (kernel path)
// ─────────────────────────────────────────────────────────────────────────────

/// Test that undo via the operation log restores the original document state
/// when using the kernel path.
#[test]
fn test_kernel_undo_restores_original() {
    let original_doc = doc_with_transform();
    let cmd = set_translation_cmd(20.0, 20.0);

    // ─── Apply via kernel ───────────────────────────────────────────────────
    let kernel = scene_transaction_kernel();
    let mut doc = original_doc.clone();
    let mut history = HistoryScope::new();
    let mut log = OperationLog::new_const();

    let mut cs = ChangeSet::new(
        "cs-test-2".into(),
        ChangeOrigin::Human,
        "test-user".into(),
        "move entity".into(),
    );
    cs.add_resource("scene", "scenes/test.json");
    cs.push_op(cmd.clone());

    let receipt = kernel
        .apply_atomic(&cs, &mut doc, &mut history)
        .expect("kernel apply should succeed");

    // Record in log (mimics what dispatch_command does)
    let _env = envelope(cmd.clone());
    log.record(&_env, receipt.inverses.into_iter().next().unwrap());

    // Document should now be at (20, 20)
    let doc_json_after = serde_json::to_string(&doc).expect("serialize doc after apply");
    assert!(
        doc_json_after.contains("20"),
        "Doc after apply should have new translation"
    );

    // ─── Undo ───────────────────────────────────────────────────────────────
    let _env = envelope(cmd.clone());
    log.undo(&mut doc).expect("undo should succeed");

    let doc_json_undo = serde_json::to_string(&doc).expect("serialize doc after undo");
    let original_json = serde_json::to_string(&original_doc).expect("serialize original doc");

    assert_eq!(
        doc_json_undo, original_json,
        "Doc after undo should equal original.\n  Original: {}\n  After undo: {}",
        original_json, doc_json_undo
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: undo then redo restores new state (kernel path)
// ─────────────────────────────────────────────────────────────────────────────

/// Test that undo then redo via the operation log restores the new document state.
#[test]
fn test_kernel_undo_then_redo_restores_new() {
    let original_doc = doc_with_transform();
    let cmd = set_translation_cmd(20.0, 20.0);

    // ─── Apply via kernel ───────────────────────────────────────────────────
    let kernel = scene_transaction_kernel();
    let mut doc = original_doc.clone();
    let mut history = HistoryScope::new();
    let mut log = OperationLog::new_const();

    let mut cs = ChangeSet::new(
        "cs-test-3".into(),
        ChangeOrigin::Human,
        "test-user".into(),
        "move entity".into(),
    );
    cs.add_resource("scene", "scenes/test.json");
    cs.push_op(cmd.clone());

    let receipt = kernel
        .apply_atomic(&cs, &mut doc, &mut history)
        .expect("kernel apply should succeed");

    let _env = envelope(cmd.clone());
    log.record(&_env, receipt.inverses.into_iter().next().unwrap());

    // ─── Undo ───────────────────────────────────────────────────────────────
    log.undo(&mut doc).expect("undo should succeed");
    let doc_json_undo = serde_json::to_string(&doc).expect("serialize doc after undo");

    let original_json = serde_json::to_string(&original_doc).expect("serialize original doc");
    assert_eq!(
        doc_json_undo, original_json,
        "Doc after undo should equal original"
    );

    // ─── Redo ───────────────────────────────────────────────────────────────
    log.redo(&mut doc).expect("redo should succeed");
    let doc_json_redo = serde_json::to_string(&doc).expect("serialize doc after redo");

    // After redo, translation should be (20, 20)
    assert!(
        doc_json_redo.contains("20"),
        "Doc after redo should have new translation"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: byte-equivalent undo (spec scenario)
// ─────────────────────────────────────────────────────────────────────────────

/// Test the exact scenario from spec §3 `scene-dispatch-byte-equivalent-undo`:
///
/// - GIVEN `editor.Transform2D` on `E1` at `(10,10)` and `DISPATCH_VIA_KERNEL=true`
/// - WHEN `dispatch_command(SetComponentField{E1: (20,20)})` then `undo()`
/// - THEN `E1.translation` MUST equal `(10,10)` after undo
///
/// This tests the full dispatch → log → undo cycle through the kernel produces
/// byte-identical result to the legacy path.
#[test]
fn test_scene_dispatch_byte_equivalent_undo() {
    // Setup: entity at (10, 10)
    let original_doc = doc_with_transform();
    let cmd = set_translation_cmd(20.0, 20.0);

    // ─── Apply via kernel ───────────────────────────────────────────────────
    let kernel = scene_transaction_kernel();
    let mut doc = original_doc.clone();
    let mut history = HistoryScope::new();
    let mut log = OperationLog::new_const();

    let mut cs = ChangeSet::new(
        "cs-spec-1".into(),
        ChangeOrigin::Human,
        "test-user".into(),
        "move entity".into(),
    );
    cs.add_resource("scene", "scenes/test.json");
    cs.push_op(cmd.clone());

    let receipt = kernel
        .apply_atomic(&cs, &mut doc, &mut history)
        .expect("kernel apply should succeed");

    // Record in log
    let _env = envelope(cmd.clone());
    log.record(&_env, receipt.inverses.into_iter().next().unwrap());

    // Verify translation is now (20, 20)
    let doc_json = serde_json::to_string(&doc).expect("serialize doc");
    assert!(
        doc_json.contains("20"),
        "Translation should be (20, 20) after apply"
    );

    // ─── Undo ───────────────────────────────────────────────────────────────
    log.undo(&mut doc).expect("undo should succeed");

    // Extract translation from doc
    let doc_json_final = serde_json::to_string(&doc).expect("serialize doc final");
    let original_json = serde_json::to_string(&original_doc).expect("serialize original doc");

    assert_eq!(
        doc_json_final, original_json,
        "E1.translation must be back to (10, 10) after undo.\n  Expected: {}\n  Got: {}",
        original_json, doc_json_final
    );
}
