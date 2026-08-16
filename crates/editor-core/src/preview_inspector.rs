//! Runtime Preview Inspector data model.
//!
//! Surfaces metrics and provenance of the Bevy preview world to the JS-side
//! inspector without leaking Bevy Entity IDs into the editor-owned model.
//! Per ADR-0006 §Capability 6 question "What runtime data can be exposed
//! without leaking Bevy Entity IDs into the editor model?", this module is
//! editor-owned: payloads reference `StableId`, `LocalId`, and `AssetReference`
//! only.

use std::cell::RefCell;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::document::StableId;
use crate::scene_asset::{AssetReference, LocalId};

/// Live preview metrics: frames per second, last frame time in milliseconds,
/// and number of times the preview world has been rebuilt.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PreviewMetrics {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub rebuild_count: u32,
}

/// One entry in the preview entity mapping list.
///
/// The payload is `StableId`-only on the editor side. No Bevy Entity ID is
/// exposed. The Bevy world maps these `StableId`s to its own Entity at
/// projection time, but that mapping is internal to the runtime and is not
/// surfaced to JS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewMappingEntry {
    pub stable_id: StableId,
    pub local_id: LocalId,
    pub asset_ref: AssetReference,
    pub component_count: usize,
}

/// Per-instance provenance detail returned by `get_preview_provenance_wasm`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewProvenance {
    pub stable_id: StableId,
    pub local_id: LocalId,
    pub asset_ref: AssetReference,
    pub components: Vec<String>,
    pub is_from_instance: bool,
    /// §6: Causality edges — typed provenance links to other editor entities.
    pub causality_edges: Vec<crate::CausalityEdge>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Thread-locals (single-threaded WASM surface)
// ─────────────────────────────────────────────────────────────────────────────

thread_local! {
    /// Live preview metrics. Updated by `emit_events` (FPS) and
    /// `rebuild_preview_world` (rebuild_count).
    static PREVIEW_METRICS: RefCell<PreviewMetrics> = const { RefCell::new(PreviewMetrics {
        fps: 0.0,
        frame_time_ms: 0.0,
        rebuild_count: 0,
    }) };

    /// Per-instance preview mapping list. Replaced atomically on each
    /// `rebuild_preview_world` call.
    static PREVIEW_MAPPING: RefCell<Vec<PreviewMappingEntry>> = const { RefCell::new(Vec::new()) };

    /// Per-instance provenance details. Replaced atomically on each
    /// `rebuild_preview_world` call.
    static PREVIEW_PROVENANCE: RefCell<BTreeMap<StableId, PreviewProvenance>> =
        const { RefCell::new(BTreeMap::new()) };

    /// Last rebuild cause recorded by §6. Written by `record_rebuild_cause`.
    static LAST_REBUILD_CAUSE: RefCell<Option<crate::RebuildCause>> = const { RefCell::new(None) };

    /// Pending causality edges collected during logic evaluation (§6).
    /// Keyed by target StableId; drained and applied to PREVIEW_PROVENANCE
    /// at the end of each preview rebuild.
    static PENDING_CAUSALITY_EDGES: RefCell<BTreeMap<StableId, Vec<crate::CausalityEdge>>> =
        const { RefCell::new(BTreeMap::new()) };
}

/// Replace the live preview metrics. Called by `emit_events` and on rebuild.
pub fn set_metrics(metrics: PreviewMetrics) {
    PREVIEW_METRICS.with(|m| *m.borrow_mut() = metrics);
}

/// Increment the rebuild counter and return the new value.
pub fn increment_rebuild_count() -> u32 {
    PREVIEW_METRICS.with(|m| {
        let mut m = m.borrow_mut();
        m.rebuild_count = m.rebuild_count.saturating_add(1);
        m.rebuild_count
    })
}

/// Replace the live preview mapping list.
pub fn set_mapping(entries: Vec<PreviewMappingEntry>) {
    PREVIEW_MAPPING.with(|m| *m.borrow_mut() = entries);
}

/// Replace the live preview provenance map.
pub fn set_provenance(entries: BTreeMap<StableId, PreviewProvenance>) {
    PREVIEW_PROVENANCE.with(|p| *p.borrow_mut() = entries);
}

