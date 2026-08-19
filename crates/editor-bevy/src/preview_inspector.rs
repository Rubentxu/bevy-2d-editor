//! Runtime Preview Inspector data model.
//!
//! Surfaces metrics and provenance of the Bevy preview world to the JS-side
//! inspector without leaking Bevy Entity IDs into the editor-owned model.
//! Per ADR-0006 §Capability 6 question "What runtime data can be exposed
//! without leaking Bevy Entity IDs into the editor model?", this module is
//! editor-owned: payloads reference `StableId`, `LocalId`, and `AssetReference`
//! only.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::document::StableId;
use crate::scene_asset::{AssetReference, LocalId};
use crate::SceneInstanceChild;
use bevy::prelude::Entity;

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

    // v0.90 PR2: `LAST_REBUILD_CAUSE` and `PENDING_CAUSALITY_EDGES` thread_locals
    // are removed. Both now live canonically in `EditorSession` (via the
    // `EditorSessionPort` trait), reached from `editor-core` Bevy systems
    // through `editor_model::ports::with_session_mut`. ADR-0052 ratifies this
    // transition; the dual-write stance from v0.89 (thread_local + session
    // field) is collapsed to a single owner.

    // RUNTIME-010: StableId -> EditorEntity index.
    // Maps StableId to the ECS entity in the PreviewWorld that carries it.
    // Populated by rebuild_preview_world after spawning scene entities.
    static STABLE_ID_INDEX: RefCell<HashMap<StableId, Entity>> =
        RefCell::new(HashMap::default());
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

// ─────────────────────────────────────────────────────────────────────────────
// RUNTIME-010: StableId → EditorEntity index
// ─────────────────────────────────────────────────────────────────────────────

/// Replace the StableId → EditorEntity index.
///
/// Called by `rebuild_preview_world` after spawning all scene entities.
/// The index maps each placed entity's StableId to its ECS Entity ID
/// in the PreviewWorld, enabling O(1) lookup instead of iterative queries.
pub fn set_stable_id_index(index: HashMap<StableId, Entity>) {
    STABLE_ID_INDEX.with(|i| *i.borrow_mut() = index);
}

/// Get the StableId → EditorEntity index.
///
/// Returns a clone of the current index. O(n) where n = number of entities.
/// For hot paths, prefer [`get_stable_id_entity`] for single lookups.
pub fn get_stable_id_index() -> HashMap<StableId, Entity> {
    STABLE_ID_INDEX.with(|i| i.borrow().clone())
}

/// Look up a single StableId in the index.
///
/// Returns the ECS Entity ID if found, None otherwise.
pub fn get_stable_id_entity(stable_id: &StableId) -> Option<Entity> {
    STABLE_ID_INDEX.with(|i| i.borrow().get(stable_id).copied())
}

// ─── §6 RebuildCause (v0.90 PR2: migrated to EditorSession via EditorSessionPort) ──

/// Record a rebuild cause (§6). Called by `rebuild_preview_world` and
/// `process_commands` (legacy sprite-move) to stamp the last trigger.
///
/// v0.90 PR2: writes through `editor_model::ports::with_session_mut` to
/// `EditorSession::preview_inspector.last_rebuild_cause` (the canonical
/// owner per ADR-0052). Returns silently if the session is not yet
/// initialized (Bevy systems may run before `init_project_store` in tests).
pub fn record_rebuild_cause(cause: crate::RebuildCause) {
    let _ = editor_model::ports::with_session_mut(|sess| {
        *sess.last_rebuild_cause_mut() = Some(cause);
    });
}

/// Read the last recorded rebuild cause, if any.
///
/// v0.90 PR2: reads from the session via `EditorSessionPort`. Returns
/// `None` if the session is not yet initialized.
pub fn last_rebuild_cause() -> Option<crate::RebuildCause> {
    editor_model::ports::with_session_mut(|sess| sess.last_rebuild_cause_mut().clone()).flatten()
}

// ─── §6 CausalityEdge (v0.90 PR2: migrated to EditorSession via EditorSessionPort) ──

/// Record a [`CausalityEdge`] to be attached to a [`PreviewProvenance`] entry.
///
/// v0.90 PR2: writes through `editor_model::ports::with_session_mut` to
/// `EditorSession::pending_causality_edges`. The edges are drained and
/// applied to `PREVIEW_PROVENANCE` by `apply_pending_causality_edges` at
/// the end of a preview rebuild.
pub fn stamp_provenance(stable_id: StableId, edge: crate::CausalityEdge) {
    // Convert editor_core::StableId to editor_model::StableId via From impl
    // (ADR-0049 Phase 1: canonical type lives in editor_model).
    let model_sid: editor_model::StableId = stable_id.into();
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.pending_causality_edges_mut()
            .entry(model_sid)
            .or_insert_with(Vec::new)
            .push(edge);
    });
}

/// Apply all pending causality edges to `PREVIEW_PROVENANCE`.
///
/// Called at the end of `push_preview_inspector_state` so that edges recorded
/// during logic evaluation are attached to the correct provenance entries.
///
/// v0.90 PR2: drains from `EditorSession::pending_causality_edges` instead
/// of the removed `PENDING_CAUSALITY_EDGES` thread_local. The map keys are
/// `editor_model::StableId`; the existing `PREVIEW_PROVENANCE` map keys are
/// `document::StableId` (the editor-core mirror). The conversion via `.0` is
/// safe because the inner `String` representation is identical.
pub fn apply_pending_causality_edges() {
    // Drain the pending map from the session.
    let pending_map: BTreeMap<editor_model::StableId, Vec<crate::CausalityEdge>> =
        match editor_model::ports::with_session_mut(|sess| {
            std::mem::take(sess.pending_causality_edges_mut())
        }) {
            Some(m) => m,
            None => return,
        };
    // Apply edges to provenance entries.
    if !pending_map.is_empty() {
        PREVIEW_PROVENANCE.with(|prov| {
            let mut prov_map = prov.borrow_mut();
            for (model_sid, edges) in pending_map {
                let sid: StableId = model_sid.into();
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

// ─────────────────────────────────────────────────────────────────────────────
// RUNTIME-010 tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod stable_id_index_tests {
    use super::*;

    #[test]
    fn stable_id_index_empty_by_default() {
        let index = get_stable_id_index();
        assert!(index.is_empty(), "index should be empty initially");
    }

    #[test]
    fn set_and_get_stable_id_index() {
        use std::collections::HashMap;
        use bevy::prelude::Entity;

        let mut map: HashMap<StableId, Entity> = HashMap::new();
        let stable_id = StableId::new("test_entity_1");
        let entity = Entity::from_bits(42u64);
        map.insert(stable_id.clone(), entity);

        set_stable_id_index(map);

        let retrieved = get_stable_id_entity(&stable_id);
        assert!(retrieved.is_some(), "should find the inserted StableId");
        assert_eq!(retrieved.unwrap(), entity, "entity should match");

        let full_index = get_stable_id_index();
        assert_eq!(full_index.len(), 1, "index should have one entry");
    }

    #[test]
    fn stable_id_index_none_for_missing_key() {
        let missing = StableId::new("does_not_exist");
        let result = get_stable_id_entity(&missing);
        assert!(result.is_none(), "missing StableId should return None");
    }
}
