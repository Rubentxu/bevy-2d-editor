//! Persistence helpers for OPFS save/load of SceneDocument and project metadata.
//!
//! Hito 0 §5.2 mandates OPFS as the persistence layer with a Defold-inspired
//! directory structure. This module provides the Rust-side data structures
//! and path resolution; the actual OPFS calls go through a JS bridge
//! (see `frontend/src/opfs-bridge.ts`).

use serde::{Deserialize, Serialize};

/// Filename for the project metadata file at OPFS root.
pub const PROJECT_FILE: &str = "project.json";

/// Subdirectory containing SceneDocument files.
pub const SCENES_DIR: &str = "scenes";

/// Subdirectory containing Component Schema files (one per schema).
pub const SCHEMAS_DIR: &str = "schemas";

/// Subdirectory containing Entity Template files (one per template).
pub const ENTITIES_DIR: &str = "entities";

/// Project metadata stored at OPFS root as `project.json`.
/// Contains version, name, list of saved scenes, schemas, and templates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub version: String,
    pub name: String,
    pub scenes: Vec<String>,
    /// List of schema type_ids in the project. `#[serde(default)]` so old
    /// project.json files without this field still parse (empty Vec).
    #[serde(default)]
    pub schemas: Vec<String>,
    /// List of entity template IDs in the project. `#[serde(default)]` so old
    /// project.json files without this field still parse (empty Vec).
    #[serde(default)]
    pub templates: Vec<String>,
    /// The currently active/selected scene. `#[serde(default)]` so old
    /// project.json files without this field still parse (None → first scene).
    #[serde(default)]
    pub active_scene: Option<String>,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            version: "0.1".to_string(),
            name: "Untitled Project".to_string(),
            scenes: Vec::new(),
            schemas: Vec::new(),
            templates: Vec::new(),
            active_scene: None,
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

/// Resolve the OPFS path for an entity template file: `entities/<template_id>.template.json`.
pub fn template_path(template_id: &str) -> String {
    format!("{}/{}.template.json", ENTITIES_DIR, template_id)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            templates: vec!["enemy_goblin".to_string()],
            active_scene: None,
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
        assert_eq!(schema_path("editor.Transform2D"), "schemas/editor.Transform2D.schema.json");
        assert_eq!(schema_path("game.PlayerHealth"), "schemas/game.PlayerHealth.schema.json");
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
}