/// Read the live preview metrics (cloned).
pub fn get_metrics() -> PreviewMetrics {
    PREVIEW_METRICS.with(|m| m.borrow().clone())
}

/// Read the live preview mapping (cloned).
pub fn get_mapping() -> Vec<PreviewMappingEntry> {
    PREVIEW_MAPPING.with(|m| m.borrow().clone())
}

/// Read the live preview provenance for a single `StableId`. Returns `None` if
/// no entry is found.
pub fn get_provenance(stable_id: &str) -> Option<PreviewProvenance> {
    let sid = StableId::new(stable_id);
    PREVIEW_PROVENANCE.with(|p| p.borrow().get(&sid).cloned())
}

// ─── §6 RebuildCause ─────────────────────────────────────────────────────────

/// Record a rebuild cause (§6). Called by `rebuild_preview_world` and
/// `process_commands` (legacy sprite-move) to stamp the last trigger.
pub fn record_rebuild_cause(cause: crate::RebuildCause) {
    LAST_REBUILD_CAUSE.with(|c| *c.borrow_mut() = Some(cause));
}

/// Read the last recorded rebuild cause, if any.
pub fn last_rebuild_cause() -> Option<crate::RebuildCause> {
    LAST_REBUILD_CAUSE.with(|c| c.borrow().clone())
}

// ─── §6 CausalityEdge ─────────────────────────────────────────────────────────

/// Record a [`CausalityEdge`] to be attached to a [`PreviewProvenance`] entry.
///
/// Edges are buffered in a thread-local and applied to `PREVIEW_PROVENANCE`
/// when `apply_pending_causality_edges` is called at the end of a preview rebuild.
pub fn stamp_provenance(stable_id: StableId, edge: crate::CausalityEdge) {
    PENDING_CAUSALITY_EDGES.with(|edges| {
        let mut map = edges.borrow_mut();
        map.entry(stable_id).or_insert_with(Vec::new).push(edge);
    });
}

/// Apply all pending causality edges to `PREVIEW_PROVENANCE`.
///
/// Called at the end of `push_preview_inspector_state` so that edges recorded
/// during logic evaluation are attached to the correct provenance entries.
pub fn apply_pending_causality_edges() {
    // Take the pending map out of the RefCell, leaving an empty one behind.
    let pending_map = PENDING_CAUSALITY_EDGES.with(|pending| {
        std::mem::take(&mut *pending.borrow_mut())
    });
    // Apply edges to provenance entries.
    if !pending_map.is_empty() {
        PREVIEW_PROVENANCE.with(|prov| {
            let mut prov_map = prov.borrow_mut();
            for (sid, edges) in pending_map {
                if let Some(entry) = prov_map.get_mut(&sid) {
                    entry.causality_edges.extend(edges);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entry(id: &str, local: &str) -> PreviewMappingEntry {
        PreviewMappingEntry {
            stable_id: StableId::new(id),
            local_id: LocalId::new(local),
            asset_ref: AssetReference::new("assets/test"),
            component_count: 3,
        }
    }

    #[test]
    fn metrics_round_trip() {
        let m = PreviewMetrics {
            fps: 60.0,
            frame_time_ms: 16.6,
            rebuild_count: 5,
        };
        let json = serde_json::to_string(&m).unwrap();
        let rt: PreviewMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, m);
    }

    #[test]
    fn mapping_round_trip() {
        let entries = vec![test_entry("inst_1", "root"), test_entry("inst_2", "weapon")];
        let json = serde_json::to_string(&entries).unwrap();
        let rt: Vec<PreviewMappingEntry> = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, entries);
    }

    #[test]
    fn mapping_contains_no_bevy_entity_id_field() {
        // Defense-in-depth: the serialized JSON must not include any Bevy Entity
        // identifier. We assert by checking that no `bevy_entity`, `entity_id`,
        // or similar field names appear.
        let entries = vec![test_entry("inst_1", "root")];
        let value: serde_json::Value = serde_json::to_value(&entries).unwrap();
        let serialized = serde_json::to_string(&value).unwrap();
        assert!(
            !serialized.contains("bevy_entity"),
            "found Bevy Entity id leak"
        );
        assert!(!serialized.contains("entity_id"), "found entity id leak");
    }
}
