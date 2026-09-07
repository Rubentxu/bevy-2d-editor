//! Built-in Tiled importer registration.
//!
//! Registers `builtin.tiled` with `Capability::Importers` in the importer registry.
//! The concrete `TiledImporter` implementation lives in `editor_bevy::importer::tiled`
//! and is composed into the registry by `editor_wasm` (the WASM composition root).

use editor_model::extension::{Capability, CapabilityDescriptor, ExtensionId, ExtensionManifest, Permission, PermissionArea, PermissionScope, SemVer};

/// Manifest for `builtin.tiled`.
///
/// Declares `Capability::Importers` so the importer registry routes
/// Tiled import requests to the `TiledImporter` implementation.
pub fn manifest() -> ExtensionManifest {
    ExtensionManifest::new(
        ExtensionId::new("builtin.tiled"),
        SemVer::new(0, 93, 0),
        vec![CapabilityDescriptor {
            kind: Capability::Importers,
            description: Some("Tiled JSON map importer for 2D level design".to_string()),
        }],
        vec![Permission::new(PermissionArea::Importers, PermissionScope::Read)],
    )
}
