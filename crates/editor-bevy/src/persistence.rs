//! Persistence helpers for OPFS save/load of SceneDocument and project metadata.
//!
//! Hito 0 §5.2 mandates OPFS as the persistence layer with a Defold-inspired
//! directory structure. This module provides the Rust-side data structures
//! and path resolution; the actual OPFS calls go through a JS bridge
//! (see `frontend/src/opfs-bridge.ts`).
//!
//! ## ADRs integrated here
//! - [ADR-0005](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md):
//!   Scene Asset identity (`asset_id` + `logical_path`), roles, versioning.
//! - [ADR-0006](../../adr/0006-authoring-first-roadmap-after-bsn-migration.md):
//!   editor-owned source of truth; `.bsn` write-back deferred.
//! - [ADR-0007](../../adr/0007-separate-asset-command-surface.md):
//!   separate `AssetCommand` surface for authoring mutations.
//! - [ADR-0008](../../adr/0008-path-based-scene-asset-opfs-layout.md):
//!   `assets/<logical_path>.asset.json` path layout; catalog in `ProjectMetadata`.

use serde::{Deserialize, Serialize};

use editor_model::WorldCatalogEntry;
use editor_model::scene_asset_catalog::SceneAssetCatalogEntry;

/// Filename for the project metadata file at OPFS root.
pub const PROJECT_FILE: &str = "project.json";

/// Subdirectory containing SceneDocument files.
pub const SCENES_DIR: &str = "scenes";

/// Subdirectory containing Component Schema files (one per schema).
pub const SCHEMAS_DIR: &str = "schemas";

/// Subdirectory containing SceneAssetDocument bodies (ADR-0008 §Decision).
pub const ASSETS_DIR: &str = "assets";

/// Subdirectory containing Tileset body files.
pub const TILESETS_DIR: &str = "tilesets";

/// Subdirectory containing LogicGraphAsset bodies (parallel to ASSETS_DIR for scene assets).
pub const LOGIC_GRAPHS_DIR: &str = "logic_graphs";

/// Subdirectory containing WorldDocument bodies (ADR-0037).
pub const WORLDS_DIR: &str = "worlds";

// RESOURCE_DIR is defined in `asset_files.rs` (the canonical location used by
// the asset pipeline). The duplicate here was unused; left only the active
// constant above.

/// Project metadata stored at OPFS root as `project.json`.
/// Contains version, name, list of saved scenes, schemas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub version: String,
    pub name: String,
    pub scenes: Vec<String>,
    /// List of schema type_ids in the project. `#[serde(default)]` so old
    /// project.json files without this field still parse (empty Vec).
    #[serde(default)]
    pub schemas: Vec<String>,
    /// The currently active/selected scene. `#[serde(default)]` so old
    /// project.json files without this field still parse (None → first scene).
    #[serde(default)]
    pub active_scene: Option<String>,
    /// Catalog of Scene Assets in this project. `#[serde(default)]` so old
    /// project.json files without this field still parse (empty Vec).
    /// See ADR-0008 §Decision rule 2.
    #[serde(default)]
    pub scene_assets: Vec<SceneAssetCatalogEntry>,
    /// Catalog of World Documents in this project (ADR-0037). `#[serde(default)]`
    /// so old project.json files without this field still parse (empty Vec).
    #[serde(default)]
    pub worlds: Vec<WorldCatalogEntry>,
    /// The currently active world. `#[serde(default)]` so old project.json
    /// files without this field still parse (None).
    #[serde(default)]
    pub active_world: Option<String>,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            version: "0.1".to_string(),
            name: "Untitled Project".to_string(),
            scenes: Vec::new(),
            schemas: Vec::new(),
            active_scene: None,
            scene_assets: Vec::new(),
            worlds: Vec::new(),
            active_world: None,
        }
    }
}

/// Resolve the OPFS path for a scene file: `scenes/<name>.scene.json`.
pub fn scene_path(name: &str) -> String {
    format!("{}/{}.scene.json", SCENES_DIR, name)
}

/// Resolve the OPFS path for a schema file: `schemas/<type_id>.schema.json`.
/// `type_id` may contain dots (e.g., `editor.Transform2D`); OPFS file names
/// accept dots.
pub fn schema_path(type_id: &str) -> String {
    format!("{}/{}.schema.json", SCHEMAS_DIR, type_id)
}

