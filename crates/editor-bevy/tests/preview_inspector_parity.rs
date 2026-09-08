//! Parity tests for the session-owned preview inspector state (H2.5 Block E).
//!
//! These tests prove that the new `EditorSession.preview_inspector` fields
//! (reached via `editor_model::ports::with_session_mut`) behave identically
//! to the legacy thread-local `PREVIEW_METRICS` / `PREVIEW_MAPPING` /
//! `PREVIEW_PROVENANCE` that were retired in Block E.
//!
//! Coverage:
//! - `set_metrics` / `get_metrics` round-trip via session
//! - `set_mapping` / `get_mapping` round-trip via session
//! - `set_provenance` / `get_provenance` round-trip via session
//! - `increment_rebuild_count` increments via session
//! - Fallback path: when no session is installed, the renamed
//!   `PREVIEW_METRICS_FALLBACK` etc. thread_locals still serve reads.
//! - Two-session isolation: two `EditorSession`s do not leak state.

#[path = "support/mod.rs"]
mod support;

use editor_bevy::preview_inspector::{
    PreviewMappingEntry, PreviewMetrics, PreviewProvenance, get_mapping, get_metrics, get_provenance,
    increment_rebuild_count, set_mapping, set_metrics, set_provenance,
};
use editor_bevy::scene_asset::AssetReference;
use editor_model::EditorSessionPort;
use editor_model::ids::StableId;

fn fresh_session() {
    let session = support::FakeSessionWithDefaults(support::FakeSession::new());
    let arc = std::sync::Arc::new(std::sync::Mutex::new(session));
    editor_model::ports::register_editor_session(arc);
}

// §S-E-1: parity at metrics round-trip (session)
#[test]
fn parity_metrics_round_trip_via_session() {
    fresh_session();
    let m = PreviewMetrics {
        fps: 60.0,
        frame_time_ms: 16.6,
        rebuild_count: 5,
    };
    set_metrics(m.clone());
    let got = get_metrics();
    assert_eq!(got, m, "metrics round-trip via session must be lossless");
}

// §S-E-2: parity at mapping round-trip (session)
#[test]
fn parity_mapping_round_trip_via_session() {
    fresh_session();
    let entries = vec![
        PreviewMappingEntry {
            stable_id: StableId::new("inst_1"),
            local_id: editor_bevy::scene_asset::LocalId::new("root"),
            asset_ref: AssetReference::new("assets/test"),
            component_count: 3,
        },
        PreviewMappingEntry {
            stable_id: StableId::new("inst_2"),
            local_id: editor_bevy::scene_asset::LocalId::new("weapon"),
            asset_ref: AssetReference::new("assets/sword"),
            component_count: 1,
        },
    ];
    set_mapping(entries.clone());
    let got = get_mapping();
    assert_eq!(got.len(), entries.len(), "mapping count");
    assert_eq!(got[0].stable_id, entries[0].stable_id);
    assert_eq!(got[1].stable_id, entries[1].stable_id);
}

// §S-E-3: parity at provenance round-trip (session)
#[test]
fn parity_provenance_round_trip_via_session() {
    fresh_session();
    let mut entries = std::collections::BTreeMap::new();
    let key = StableId::new("inst_1");
    let value = PreviewProvenance {
        stable_id: key.clone(),
        local_id: editor_bevy::scene_asset::LocalId::new("root"),
        asset_ref: AssetReference::new("assets/test"),
        components: vec!["Sprite".to_string(), "Transform".to_string()],
        is_from_instance: true,
        causality_edges: vec![],
    };
    entries.insert(key.clone(), value.clone());
    set_provenance(entries);
    let got = get_provenance("inst_1");
    assert!(got.is_some(), "provenance for inst_1 must be present");
    let got = got.unwrap();
    assert_eq!(got.stable_id, value.stable_id);
    assert_eq!(got.components, value.components);
    assert!(got.is_from_instance);
}

// §S-E-4: increment_rebuild_count via session
#[test]
fn parity_increment_rebuild_count_via_session() {
    fresh_session();
    set_metrics(PreviewMetrics {
        fps: 0.0,
        frame_time_ms: 0.0,
        rebuild_count: 0,
    });
    let n1 = increment_rebuild_count();
    assert_eq!(n1, 1);
    let n2 = increment_rebuild_count();
    assert_eq!(n2, 2);
    let m = get_metrics();
    assert_eq!(m.rebuild_count, 2);
}

// §S-E-5: fallback path when no session is installed.
//
// We do NOT call fresh_session() here. The legacy `PREVIEW_METRICS_FALLBACK`
// thread_local must still serve reads so that pre-Block-E tests that don't
// install a session continue to work.
#[test]
fn parity_fallback_path_without_session() {
    // Do NOT install a session.
    set_metrics(PreviewMetrics {
        fps: 30.0,
        frame_time_ms: 33.3,
        rebuild_count: 7,
    });
    let got = get_metrics();
    assert_eq!(got.fps, 30.0);
    assert_eq!(got.frame_time_ms, 33.3);
    assert_eq!(got.rebuild_count, 7);

    let n = increment_rebuild_count();
    assert_eq!(n, 8);
}

// §S-E-6: two-session isolation.
//
// Two EditorSessions installed in succession must not leak state. We install
// session A, write metrics, then install session B (which should start
// empty per `Default` for PreviewInspectorState), and assert that B sees
// empty metrics (not A's values).
#[test]
fn parity_two_session_isolation() {
    // Session A.
    fresh_session();
    set_metrics(PreviewMetrics {
        fps: 60.0,
        frame_time_ms: 16.6,
        rebuild_count: 42,
    });
    let m_a = get_metrics();
    assert_eq!(m_a.rebuild_count, 42);

    // Session B (a fresh registration replaces the prior one).
    fresh_session();
    let m_b = get_metrics();
    assert_eq!(
        m_b.rebuild_count, 0,
        "new session must start with default metrics (rebuild_count=0)"
    );
    assert_eq!(
        m_b.fps, 0.0,
        "new session must start with default metrics (fps=0)"
    );
}
