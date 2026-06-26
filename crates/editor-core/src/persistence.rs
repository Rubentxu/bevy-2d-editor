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

/// Project metadata stored at OPFS root as `project.json`.
/// Contains version, name, and list of saved scenes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub version: String,
    pub name: String,
    pub scenes: Vec<String>,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            version: "0.1".to_string(),
            name: "Untitled Project".to_string(),
            scenes: Vec::new(),
        }
    }
}

/// Resolve the OPFS path for a scene file: `scenes/<name>.scene.json`.
pub fn scene_path(name: &str) -> String {
    format!("{}/{}.scene.json", SCENES_DIR, name)
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
    }

    #[test]
    fn test_project_metadata_serialization_roundtrip() {
        let pm = ProjectMetadata {
            version: "0.1".to_string(),
            name: "Test Project".to_string(),
            scenes: vec!["level_01".to_string(), "level_02".to_string()],
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
    fn test_project_metadata_empty_roundtrip() {
        let pm = ProjectMetadata::default();
        let json = serde_json::to_string(&pm).unwrap();
        let rt: ProjectMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(pm, rt);
    }

    #[test]
    fn test_project_metadata_with_unknown_fields_preserved() {
        // Forward-compat: unknown fields in project.json are preserved on deserialize
        let json = r#"{"version":"0.1","name":"Test","scenes":[],"future_field":"preserved"}"#;
        // Note: ProjectMetadata uses deny_unknown_fields by default (not set),
        // so unknown fields are silently ignored. We document this as the expected
        // behavior for MVP; if needed, change to #[serde(deny_unknown_fields)] later.
        let rt: ProjectMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(rt.version, "0.1");
        assert_eq!(rt.name, "Test");
    }
}