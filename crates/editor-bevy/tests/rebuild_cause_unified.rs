//! v0.90 PR2 (MUST) — RebuildCause + CausalityEdge unification (ADR-0052).
//!
//! Verifies that `LAST_REBUILD_CAUSE` and `PENDING_CAUSALITY_EDGES` thread_locals
//! in `editor-core::preview_inspector` are gone, and that the write path
//! goes through `EditorSession` (via the `EditorSessionPort` trait).
//!
//! The `get_rebuild_cause_wasm` export reads from the session only (no
//! dual-read fallback from the removed thread_local).

#[path = "support/mod.rs"]
mod support;

use editor_model::EditorSessionPort;
use editor_model::{CausalityEdge, CausalityEdgeKind, RebuildCause, StableId};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

fn fresh_session() {
    let session = support::FakeSessionWithDefaults(support::FakeSession::new());
    let arc: Arc<Mutex<dyn EditorSessionPort>> = Arc::new(Mutex::new(session));
    editor_model::ports::register_editor_session(arc);
}

#[test]
fn record_rebuild_cause_writes_via_session() {
    fresh_session();
    // The thread_local LAST_REBUILD_CAUSE no longer exists; we write through
    // the trait.
    editor_core::preview_inspector::record_rebuild_cause(RebuildCause::UserEdit {
        command_id: "legacy_sprite_move".to_string(),
    });

    let read = editor_model::ports::with_session_mut(|sess| sess.last_rebuild_cause_mut().clone())
        .flatten();
    assert!(read.is_some(), "session must carry the rebuild cause");
    let cause = read.unwrap();
    match cause {
        RebuildCause::UserEdit { command_id } => {
            assert_eq!(command_id, "legacy_sprite_move");
        }
        _ => panic!("expected UserEdit, got {cause:?}"),
    }
}

#[test]
fn last_rebuild_cause_reads_via_session() {
    fresh_session();
    // Write directly to session, then verify editor-core's last_rebuild_cause()
    // (which now reads via with_session_mut) returns the same value.
    let _ = editor_model::ports::with_session_mut(|sess| {
        *sess.last_rebuild_cause_mut() = Some(RebuildCause::PlayModeEnter);
    });
    let cause = editor_core::preview_inspector::last_rebuild_cause();
    assert!(matches!(cause, Some(RebuildCause::PlayModeEnter)));
}

#[test]
fn stamp_provenance_writes_via_session() {
    fresh_session();
    let sid = editor_core::document::StableId::new("E1");
    let edge = CausalityEdge {
        edge_kind: CausalityEdgeKind::Definition,
        target_stable_id: "def1".to_string(),
    };
    editor_core::preview_inspector::stamp_provenance(sid.clone(), edge.clone());

    let edges: Option<Vec<CausalityEdge>> = editor_model::ports::with_session_mut(|sess| {
        sess.pending_causality_edges_mut()
            .get(&StableId::new(sid.as_str()))
            .cloned()
    })
    .flatten();
    assert!(edges.is_some());
    assert_eq!(edges.unwrap().len(), 1);
}

#[test]
fn apply_pending_causality_edges_drains_session() {
    fresh_session();
    // Pre-populate the session with pending edges for E1 and E2.
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.pending_causality_edges_mut()
            .entry(StableId::new("E1"))
            .or_insert_with(Vec::new)
            .push(CausalityEdge {
                edge_kind: CausalityEdgeKind::Definition,
                target_stable_id: "def1".to_string(),
            });
        sess.pending_causality_edges_mut()
            .entry(StableId::new("E2"))
            .or_insert_with(Vec::new)
            .push(CausalityEdge {
                edge_kind: CausalityEdgeKind::Instance,
                target_stable_id: "inst1".to_string(),
            });
    });

    // Set up the PREVIEW_PROVENANCE thread_local with matching entries.
    use editor_core::preview_inspector::PreviewProvenance;
    use editor_core::scene_asset::{AssetReference, LocalId};
    let mut prov: BTreeMap<editor_core::document::StableId, PreviewProvenance> = BTreeMap::new();
    prov.insert(
        editor_core::document::StableId::new("E1"),
        PreviewProvenance {
            stable_id: editor_core::document::StableId::new("E1"),
            local_id: LocalId::new("local1"),
            asset_ref: AssetReference::new("asset1"),
            components: vec![],
            is_from_instance: false,
            causality_edges: vec![],
        },
    );
    prov.insert(
        editor_core::document::StableId::new("E2"),
        PreviewProvenance {
            stable_id: editor_core::document::StableId::new("E2"),
            local_id: LocalId::new("local2"),
            asset_ref: AssetReference::new("asset2"),
            components: vec![],
            is_from_instance: false,
            causality_edges: vec![],
        },
    );
    editor_core::preview_inspector::set_provenance(prov);

    editor_core::preview_inspector::apply_pending_causality_edges();

    // Session pending edges are drained.
    let pending_count: Option<usize> =
        editor_model::ports::with_session_mut(|sess| sess.pending_causality_edges_mut().len());
    assert_eq!(
        pending_count,
        Some(0),
        "session pending edges should be drained"
    );

    // PROVENANCE has the edges.
    let e1_prov = editor_core::preview_inspector::get_provenance("E1").unwrap();
    assert_eq!(e1_prov.causality_edges.len(), 1);
    let e2_prov = editor_core::preview_inspector::get_provenance("E2").unwrap();
    assert_eq!(e2_prov.causality_edges.len(), 1);
}
