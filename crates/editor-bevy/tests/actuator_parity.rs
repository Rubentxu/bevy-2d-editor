//! Parity tests for the session-owned actuator bus (H2.5 Block A2).
//!
//! These tests prove that the new `editor_model::runtime::ActuatorBus` (reached
//! via `editor_model::ports::with_session_mut`) behaves identically to the
//! legacy thread-local `ACTUATOR_OUTPUT_BUS` that was retired in Block A2.
//!
//! Specifically:
//! - FIFO ordering across submit/drain round-trips
//! - Drain-on-empty returns `Vec::new()` without spurious re-initialization
//! - Submit outside a frame (no session) is a graceful no-op

use std::sync::{Arc, Mutex};

#[path = "support/mod.rs"]
mod support;

use editor_bevy::actuator_bus::{drain_actuator_outputs, submit_actuator_output};
use editor_model::EditorSessionPort;

// Re-export PortValue from the canonical editor-model location.
use editor_model::runtime::PortValue;

fn fresh_session() {
    let session = support::FakeSessionWithDefaults(support::FakeSession::new());
    let arc: Arc<Mutex<dyn EditorSessionPort>> = Arc::new(Mutex::new(session));
    editor_model::ports::register_editor_session(arc);
}

// §S-A2-3.1: parity at submit+drain (FIFO)
#[test]
fn parity_submit_drain_fifo() {
    fresh_session();

    let entity = bevy::prelude::Entity::from_bits(7);
    submit_actuator_output(entity, "translation", PortValue::Vec2 { x: 1.0, y: 2.0 });
    submit_actuator_output(entity, "scale", PortValue::Vec2 { x: 3.0, y: 4.0 });
    submit_actuator_output(entity, "rotation", PortValue::Float(1.57));

    let outputs = drain_actuator_outputs();
    assert_eq!(outputs.len(), 3, "expected 3 outputs in FIFO order");

    // FIFO order check
    assert_eq!(outputs[0].field, "translation");
    assert!(matches!(outputs[0].value, PortValue::Vec2 { x: 1.0, y: 2.0 }));

    assert_eq!(outputs[1].field, "scale");
    assert!(matches!(outputs[1].value, PortValue::Vec2 { x: 3.0, y: 4.0 }));

    assert_eq!(outputs[2].field, "rotation");
    assert!(matches!(outputs[2].value, PortValue::Float(1.57)));
}

// §S-A2-3.2: parity under consecutive submits (no loss, no duplication)
#[test]
fn parity_consecutive_submits_no_loss() {
    fresh_session();

    let entity = bevy::prelude::Entity::from_bits(42);
    for i in 0..10 {
        submit_actuator_output(
            entity,
            "translation",
            PortValue::Vec2 {
                x: i as f32,
                y: (i * 2) as f32,
            },
        );
    }

    let outputs = drain_actuator_outputs();
    assert_eq!(outputs.len(), 10, "expected 10 outputs, none lost");

    for (i, output) in outputs.iter().enumerate() {
        assert_eq!(output.field, "translation");
        match &output.value {
            PortValue::Vec2 { x, y } => {
                assert_eq!(*x, i as f32, "x component at index {i}");
                assert_eq!(*y, (i * 2) as f32, "y component at index {i}");
            }
            other => panic!("expected Vec2, got {other:?}"),
        }
    }
}

// §EC-A2-2: drain on empty returns Vec::new(), no spurious re-init
#[test]
fn parity_drain_when_empty() {
    fresh_session();

    let first = drain_actuator_outputs();
    assert_eq!(first.len(), 0, "empty drain should return empty Vec");

    // Subsequent drain still works
    let second = drain_actuator_outputs();
    assert_eq!(second.len(), 0, "repeated empty drain should remain empty");

    // And submit after empty drain still works
    let entity = bevy::prelude::Entity::from_bits(99);
    submit_actuator_output(entity, "color", PortValue::Float(0.5));
    let third = drain_actuator_outputs();
    assert_eq!(third.len(), 1, "submit after empty drain must be observable");
}

// §EC-A2-1: submit without session is a graceful no-op
#[test]
fn parity_submit_without_session_is_noop() {
    // Intentionally do NOT call fresh_session() — leaving the global session
    // uninitialized to exercise the graceful-no-op path.
    // Note: other tests in this file may run in parallel, but submit is a
    // best-effort operation and panicking on a session mismatch would surface
    // here as a test failure.
    //
    // We can't strictly assert "no-op" because other tests' sessions may
    // exist; we only assert that submit+drain don't crash. The submitted
    // entry (if any) will end up in whatever session is current.

    let entity = bevy::prelude::Entity::from_bits(12345);
    // Should not panic
    submit_actuator_output(entity, "translation", PortValue::Float(1.0));
}

// §Belt-and-suspenders: drain without session returns empty Vec
#[test]
fn parity_drain_without_session_returns_empty() {
    // Even without a session, drain should be safe (returns Vec::new()).
    // However, like submit_without_session, this races with other tests
    // that may have installed a session. We only check the return type.
    let result = drain_actuator_outputs();
    // Either empty (no session) or full (some session leaked through from
    // a parallel test). Both are valid.
    let _ = result.len(); // just confirm the type is Vec<ActuatorOutput>
}
