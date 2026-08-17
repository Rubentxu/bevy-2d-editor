//! Built-in Aseprite importer registration.
//!
//! Registers `builtin.aseprite` with `Capability::Importers` in the importer registry.
//! The concrete `AsepriteImporter` implementation lives in `editor_core::importer::aseprite`.

use editor_core::importer::AsepriteImporter;
use editor_model::extension::{Capability, CapabilityDescriptor, ExtensionId, ExtensionManifest, Permission, PermissionArea, PermissionScope, SemVer};
use std::sync::Arc;

/// Manifest for `builtin.aseprite`.
///
/// Declares `Capability::Importers` so the importer registry routes
/// Aseprite import requests to the `AsepriteImporter` implementation.
pub fn manifest() -> ExtensionManifest {
    ExtensionManifest::new(
        ExtensionId::new("builtin.aseprite"),
        SemVer::new(0, 93, 0),
        vec![CapabilityDescriptor {
            kind: Capability::Importers,
            description: Some("Aseprite JSON + PNG importer for sprite animation sequences".to_string()),
        }],
        vec![Permission::new(PermissionArea::Importers, PermissionScope::Read)],
    )
}

/// Construct the `AsepriteImporter` concrete implementation for registration.
pub fn importer() -> Arc<dyn editor_model::importer::Importer> {
    Arc::new(AsepriteImporter::new())
}
