//! Spec §6: activation-ring-capped-at-64 (MUST, D3).

use editor_model::logic_activation::{
    LOGIC_ACTIVATION_RING_CAP, LogicActivationEvent, LogicActivationRing, ring_push,
};

#[test]
fn cap_holds_at_64_and_evicts_oldest_fifo() {
    let mut ring: LogicActivationRing = LogicActivationRing::with_capacity(64);
    for i in 0..100 {
        let event = LogicActivationEvent {
            node_id: format!("node_{i}"),
            triggered_at_ms: i as u64,
            payload_summary: None,
        };
        let evicted = ring_push(&mut ring, event);
        if i < LOGIC_ACTIVATION_RING_CAP {
            assert!(evicted.is_none(), "no evict before cap: i={i}");
        } else {
            assert!(evicted.is_some(), "evict at cap: i={i}");
            let evicted = evicted.unwrap();
            // The 1st event (i=0) is evicted at i=64; thereafter FIFO.
            let expected_oldest = if i == LOGIC_ACTIVATION_RING_CAP {
                "node_0"
            } else {
                // After cap is reached, the oldest is always the (i - cap + 1)th event.
                // For i=64 we evicted node_0. For i=65 we evicted node_1. ...
                &format!("node_{}", i - LOGIC_ACTIVATION_RING_CAP)
            };
            assert_eq!(evicted.node_id, expected_oldest, "wrong FIFO at i={i}");
        }
    }
    assert_eq!(ring.len(), LOGIC_ACTIVATION_RING_CAP);
    // After 100 pushes with cap 64, the ring holds events 36..=99 (oldest 36, newest 99).
    let oldest = ring.front().expect("ring non-empty after 100 pushes");
    assert_eq!(oldest.node_id, "node_36");
    let newest = ring.back().expect("ring non-empty after 100 pushes");
    assert_eq!(newest.node_id, "node_99");
}
