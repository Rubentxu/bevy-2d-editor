//! Project metadata — the root document for editor project state.
//!
//! Stored at OPFS root as `project.json`. Contains version, name, list of
//! saved scenes, schemas, and catalogs for scene assets and world documents.

use serde::{Deserialize, Serialize};

use crate::scene_asset_catalog::SceneAssetCatalogEntry;
use crate::world::WorldCatalogEntry;

/// Project metadata stored at OPFS root as `project.json`.
/// Contains version, name, list of saved scenes, schemas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Format version string.
    pub version: String,
    /// Human-readable project name.
    pub name: String,
    /// List of scene names in this project.
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
