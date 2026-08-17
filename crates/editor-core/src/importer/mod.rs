//! Importer module — built-in external source importers for editor-core.
//!
//! ## Architecture
//!
//! Importers live here (in `editor-core`) so they can be used by Bevy systems
//! during preview rebuild without importing `editor-application`. The trait impl
//! objects (`Arc<dyn Importer>`) are registered in `EditorSession::importer_registry`
//! (in `editor-application`) at session creation via `ImporterRegistry::with_builtins`.
//!
//! ## Built-in importers
//!
//! | ID | Kind | Supported versions |
//! |---|---|---|
//! | `builtin.aseprite` | `ExternalSourceKind::Aseprite` | 1.0.0 – 2.0.0 |
//! | `builtin.ldtk` | `ExternalSourceKind::Ldtk` | 1.0.0 – 1.5.0 |
//! | `builtin.tiled` | `ExternalSourceKind::Tiled` | 1.0.0 – 1.10.0 |
//!
//! ## Note on base64 encoding
//!
//! The `base64_engine` module is referenced but not directly imported in this module.
//! A minimal inline base64 implementation is used to avoid an extra dependency.

pub mod aseprite;
pub mod ldtk;
pub mod tiled;

// Re-export the public importer types for convenience.
pub use aseprite::AsepriteImporter;
pub use ldtk::LdtkImporter;
pub use tiled::TiledImporter;
