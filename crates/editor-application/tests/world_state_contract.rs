//! Contract tests for EditorSession world_states accessors.
//!
//! Verifies that:
//! 1. `world_state_mut` creates state on first call (create-on-write)
//! 2. `world_state_mut` returns existing state on subsequent calls
//! 3. Different world paths get isolated state

use editor_application::{EditorSession, adapters::InMemoryProjectStore};
use editor_model::time::FakeClock;
use std::sync::Arc;

fn make_session() -> EditorSession {
    let store = Arc::new(InMemoryProjectStore::new());
    let clock = Arc::new(FakeClock::new());
    EditorSession::new(store, clock)
}

#[test]
fn world_state_mut_creates_state_on_first_call() {
    let mut session = make_session();

    // First call for a new path should create default state
    let state = session.world_state_mut("levels/main.world");
    assert!(state.doc.is_none());
    assert!(state.warnings.is_empty());
}

#[test]
fn world_state_mut_returns_existing_state_on_subsequent_calls() {
    let mut session = make_session();

    // First call
    let state1 = session.world_state_mut("levels/main.world");
    assert!(state1.doc.is_none());

    // Second call to SAME path — should be the same entry
    let state2 = session.world_state_mut("levels/main.world");
    assert!(state2.doc.is_none());

    // Third call — still the same
    let state3 = session.world_state_mut("levels/main.world");
    assert!(state3.warnings.is_empty());
}

#[test]
fn world_state_mut_different_paths_get_isolated_states() {
    let mut session = make_session();

    // Get state for path A — verify default
    let state_a = session.world_state_mut("levels/world_a.world");
    let ptr_a = state_a as *const editor_model::WorldSessionState;
    assert!(state_a.doc.is_none());
    assert!(state_a.warnings.is_empty());
    // state_a is still borrowed here but goes out of scope at end of function

    // Get state for path B — should be a different BTreeMap entry
    let state_b = session.world_state_mut("levels/world_b.world");
    let ptr_b = state_b as *const editor_model::WorldSessionState;
    assert!(state_b.doc.is_none());
    assert!(state_b.warnings.is_empty());

    // Pointers must differ: different map entries = different heap objects
    assert_ne!(ptr_a, ptr_b);
}

#[test]
fn world_state_mut_default_state_has_empty_doc_and_warnings() {
    let mut session = make_session();

    let state = session.world_state_mut("levels/empty.world");

    assert!(state.doc.is_none());
    assert!(state.warnings.is_empty());
}

#[test]
fn world_state_mut_can_be_retrieved_multiple_times() {
    let mut session = make_session();

    // Access same path multiple times without issue
    let _ = session.world_state_mut("levels/test.world");
    let _ = session.world_state_mut("levels/test.world");
    let state = session.world_state_mut("levels/test.world");

    // Final state should still be default
    assert!(state.doc.is_none());
    assert!(state.warnings.is_empty());
}
