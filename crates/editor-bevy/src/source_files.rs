//! Source Files module — CRUD for Rust source files in OPFS `sources/` directory.
//!
//! Per design.md §Decision: Source file schema — raw `.rs` text in OPFS `sources/`,
//! no JSON envelope. This is user-authored code text, not editor-owned structured
//! documents (scenes/assets/schemas), so the JSON-envelope pattern does not apply.
//!
//! H2.2: the in-memory source-file cache no longer lives in this module. The
//! canonical cache lives on `EditorSession.preview_state.source_files`
//! (typed as [`editor_model::session::SourceFilesCache`]) and is exposed to
//! Bevy systems through the `EditorSessionPort::source_files_mut` accessor
//! in [`editor_model::session_port`]. This module provides **facades** that
//! delegate to that canonical cache; the Bevy adapter cannot import
//! `editor-application` directly (H1.4), so the `EditorSessionPort` trait is
//! the sanctioned seam.
//!
//! The cache is invalidated when a hot-reload Source request is processed.

use serde::{Deserialize, Serialize};

/// Cache source file content (keyed by file_id, e.g. "a.rs").
///
/// Facade over `EditorSessionPort::source_files_mut` via
/// [`editor_model::ports::with_session_mut`]. No-op if the session has
/// not been registered (e.g. in unit tests without a session).
pub fn cache_source(file_id: &str, content: &str) {
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.source_files_mut()
            .files
            .insert(file_id.to_string(), content.to_string());
    });
}

/// Get cached source content, if present.
pub fn get_cached_source(file_id: &str) -> Option<String> {
    editor_model::ports::with_session_mut(|sess| {
        sess.source_files_mut().files.get(file_id).cloned()
    })
    .flatten()
}

/// Invalidate (remove) a single source file from the cache.
pub fn invalidate_cache(file_id: &str) {
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.source_files_mut().files.remove(file_id);
    });
}

/// Clear the entire source file cache (used by ForceReloadAll).
pub fn clear_cache() {
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.source_files_mut().files.clear();
    });
}

/// Subdirectory containing Rust source files.
pub const SOURCES_DIR: &str = "sources";

/// Opaque stable identity for a source file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceFileId(pub String);

impl SourceFileId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A Rust source file entry.
///
/// `id` is the opaque stable identifier used for API calls.
/// `path` is the OPFS path relative to the sources directory (e.g., `src/main.rs`).
/// `name` is the human-facing file name (e.g., `main.rs`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceFile {
    pub id: SourceFileId,
    pub path: String,
    pub name: String,
}

/// Resolve the OPFS path for a source file given its id.
/// The id IS the OPFS path without extension, so `src/main` → `sources/src/main.rs`.
pub fn source_path_from_id(id: &str) -> String {
    format!("{}/{}.rs", SOURCES_DIR, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    // §1.1: SourceFileId identity and ordering
    #[test]
    fn source_file_id_as_str() {
        let id = SourceFileId::new("src/lib");
        assert_eq!(id.as_str(), "src/lib");
        assert_eq!(id.as_str(), "src/lib"); // idempotent
    }

    #[test]
    fn source_file_id_equality() {
        let id1 = SourceFileId::new("src/main");
        let id2 = SourceFileId::new("src/main");
        let id3 = SourceFileId::new("lib");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn source_file_id_ord() {
        let id1 = SourceFileId::new("a");
        let id2 = SourceFileId::new("b");
        assert!(id1 < id2);
        assert!(id2 > id1);
    }

    // §1.2: Path derivation
    #[test]
    fn test_source_path_from_id() {
        assert_eq!(source_path_from_id("src/main"), "sources/src/main.rs");
        assert_eq!(source_path_from_id("lib"), "sources/lib.rs");
        assert_eq!(source_path_from_id(""), "sources/.rs"); // empty id edge case
        assert_eq!(source_path_from_id("src/a/b/c"), "sources/src/a/b/c.rs");
    }

    #[test]
    fn sources_dir_constant() {
        assert_eq!(SOURCES_DIR, "sources");
    }

    // §1.3: SourceFile struct and serde
    #[test]
    fn source_file_serde_roundtrip() {
        let file = SourceFile {
            id: SourceFileId::new("src/main"),
            path: "src/main".to_string(),
            name: "main.rs".to_string(),
        };
        let json = serde_json::to_string(&file).unwrap();
        let parsed: SourceFile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, file);
    }

    #[test]
    fn source_file_clone() {
        let file = SourceFile {
            id: SourceFileId::new("src/main"),
            path: "src/main".to_string(),
            name: "main.rs".to_string(),
        };
        let cloned = file.clone();
        assert_eq!(cloned, file);
        assert_eq!(cloned.id, file.id);
        assert_eq!(cloned.path, file.path);
        assert_eq!(cloned.name, file.name);
    }

    #[test]
    fn source_file_id_is_transparent_serde() {
        // SourceFileId is #[serde(transparent)] so serializes as the inner String.
        let id = SourceFileId::new("my/file");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"my/file\"");
        let parsed: SourceFileId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, id);
    }

    // §1.4: CRUD invariants (pure functions only)
    // Note: Full create/list/read/write/delete round-trip tests require
    // a WASM+OPFS mock harness (js_* externs). Those live in integration tests
    // that compile to wasm32-unknown-unknown and run in a browser context.
    // Per-task §4.1 scope: unit tests for the pure module surface.

    #[test]
    fn source_file_debug() {
        let file = SourceFile {
            id: SourceFileId::new("src/main"),
            path: "src/main".to_string(),
            name: "main.rs".to_string(),
        };
        let debug = format!("{:?}", file);
        assert!(debug.contains("SourceFile"));
        assert!(debug.contains("src/main"));
    }

    // §1.3: SourceFileRegistry cache API tests
    //
    // H2.2 NOTE: the cache now lives on `EditorSession.preview_state.source_files`
    // (typed as `editor_model::session::SourceFilesCache`) and is accessed
    // through the `EditorSessionPort::source_files_mut` trait method via
    // `editor_model::ports::with_session_mut`. The Bevy adapter cannot
    // import `editor-application` directly (H1.4), so a unit test in this
    // crate would need to construct a mock `EditorSessionPort` — the
    // canonical cache CRUD test lives in
    // `crates/editor-application/tests/source_files_cache.rs` where the
    // concrete `EditorSession` is reachable. The end-to-end facade test
    // here requires the mock session-port infra that lands with the
    // H2.6 mock harness; until then this test is ignored.
    #[test]
    #[ignore = "H2.2: cache CRUD moved to editor-application (see editor-application/tests/source_files_cache.rs); end-to-end facade test lands with the H2.6 mock harness"]
    fn invalidate_source_cache_clears_only_target() {
        // Register two sources
        cache_source("a.rs", "content a");
        cache_source("b.rs", "content b");

        // Verify both are cached
        assert_eq!(get_cached_source("a.rs"), Some("content a".to_string()));
        assert_eq!(get_cached_source("b.rs"), Some("content b".to_string()));

        // Invalidate only "a.rs"
        invalidate_cache("a.rs");

        // "a.rs" should be gone, "b.rs" should survive
        assert!(
            get_cached_source("a.rs").is_none(),
            "a.rs should be invalidated"
        );
        assert_eq!(
            get_cached_source("b.rs"),
            Some("content b".to_string()),
            "b.rs should survive"
        );
    }
}
