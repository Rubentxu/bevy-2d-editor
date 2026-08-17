//! Built-in extension manifests (ADR-0040 step 2).
//!
//! Each built-in ships as a `pub fn manifest() -> ExtensionManifest` constant
//! so the extension registry can pre-populate via `ExtensionRegistry::with_builtins`.

pub mod logic_bricks;
pub mod logic_recipes;
pub mod scene_validator;
