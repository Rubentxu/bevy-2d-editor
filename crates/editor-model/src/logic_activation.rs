//! Logic activation ring — records recent logic-graph activation events.
//
//! §6: Ring buffer capped at 64 entries; oldest entry evicted on overflow.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// One logic-graph node activation event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicActivationEvent {
    /// The activated node's ID.
    pub node_id: String,
    /// Monotonic timestamp in milliseconds when this event was recorded.
    pub triggered_at_ms: u64,
    /// Optional human-readable summary of the activation payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_summary: Option<String>,
}

/// A FIFO ring buffer of [`LogicActivationEvent`] entries.
pub type LogicActivationRing = VecDeque<LogicActivationEvent>;

/// Maximum number of events retained in [`LogicActivationRing`].
pub const LOGIC_ACTIVATION_RING_CAP: usize = 64;

/// Push an event onto the ring, evicting the oldest entry if at capacity.
///
/// Returns the evicted event, if any.
pub fn ring_push(
    ring: &mut LogicActivationRing,
    event: LogicActivationEvent,
) -> Option<LogicActivationEvent> {
    let evicted = if ring.len() >= LOGIC_ACTIVATION_RING_CAP {
        ring.pop_front()
    } else {
        None
    };
    ring.push_back(event);
    evicted
}
