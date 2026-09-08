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
// Thread-locals (fallback for tests without an EditorSession)
// ─────────────────────────────────────────────────────────────────────────────
//
// H2.5 Block E: these thread_locals are now the FALLBACK path for the
// session-owned `EditorSession.preview_inspector.{metrics, mapping,
// provenance}` fields. The canonical owner is the session (per
// ADR-0052). Production callers go through the session first; this
// thread_local only serves legacy tests that don't install a session.
//
// Pattern precedent: Block A2 ActuatorBus, v0.90 PR2
// `record_rebuild_cause` / `stamp_provenance`. See
// `docs/architecture/state-ownership-matrix.md` § H2.5 for the
// migration ledger.

thread_local! {
    /// Live preview metrics fallback. Updated by `emit_events` (FPS) and
    /// `rebuild_preview_world` (rebuild_count) when no session is
    /// installed. Production path is `EditorSession.preview_inspector.metrics`.
    static PREVIEW_METRICS_FALLBACK: RefCell<PreviewMetrics> = const { RefCell::new(PreviewMetrics {
        fps: 0.0,
        frame_time_ms: 0.0,
        rebuild_count: 0,
    }) };

    /// Per-instance preview mapping list fallback. Replaced atomically on
    /// each `rebuild_preview_world` call when no session is installed.
    /// Production path is `EditorSession.preview_inspector.mapping`.
    static PREVIEW_MAPPING_FALLBACK: RefCell<Vec<PreviewMappingEntry>> = const { RefCell::new(Vec::new()) };

    /// Per-instance provenance details fallback. Replaced atomically on
    /// each `rebuild_preview_world` call when no session is installed.
    /// Production path is `EditorSession.preview_inspector.provenance`.
    static PREVIEW_PROVENANCE_FALLBACK: RefCell<BTreeMap<StableId, PreviewProvenance>> =
        const { RefCell::new(BTreeMap::new()) };

    // v0.90 PR2: `LAST_REBUILD_CAUSE` and `PENDING_CAUSALITY_EDGES` thread_locals
    // are removed. Both now live canonically in `EditorSession` (via the
    // `EditorSessionPort` trait), reached from `editor-core` Bevy systems
    // through `editor_model::ports::with_session_mut`. ADR-0052 ratifies this
    // transition; the dual-write stance from v0.89 (thread_local + session
    // field) is collapsed to a single owner.
}

/// Replace the live preview metrics. Called by `emit_events` and on rebuild.
///
/// H2.5 Block E: writes through `editor_model::ports::with_session_mut`
/// to `EditorSession::preview_inspector.metrics`. Falls back to
/// `PREVIEW_METRICS_FALLBACK` when no session is installed (legacy
/// tests). Returns silently if both paths are unavailable.
pub fn set_metrics(metrics: PreviewMetrics) {
    let payload = serde_json::to_value(&metrics)
        .expect("PreviewMetrics serializes (round-trip tested)");
    let written = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().metrics = payload;
    })
    .is_some();
    if !written {
        PREVIEW_METRICS_FALLBACK.with(|m| *m.borrow_mut() = metrics);
    }
}

/// Increment the rebuild counter and return the new value.
///
/// H2.5 Block E: reads+mutates `EditorSession::preview_inspector.metrics`
/// via the session; falls back to `PREVIEW_METRICS_FALLBACK` when no
/// session is installed.
pub fn increment_rebuild_count() -> u32 {
    let via_session = editor_model::ports::with_session_mut(|sess| {
        let pi = sess.preview_inspector_mut();
        let mut m: PreviewMetrics = serde_json::from_value(pi.metrics.clone()).unwrap_or_default();
        m.rebuild_count = m.rebuild_count.saturating_add(1);
        pi.metrics = serde_json::to_value(&m)
            .expect("PreviewMetrics serializes (round-trip tested)");
        m.rebuild_count
    });
    if let Some(n) = via_session {
        return n;
    }
    PREVIEW_METRICS_FALLBACK.with(|m| {
        let mut m = m.borrow_mut();
        m.rebuild_count = m.rebuild_count.saturating_add(1);
        m.rebuild_count
    })
}

/// Replace the live preview mapping list.
///
/// H2.5 Block E: writes through `editor_model::ports::with_session_mut`
/// to `EditorSession::preview_inspector.mapping`. Falls back to
/// `PREVIEW_MAPPING_FALLBACK` when no session is installed.
pub fn set_mapping(entries: Vec<PreviewMappingEntry>) {
    let payload: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| serde_json::to_value(e).expect("PreviewMappingEntry serializes"))
        .collect();
    let written = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().mapping = payload;
    })
    .is_some();
    if !written {
        PREVIEW_MAPPING_FALLBACK.with(|m| *m.borrow_mut() = entries);
    }
}

