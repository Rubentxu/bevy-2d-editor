//! Source Files module — CRUD for Rust source files in OPFS `sources/` directory.
//!
//! Per design.md §Decision: Source file schema — raw `.rs` text in OPFS `sources/`,
//! no JSON envelope. This is user-authored code text, not editor-owned structured
//! documents (scenes/assets/schemas), so the JSON-envelope pattern does not apply.
//!
//! The module holds an in-memory catalog of source files; OPFS holds raw `.rs` text.

use serde::{Deserialize, Serialize};

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

/// Resolve the OPFS path for a source file: `sources/<path>.rs`.
pub fn source_path(path: &str) -> String {
    format!("{}/{}.rs", SOURCES_DIR, path)
}

/// Resolve the OPFS path for a source file given its id.
/// The id IS the OPFS path without extension, so `src/main` → `sources/src/main.rs`.
pub fn source_path_from_id(id: &str) -> String {
    format!("{}/{}.rs", SOURCES_DIR, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_file_id_as_str() {
        let id = SourceFileId::new("src/lib");
        assert_eq!(id.as_str(), "src/lib");
    }

    #[test]
    fn source_path_format() {
        assert_eq!(source_path("src/main"), "sources/src/main.rs");
        assert_eq!(source_path("lib"), "sources/lib.rs");
    }

    #[test]
    fn test_source_path_from_id() {
        assert_eq!(source_path_from_id("src/main"), "sources/src/main.rs");
        assert_eq!(source_path_from_id("lib"), "sources/lib.rs");
    }

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
}