/// Resolve the OPFS path for a Scene Asset body: `assets/<logical_path>.asset.json`.
/// `logical_path` MUST be already-normalized (segments joined by '/').
/// See ADR-0008 §Decision rule 1.
pub fn asset_path(logical_path: &str) -> String {
    format!("{}/{}.asset.json", ASSETS_DIR, logical_path)
}

/// Resolve the OPFS path for a Tileset body: `tilesets/<id>.tileset.json`.
/// `<id>` is the TilesetId string (already opaque and escaped).
pub fn tileset_path(id: &str) -> String {
    format!("{}/{}.tileset.json", TILESETS_DIR, id)
}

/// Resolve the OPFS path for a LogicGraphAsset body: `logic_graphs/<logical_path>.logic.json`.
/// `logical_path` MUST be already-normalized (segments joined by '/').
pub fn logic_graph_path(logical_path: &str) -> String {
    format!("{}/{}.logic.json", LOGIC_GRAPHS_DIR, logical_path)
}

/// Resolve the OPFS path for a WorldDocument body: `worlds/<logical_path>.world.json`.
pub fn world_path(logical_path: &str) -> String {
    format!("{}/{}.world.json", WORLDS_DIR, logical_path)
}

/// Error type for asset path validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AssetPathError {
    #[error("asset path is empty")]
    Empty,
    #[error("path traversal not allowed: {0}")]
    PathTraversal(String),
}

