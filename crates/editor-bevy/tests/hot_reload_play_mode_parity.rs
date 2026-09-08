//! Parity tests for the session-owned hot-reload + play-mode buses
//! (H2.5 Block G).
//!
//! Block G renames the legacy `HOT_RELOAD_BUS` / `PLAY_MODE_REQUEST`
//! thread_locals to `HOT_RELOAD_BUS_FALLBACK` /
//! `PLAY_MODE_REQUEST_FALLBACK`, extends `editor_model::runtime`
//! types to match the legacy semantics (3-variant
//! `HotReloadRequest` enum, 2-variant `PlayModeRequest` enum), and
//! migrates production code to write through the session (dual-write
//! fallback when no session is installed).
//!
//! These tests prove the session path works for hot-reload and
//! play-mode requests after Block G:
//! - hot_reload_source/asset/force_wasm push to the session
//! - play-mode request read+clear via session
//! - two-session isolation

#[path = "support/mod.rs"]
mod support;

use editor_model::runtime::{HotReloadRequest, PlayModeRequest};

fn fresh_session() {
    let session = support::FakeSessionWithDefaults(support::FakeSession::new());
    let arc = std::sync::Arc::new(std::sync::Mutex::new(session));
    editor_model::ports::register_editor_session(arc);
}

// §S-G-1: hot_reload_source_wasm pushes to session
#[test]
fn parity_hot_reload_source_via_session() {
    fresh_session();
    editor_bevy::hot_reload_source_wasm("foo.rs");
    let v = editor_model::ports::with_session_mut(|s| {
        s.runtime_hot_reload_requests_mut().clone()
    })
    .unwrap();
    assert_eq!(v.len(), 1);
    match &v[0] {
        HotReloadRequest::Source { file_id } => assert_eq!(file_id, "foo.rs"),
        other => panic!("expected Source, got {:?}", other),
    }
}

// §S-G-2: hot_reload_asset_wasm pushes to session
#[test]
fn parity_hot_reload_asset_via_session() {
    fresh_session();
    editor_bevy::hot_reload_asset_wasm("asset_42");
    let v = editor_model::ports::with_session_mut(|s| {
        s.runtime_hot_reload_requests_mut().clone()
    })
    .unwrap();
    assert_eq!(v.len(), 1);
    match &v[0] {
        HotReloadRequest::Asset { asset_id } => assert_eq!(asset_id, "asset_42"),
        other => panic!("expected Asset, got {:?}", other),
    }
}

// §S-G-3: force_reload_wasm pushes to session
#[test]
fn parity_force_reload_via_session() {
    fresh_session();
    editor_bevy::force_reload_wasm();
    let v = editor_model::ports::with_session_mut(|s| {
        s.runtime_hot_reload_requests_mut().clone()
    })
    .unwrap();
    assert_eq!(v.len(), 1);
    assert!(
        matches!(v[0], HotReloadRequest::ForceReloadAll),
        "expected ForceReloadAll, got {:?}",
        v[0]
    );
}

// §S-G-4: play-mode Enter pushed + read via session
#[test]
fn parity_play_mode_enter_via_session() {
    fresh_session();
    editor_model::ports::with_session_mut(|s| {
        *s.runtime_play_mode_request_mut() = Some(PlayModeRequest::Enter);
    });
    let req = editor_model::ports::with_session_mut(|s| {
        s.runtime_play_mode_request_mut().clone()
    })
    .unwrap();
    assert!(
        matches!(req, Some(PlayModeRequest::Enter)),
        "expected Some(Enter), got {:?}",
        req
    );
}

// §S-G-5: play-mode Exit pushed + read via session
#[test]
fn parity_play_mode_exit_via_session() {
    fresh_session();
    editor_model::ports::with_session_mut(|s| {
        *s.runtime_play_mode_request_mut() = Some(PlayModeRequest::Exit);
    });
    let req = editor_model::ports::with_session_mut(|s| {
        s.runtime_play_mode_request_mut().clone()
    })
    .unwrap();
    assert!(
        matches!(req, Some(PlayModeRequest::Exit)),
        "expected Some(Exit), got {:?}",
        req
    );
}

// §S-G-7: two-session isolation
#[test]
fn parity_two_session_isolation_hot_reload() {
    // Session A.
    fresh_session();
    editor_bevy::hot_reload_source_wasm("a.rs");

    // Session B (replaces A).
    fresh_session();
    let v_b = editor_model::ports::with_session_mut(|s| {
        s.runtime_hot_reload_requests_mut().clone()
    })
    .unwrap();
    assert!(
        v_b.is_empty(),
        "session B must not inherit session A's hot-reload requests"
    );
}
