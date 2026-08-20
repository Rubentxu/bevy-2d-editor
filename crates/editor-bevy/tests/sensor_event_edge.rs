//! Edge-transition tests for SensorEvent.
//!
//! R2: sensors emit on edge transitions only (0→1 = fire, 1→0 = no fire, 1→1 = no fire).
//! A key held for 5 seconds at 60fps MUST produce 1 event, not 300.
//!
//! Tests the `SensorStateCache` helpers and the edge-detection logic.

use editor_bevy::sensor_event::SensorEvent;
use editor_bevy::sensor_state_cache::SensorStateCache;
use editor_model::logic_graph::NodeId;

/// Helper: create a NodeId from a string.
fn node(s: &str) -> NodeId {
    NodeId(s.to_string())
}

/// Entity bits constant for testing.
const ENTITY_BITS: u64 = 0xDEADBEEF_u64;

#[test]
fn stable_input_no_event() {
    // GIVEN: sensor was not fired before (prev = false)
    let prev = false;
    // AND: sensor is not fired now (curr = false)
    let curr = false;

    // WHEN: we check for edge fire
    let is_edge = SensorStateCache::was_edge_fire(prev, curr);

    // THEN: no edge transition (0→0)
    assert!(!is_edge, "0→0 must not be an edge fire");
}

#[test]
fn edge_0_to_1_emits_event() {
    // GIVEN: sensor was not fired before (prev = false)
    let prev = false;
    // AND: sensor is now fired (curr = true)
    let curr = true;

    // WHEN: we check for edge fire
    let is_edge = SensorStateCache::was_edge_fire(prev, curr);

    // THEN: edge transition 0→1 is a fire event
    assert!(is_edge, "0→1 must be an edge fire");
}

#[test]
fn edge_1_to_0_no_event() {
    // GIVEN: sensor was fired before (prev = true)
    let prev = true;
    // AND: sensor is not fired now (curr = false)
    let curr = false;

    // WHEN: we check for edge fire
    let is_edge = SensorStateCache::was_edge_fire(prev, curr);

    // THEN: no edge transition (1→0 is release, not fire)
    assert!(!is_edge, "1→0 must not be an edge fire");
}

#[test]
fn edge_1_to_1_no_event() {
    // GIVEN: sensor was fired before (prev = true)
    let prev = true;
    // AND: sensor is still fired (curr = true)
    let curr = true;

    // WHEN: we check for edge fire
    let is_edge = SensorStateCache::was_edge_fire(prev, curr);

    // THEN: no edge transition (1→1 = held, not fire)
    assert!(!is_edge, "1→1 must not be an edge fire");
}

#[test]
fn sensor_event_fields() {
    // Verify SensorEvent carries the correct fields for the activation ring.
    let event = SensorEvent {
        entity_bits: ENTITY_BITS,
        node_id: node("sensor_jump"),
        payload: None,
    };

    assert_eq!(event.entity_bits, ENTITY_BITS);
    assert_eq!(event.node_id.0, "sensor_jump");
    assert!(event.payload.is_none());
}

#[test]
fn sensor_state_cache_update_and_get() {
    // GIVEN: an empty cache
    let mut cache = SensorStateCache::default();

    // WHEN: we get the previous state for an unknown entity/node
    let prev = cache.get_previous(ENTITY_BITS, &node("sensor_jump"));

    // THEN: default to false (never fired)
    assert!(!prev, "unknown sensor must default to false");

    // WHEN: we update the state to fired
    cache.update(ENTITY_BITS, node("sensor_jump"), true);

    // THEN: get_previous returns the new state
    let prev = cache.get_previous(ENTITY_BITS, &node("sensor_jump"));
    assert!(prev, "updated sensor state must be true");
}

#[test]
fn sensor_state_cache_remove_entity() {
    // GIVEN: a cache with state for an entity
    let mut cache = SensorStateCache::default();
    cache.update(ENTITY_BITS, node("sensor_jump"), true);
    assert!(cache.get_previous(ENTITY_BITS, &node("sensor_jump")));

    // WHEN: we remove the entity
    cache.remove_entity(ENTITY_BITS);

    // THEN: state reverts to default (false)
    let prev = cache.get_previous(ENTITY_BITS, &node("sensor_jump"));
    assert!(!prev, "after remove, sensor must default to false");
}
