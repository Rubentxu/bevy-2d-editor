//! Parity tests for the session-owned runtime command/event buses
//! (H2.5 Block F).
//!
//! Block A (v0.108.0) shipped the session-first runtime path for the
//! four `get_*_bus_ptr/len` WASM exports. Block F renames the
//! legacy `COMMAND_BUS` / `EVENT_BUS` thread_locals to
//! `COMMAND_BUS_FALLBACK` / `EVENT_BUS_FALLBACK` and removes the
//! duplicated local `LinearBus` struct (now `editor_model::runtime::LinearBus`).
//!
//! These tests prove that the session path continues to work after
//! Block F:
//! - Session-installed: `runtime_command_bus_mut()` and
//!   `runtime_event_bus_mut()` return the session-bus references.
//! - Drain + write semantics: `LinearBus::drain` returns events in
//!   FIFO order; `reset` clears the write offset.
//! - Two-session isolation: two `EditorSession`s do not leak state.

#[path = "support/mod.rs"]
mod support;

use editor_model::runtime::LinearBus;

fn fresh_session() {
    let session = support::FakeSessionWithDefaults(support::FakeSession::new());
    let arc = std::sync::Arc::new(std::sync::Mutex::new(session));
    editor_model::ports::register_editor_session(arc);
}

// §S-F-1: command_bus session path is reachable and writeable.
#[test]
fn parity_command_bus_session_path() {
    fresh_session();
    let wrote = editor_model::ports::with_session_mut(|s| {
        let bus: &mut LinearBus = s.runtime_command_bus_mut();
        bus.write(0x1234, &[1, 2, 3, 4])
    });
    assert_eq!(wrote, Some(true), "command bus write must succeed");

    let drained = editor_model::ports::with_session_mut(|s| {
        let bus: &mut LinearBus = s.runtime_command_bus_mut();
        bus.drain()
    });
    let drained = drained.unwrap();
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].0, 0x1234);
    assert_eq!(drained[0].1, vec![1, 2, 3, 4]);
}

// §S-F-2: event_bus session path: write → drain returns FIFO order.
#[test]
fn parity_event_bus_session_drain_fifo() {
    fresh_session();
    editor_model::ports::with_session_mut(|s| {
        let bus: &mut LinearBus = s.runtime_event_bus_mut();
        bus.write(0x0001, &[10]);
        bus.write(0x0002, &[20, 21]);
        bus.write(0x0003, &[30, 31, 32]);
    });

    let drained = editor_model::ports::with_session_mut(|s| {
        let bus: &mut LinearBus = s.runtime_event_bus_mut();
        bus.drain()
    });
    let drained = drained.unwrap();
    assert_eq!(drained.len(), 3, "expected 3 events in FIFO order");
    assert_eq!(drained[0].0, 0x0001);
    assert_eq!(drained[0].1, vec![10]);
    assert_eq!(drained[1].0, 0x0002);
    assert_eq!(drained[1].1, vec![20, 21]);
    assert_eq!(drained[2].0, 0x0003);
    assert_eq!(drained[2].1, vec![30, 31, 32]);
}

// §S-F-3: drain on empty returns empty Vec without spurious re-init.
#[test]
fn parity_drain_empty() {
    fresh_session();
    let drained = editor_model::ports::with_session_mut(|s| {
        let bus: &mut LinearBus = s.runtime_event_bus_mut();
        bus.drain()
    });
    let drained = drained.unwrap();
    assert!(drained.is_empty(), "empty drain returns no events");
}

// §S-F-4: reset clears the write offset.
#[test]
fn parity_reset_clears_write_offset() {
    fresh_session();
    editor_model::ports::with_session_mut(|s| {
        let bus: &mut LinearBus = s.runtime_command_bus_mut();
        bus.write(0xABCD, &[1, 2, 3]);
    });
    // Drain once: should return 1 event.
    let drained_first = editor_model::ports::with_session_mut(|s| {
        let bus: &mut LinearBus = s.runtime_command_bus_mut();
        bus.drain()
    })
    .unwrap();
    assert_eq!(drained_first.len(), 1);

    // Drain again immediately: empty (reset happened in drain).
    let drained_second = editor_model::ports::with_session_mut(|s| {
        let bus: &mut LinearBus = s.runtime_command_bus_mut();
        bus.drain()
    })
    .unwrap();
    assert!(
        drained_second.is_empty(),
        "second drain after first must be empty"
    );
}

// §S-F-5: two-session isolation — sessions don't share bus state.
#[test]
fn parity_two_session_isolation() {
    // Session A.
    fresh_session();
    editor_model::ports::with_session_mut(|s| {
        let bus: &mut LinearBus = s.runtime_event_bus_mut();
        bus.write(0xAAAA, &[1]);
    });

    // Session B (replaces A).
    fresh_session();
    let drained_b = editor_model::ports::with_session_mut(|s| {
        let bus: &mut LinearBus = s.runtime_event_bus_mut();
        bus.drain()
    })
    .unwrap();
    assert!(
        drained_b.is_empty(),
        "session B must not inherit session A's events"
    );
}
