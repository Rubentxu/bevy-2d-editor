//! Built-in extension manifests (ADR-0040 step 2).
//!
//! Each built-in ships as a `pub fn manifest() -> ExtensionManifest` constant
//! so the extension registry can pre-populate via `ExtensionRegistry::with_builtins`.

pub mod importer_aseprite;
pub mod importer_ldtk;
pub mod importer_tiled;
pub mod logic_bricks;
pub mod logic_recipes;
pub mod scene_validator;

// Re-export manifests at the builtins level for ergonomic access.
pub use importer_aseprite::importer as aseprite_importer;
pub use importer_aseprite::manifest as aseprite_manifest;
pub use importer_ldtk::importer as ldtk_importer;
pub use importer_ldtk::manifest as ldtk_manifest;
pub use importer_tiled::importer as tiled_importer;
pub use importer_tiled::manifest as tiled_manifest;
pub use logic_bricks::manifest as logic_bricks_manifest;
pub use logic_recipes::manifest as logic_recipes_manifest;
pub use scene_validator::manifest as scene_validator_manifest;