/// Replace the live preview provenance map.
///
/// H2.5 Block E: writes through `editor_model::ports::with_session_mut`
/// to `EditorSession::preview_inspector.provenance`. Falls back to
/// `PREVIEW_PROVENANCE_FALLBACK` when no session is installed.
pub fn set_provenance(entries: BTreeMap<StableId, PreviewProvenance>) {
    let payload: std::collections::BTreeMap<String, serde_json::Value> = entries
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_string(),
                serde_json::to_value(v).expect("PreviewProvenance serializes"),
            )
        })
        .collect();
    let written = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().provenance = payload;
    })
    .is_some();
    if !written {
        PREVIEW_PROVENANCE_FALLBACK.with(|p| *p.borrow_mut() = entries);
    }
}

/// Read the live preview metrics (cloned).
///
/// H2.5 Block E: reads from `EditorSession::preview_inspector.metrics`
/// when a session is installed; falls back to
/// `PREVIEW_METRICS_FALLBACK` otherwise. Returns `Default::default()`
/// if the session metric cannot be deserialized (treat as empty).
pub fn get_metrics() -> PreviewMetrics {
    if let Some(v) = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().metrics.clone()
    }) {
        return serde_json::from_value(v).unwrap_or_default();
    }
    PREVIEW_METRICS_FALLBACK.with(|m| m.borrow().clone())
}

/// Read the live preview mapping (cloned).
///
/// H2.5 Block E: reads from `EditorSession::preview_inspector.mapping`
/// when a session is installed; falls back to
/// `PREVIEW_MAPPING_FALLBACK` otherwise. Entries that fail
/// deserialization are skipped (preserves the others).
pub fn get_mapping() -> Vec<PreviewMappingEntry> {
    if let Some(arr) = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().mapping.clone()
    }) {
        return arr
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
    }
    PREVIEW_MAPPING_FALLBACK.with(|m| m.borrow().clone())
}

/// Read the live preview provenance for a single `StableId`. Returns `None` if
/// no entry is found.
///
/// H2.5 Block E: reads from
/// `EditorSession::preview_inspector.provenance[stable_id]` when a
/// session is installed; falls back to `PREVIEW_PROVENANCE_FALLBACK`
/// otherwise. The session map is keyed by the inner `String` of
/// `StableId` (ADR-0049 Phase 1); the conversion is safe because both
/// `editor_model::StableId` and `editor_core::StableId` wrap the same
/// `String`.
pub fn get_provenance(stable_id: &str) -> Option<PreviewProvenance> {
    let sid_str = stable_id.to_string();
    if let Some(map) = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().provenance.clone()
    }) {
        return map
            .get(&sid_str)
            .and_then(|v| serde_json::from_value(v.clone()).ok());
    }
    PREVIEW_PROVENANCE_FALLBACK.with(|p| {
        p.borrow().get(&StableId::new(stable_id)).cloned()
    })
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
///
/// H2.5 Block E: writes through `set_provenance` so the dual-write
/// fallback path applies (session-first, then `PREVIEW_PROVENANCE_FALLBACK`).
pub fn apply_pending_causality_edges() {
    // Drain the pending map from the session.
    let pending_map: BTreeMap<editor_model::StableId, Vec<crate::CausalityEdge>> =
        match editor_model::ports::with_session_mut(|sess| {
            std::mem::take(sess.pending_causality_edges_mut())
        }) {
            Some(m) => m,
            None => return,
        };
    // Apply edges to provenance entries, then write back via set_provenance
    // (which respects the session-first/fallback dual-write contract).
    if pending_map.is_empty() {
        return;
    }
    // Read current provenance (session-first, fallback via the getters above).
    let mut current = get_provenance_map_for_internal_write();
    for (model_sid, edges) in pending_map {
        let sid: StableId = model_sid.into();
        if let Some(entry) = current.get_mut(&sid) {
            entry.causality_edges.extend(edges);
        }
    }
    set_provenance(current);
}

/// Internal helper for `apply_pending_causality_edges`: read the full
/// provenance map directly (without re-serializing through JSON),
/// preferring the session when installed and falling back to
/// `PREVIEW_PROVENANCE_FALLBACK`. Mirrors `get_provenance` but returns
/// the whole map instead of a single entry.
fn get_provenance_map_for_internal_write() -> BTreeMap<StableId, PreviewProvenance> {
    if let Some(map) = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().provenance.clone()
    }) {
        let mut out = BTreeMap::new();
        for (k, v) in map {
            if let Ok(entry) = serde_json::from_value::<PreviewProvenance>(v) {
                out.insert(StableId::new(k), entry);
            }
        }
        return out;
    }
    PREVIEW_PROVENANCE_FALLBACK.with(|p| p.borrow().clone())
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
