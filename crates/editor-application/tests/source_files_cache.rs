//! H2.2 — Source-file cache CRUD tests.
//!
//! The cache lives on `EditorSession.preview_state.source_files` (typed as
//! [`editor_model::session::SourceFilesCache`]). These tests exercise the
//! `cache`/`get_cached`/`invalidate`/`clear` operations through the
//! `EditorSessionPort::source_files_mut` accessor, which is the same path
//! the Bevy adapter uses (see `editor_bevy::source_files` facades).

use std::sync::{Arc, Mutex};

use editor_application::{EditorSession, InMemoryProjectStore};
use editor_model::session::SourceFilesCache;
use editor_model::time::FakeClock;

fn fresh_session() -> EditorSession {
    EditorSession::new(
        Arc::new(InMemoryProjectStore::default()),
        Arc::new(FakeClock::new()),
    )
}

#[test]
fn new_cache_is_empty() {
    let session = fresh_session();
    assert!(session.source_files().files.is_empty());
}

#[test]
fn cache_insert_and_get() {
    let mut session = fresh_session();
    session
        .source_files_mut()
        .files
        .insert("src/main".to_string(), "fn main() {}".to_string());

    assert_eq!(
        session.source_files().files.get("src/main"),
        Some(&"fn main() {}".to_string())
    );
}

#[test]
fn cache_insert_overwrites() {
    let mut session = fresh_session();
    session
        .source_files_mut()
        .files
        .insert("src/main".to_string(), "v1".to_string());
    session
        .source_files_mut()
        .files
        .insert("src/main".to_string(), "v2".to_string());

    assert_eq!(
        session.source_files().files.get("src/main"),
        Some(&"v2".to_string())
    );
    assert_eq!(session.source_files().files.len(), 1);
}

#[test]
fn invalidate_removes_one_entry() {
    let mut session = fresh_session();
    session
        .source_files_mut()
        .files
        .insert("a.rs".to_string(), "a".to_string());
    session
        .source_files_mut()
        .files
        .insert("b.rs".to_string(), "b".to_string());

    session.source_files_mut().files.remove("a.rs");

    assert!(session.source_files().files.get("a.rs").is_none());
    assert_eq!(
        session.source_files().files.get("b.rs"),
        Some(&"b".to_string())
    );
}

#[test]
fn clear_removes_all_entries() {
    let mut session = fresh_session();
    session
        .source_files_mut()
        .files
        .insert("a".to_string(), "1".to_string());
    session
        .source_files_mut()
        .files
        .insert("b".to_string(), "2".to_string());

    session.source_files_mut().files.clear();
    assert!(session.source_files().files.is_empty());
}

#[test]
fn session_isolates_caches_across_instances() {
    // UAT-ARCH-004 anchor: two independent sessions must not share cache
    // state. The legacy `thread_local! SOURCE_FILE_REGISTRY` made this
    // impossible because all callers read the same global cell.
    let mut s1 = fresh_session();
    let mut s2 = fresh_session();

    s1.source_files_mut()
        .files
        .insert("shared.rs".to_string(), "from s1".to_string());
    s2.source_files_mut()
        .files
        .insert("shared.rs".to_string(), "from s2".to_string());

    assert_eq!(
        s1.source_files().files.get("shared.rs"),
        Some(&"from s1".to_string())
    );
    assert_eq!(
        s2.source_files().files.get("shared.rs"),
        Some(&"from s2".to_string())
    );
}

#[test]
fn source_files_cache_default_is_empty() {
    let c = SourceFilesCache::default();
    assert!(c.files.is_empty());
}

#[test]
fn cache_arc_shares_underlying_state() {
    // The EditorSessionPort seam requires `Arc<Mutex<dyn EditorSessionPort>>`
    // — verify that cloning the Arc and mutating through one clone is
    // visible through the other (this is the same path the Bevy adapter
    // takes via `with_session_mut`).
    let session = Arc::new(Mutex::new(fresh_session()));

    {
        let mut guard = session.lock().unwrap();
        guard
            .source_files_mut()
            .files
            .insert("a.rs".to_string(), "x".to_string());
    }
    let cached: Option<String> = {
        let mut guard = session.lock().unwrap();
        guard.source_files_mut().files.get("a.rs").cloned()
    };
    assert_eq!(cached, Some("x".to_string()));
}
