//! Asset Files module — CRUD for binary texture assets in OPFS `resources/` directory.
//!
//! Per design.md §Decision: Asset files are binary blobs stored directly in OPFS
//! `resources/` directory. There is no in-memory catalog — `list_asset_files` reads
//! OPFS on every call, mirroring `list_source_files` behavior.

use serde::{Deserialize, Serialize};

/// Subdirectory containing imported texture assets.
pub const RESOURCE_DIR: &str = "resources";

/// Supported image MIME types for texture assets.
const SUPPORTED_MIMES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/svg+xml",
];

/// Kind of asset file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetFileKind {
    Texture,
    // Extensible: Audio, Font, Video, etc.
}

/// Opaque stable identity for an asset file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssetFileId(pub String);

impl AssetFileId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An asset file entry (texture or other binary asset).
///
/// `id` is the opaque stable identifier used for API calls.
/// `path` is the OPFS path relative to the resources directory (e.g., `player.png`).
/// `name` is the human-facing file name (e.g., `player.png`).
/// `kind` categorizes the asset (e.g., Texture).
/// `mime_type` is the MIME type (e.g., `image/png`).
/// `size_bytes` is the file size in bytes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetFile {
    pub id: AssetFileId,
    pub path: String,
    pub name: String,
    pub kind: AssetFileKind,
    pub mime_type: String,
    pub size_bytes: u64,
}

/// Resolve the OPFS path for an asset file given its id.
/// The id IS the OPFS path, so `player.png` → `resources/player.png`.
pub fn asset_file_path_from_id(id: &str) -> String {
    format!("{}/{}", RESOURCE_DIR, id)
}

/// Check if a MIME type is supported for texture assets.
pub fn is_supported_mime(mime: &str) -> bool {
    SUPPORTED_MIMES.contains(&mime)
}

#[cfg(test)]
mod tests {
    use super::*;

    // §1.2: AssetFileId identity and ordering
    #[test]
    fn asset_file_id_as_str() {
        let id = AssetFileId::new("player.png");
        assert_eq!(id.as_str(), "player.png");
        assert_eq!(id.as_str(), "player.png"); // idempotent
    }

    #[test]
    fn asset_file_id_equality() {
        let id1 = AssetFileId::new("hero.png");
        let id2 = AssetFileId::new("hero.png");
        let id3 = AssetFileId::new("enemy.png");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn asset_file_id_ord() {
        let id1 = AssetFileId::new("a.png");
        let id2 = AssetFileId::new("b.png");
        assert!(id1 < id2);
        assert!(id2 > id1);
    }

    // §1.2: Path derivation
    #[test]
    fn test_asset_file_path_from_id() {
        assert_eq!(asset_file_path_from_id("player.png"), "resources/player.png");
        assert_eq!(asset_file_path_from_id("characters/hero.png"), "resources/characters/hero.png");
        assert_eq!(asset_file_path_from_id(""), "resources/");
    }

    // §1.2: MIME validation
    #[test]
    fn test_is_supported_mime_accepts_valid() {
        assert!(is_supported_mime("image/png"));
        assert!(is_supported_mime("image/jpeg"));
        assert!(is_supported_mime("image/gif"));
        assert!(is_supported_mime("image/webp"));
        assert!(is_supported_mime("image/svg+xml"));
    }

    #[test]
    fn test_is_supported_mime_rejects_invalid() {
        assert!(!is_supported_mime("application/zip"));
        assert!(!is_supported_mime("text/plain"));
        assert!(!is_supported_mime("audio/mpeg"));
        assert!(!is_supported_mime("video/mp4"));
        assert!(!is_supported_mime("image/tiff"));
    }

    // §1.2: Resource directory constant
    #[test]
    fn resource_dir_constant() {
        assert_eq!(RESOURCE_DIR, "resources");
    }

    // §1.2: AssetFile serde roundtrip
    #[test]
    fn asset_file_serde_roundtrip() {
        let file = AssetFile {
            id: AssetFileId::new("hero.png"),
            path: "hero.png".to_string(),
            name: "hero.png".to_string(),
            kind: AssetFileKind::Texture,
            mime_type: "image/png".to_string(),
            size_bytes: 4096,
        };
        let json = serde_json::to_string(&file).unwrap();
        let parsed: AssetFile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, file);
    }

    #[test]
    fn asset_file_clone() {
        let file = AssetFile {
            id: AssetFileId::new("hero.png"),
            path: "hero.png".to_string(),
            name: "hero.png".to_string(),
            kind: AssetFileKind::Texture,
            mime_type: "image/png".to_string(),
            size_bytes: 4096,
        };
        let cloned = file.clone();
        assert_eq!(cloned, file);
        assert_eq!(cloned.id, file.id);
        assert_eq!(cloned.path, file.path);
        assert_eq!(cloned.name, file.name);
        assert_eq!(cloned.mime_type, file.mime_type);
        assert_eq!(cloned.size_bytes, file.size_bytes);
    }

    #[test]
    fn asset_file_id_is_transparent_serde() {
        // AssetFileId is #[serde(transparent)] so serializes as the inner String.
        let id = AssetFileId::new("textures/player.png");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"textures/player.png\"");
        let parsed: AssetFileId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn asset_file_debug() {
        let file = AssetFile {
            id: AssetFileId::new("hero.png"),
            path: "hero.png".to_string(),
            name: "hero.png".to_string(),
            kind: AssetFileKind::Texture,
            mime_type: "image/png".to_string(),
            size_bytes: 4096,
        };
        let debug = format!("{:?}", file);
        assert!(debug.contains("AssetFile"));
        assert!(debug.contains("hero.png"));
    }

    #[test]
    fn asset_file_kind_serde() {
        let kind = AssetFileKind::Texture;
        let json = serde_json::to_string(&kind).unwrap();
        let parsed: AssetFileKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, kind);
    }
}