/// Validate an asset logical path.
///
/// Returns `Ok(())` if the path is valid, or an `AssetPathError` if not.
/// A valid path:
/// - Is not empty (after trimming whitespace)
/// - Does not contain `..` or `.` path segments (no path traversal)
///
/// This is a security and correctness check per ADR-0008 §Decision rule 1.
/// Use this before calling `asset_path()` to give early feedback on invalid input.
pub fn validate_logical_path(s: &str) -> Result<(), AssetPathError> {
    if s.trim().is_empty() {
        return Err(AssetPathError::Empty);
    }
    let segments: Vec<&str> = s.split('/').collect();
    for seg in segments {
        if seg == ".." {
            return Err(AssetPathError::PathTraversal(
                "'..' segment not allowed".to_string(),
            ));
        }
        if seg == "." {
            return Err(AssetPathError::PathTraversal(
                "'.' segment not allowed".to_string(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_files::RESOURCE_DIR;

    #[test]
    fn test_resource_dir_constant() {
        // §1.1: RESOURCE_DIR must be "resources" for asset file storage
        // (canonical definition now lives in asset_files.rs)
        assert_eq!(RESOURCE_DIR, "resources");
    }

    #[test]
    fn test_project_metadata_default() {
        let pm = ProjectMetadata::default();
        assert_eq!(pm.version, "0.1");
        assert_eq!(pm.name, "Untitled Project");
        assert!(pm.scenes.is_empty());
        assert!(pm.schemas.is_empty());
    }

    #[test]
    fn test_project_metadata_serialization_roundtrip() {
        let pm = ProjectMetadata {
            version: "0.1".to_string(),
            name: "Test Project".to_string(),
            scenes: vec!["level_01".to_string(), "level_02".to_string()],
            schemas: vec!["game.PlayerHealth".to_string()],
            active_scene: None,
            scene_assets: vec![],
            worlds: vec![],
            active_world: None,
        };
        let json = serde_json::to_string(&pm).unwrap();
        let rt: ProjectMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(pm, rt);
    }

    #[test]
    fn test_scene_path_format() {
        assert_eq!(scene_path("level_01"), "scenes/level_01.scene.json");
        assert_eq!(scene_path("boss_room"), "scenes/boss_room.scene.json");
    }

    #[test]
    fn test_schema_path_format() {
        assert_eq!(
            schema_path("editor.Transform2D"),
            "schemas/editor.Transform2D.schema.json"
        );
        assert_eq!(
            schema_path("game.PlayerHealth"),
            "schemas/game.PlayerHealth.schema.json"
        );
    }

    #[test]
    fn test_project_metadata_without_schemas_field_deserializes() {
        // Backward compat: old project.json files without schemas field still parse
        let json = r#"{"version":"0.1","name":"Old","scenes":["s1"]}"#;
        let pm: ProjectMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(pm.name, "Old");
        assert_eq!(pm.scenes, vec!["s1".to_string()]);
        assert!(pm.schemas.is_empty()); // default to empty
    }

    #[test]
    fn test_project_metadata_empty_roundtrip() {
        let pm = ProjectMetadata::default();
        let json = serde_json::to_string(&pm).unwrap();
        let rt: ProjectMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(pm, rt);
    }

    #[test]
    fn test_project_metadata_with_unknown_fields_preserved() {
        // Forward-compat: unknown fields in project.json are preserved on deserialize
        let json = r#"{"version":"0.1","name":"Test","scenes":[],"schemas":[],"future_field":"preserved"}"#;
        // Note: ProjectMetadata uses deny_unknown_fields by default (not set),
        // so unknown fields are silently ignored. We document this as the expected
        // behavior for MVP; if needed, change to #[serde(deny_unknown_fields)] later.
        let rt: ProjectMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(rt.version, "0.1");
        assert_eq!(rt.name, "Test");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // S18 RED — asset_path produces the expected OPFS path
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_asset_path_produces_expected_format() {
        // RED phase: this test captures the expected contract from ADR-0008 §Decision.
        // GIVEN logical_path = "characters/player"
        // WHEN asset_path(logical_path) is called
        // THEN the result equals "assets/characters/player.asset.json"
        assert_eq!(
            asset_path("characters/player"),
            "assets/characters/player.asset.json"
        );
    }

    #[test]
    fn test_asset_path_simple_name() {
        assert_eq!(asset_path("player"), "assets/player.asset.json");
    }

    #[test]
    fn test_asset_path_nested() {
        assert_eq!(
            asset_path("characters/player"),
            "assets/characters/player.asset.json"
        );
        assert_eq!(
            asset_path("ui/menus/title_screen"),
            "assets/ui/menus/title_screen.asset.json"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // S17 RED — ProjectMetadata with old shape still loads (back-compat)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_project_metadata_without_scene_assets_field_deserializes() {
        // RED phase: old project.json files without scene_assets field must parse.
        // GIVEN a project.json written before this change (no scene_assets field)
        // WHEN load_project parses it
        // THEN parsing succeeds AND scene_assets defaults to empty Vec
        // AND no warning is emitted for the missing field.
        let json = r#"{"version":"0.1","name":"Old Project","scenes":["main"]}"#;
        let pm: ProjectMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(pm.name, "Old Project");
        assert_eq!(pm.scenes, vec!["main"]);
        assert!(pm.scene_assets.is_empty()); // default to empty Vec
    }

    #[test]
    fn test_project_metadata_with_empty_scene_assets_roundtrip() {
        let pm = ProjectMetadata {
            version: "0.1".to_string(),
            name: "Test Project".to_string(),
            scenes: vec!["level_01".to_string()],
            schemas: vec![],
            active_scene: None,
            scene_assets: vec![],
            worlds: vec![],
            active_world: None,
        };
        let json = serde_json::to_string(&pm).unwrap();
        let rt: ProjectMetadata = serde_json::from_str(&json).unwrap();
        assert!(rt.scene_assets.is_empty());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // validate_logical_path tests (PR1 debt, Engram #3351)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_validate_logical_path_empty_string() {
        let result = validate_logical_path("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AssetPathError::Empty));
    }

    #[test]
    fn test_validate_logical_path_whitespace_only() {
        let result = validate_logical_path("   ");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AssetPathError::Empty));
    }

    #[test]
    fn test_validate_logical_path_valid_simple() {
        validate_logical_path("player").expect("simple name should be valid");
        validate_logical_path("assets/player").expect("nested path should be valid");
        validate_logical_path("a/b/c").expect("multi-segment should be valid");
    }

    #[test]
    fn test_validate_logical_path_double_dot() {
        let result = validate_logical_path("foo/../bar");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AssetPathError::PathTraversal(ref s) if s.contains("..")));
    }

    #[test]
    fn test_validate_logical_path_single_dot() {
        let result = validate_logical_path("foo/./bar");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AssetPathError::PathTraversal(ref s) if s.contains(".")));
    }

    #[test]
    fn test_asset_path_error_display() {
        let empty = AssetPathError::Empty;
        assert_eq!(empty.to_string(), "asset path is empty");

        let traversal = AssetPathError::PathTraversal("'..' segment not allowed".to_string());
        assert!(traversal.to_string().contains("path traversal"));
    }

    #[test]
    fn test_logic_graph_path_simple() {
        assert_eq!(logic_graph_path("jump"), "logic_graphs/jump.logic.json");
    }

    #[test]
    fn test_logic_graph_path_nested() {
        assert_eq!(
            logic_graph_path("platformer/jump"),
            "logic_graphs/platformer/jump.logic.json"
        );
    }
}
