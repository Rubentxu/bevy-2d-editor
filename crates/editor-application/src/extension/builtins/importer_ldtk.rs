//! Built-in LDtk importer registration.
//!
//! Registers `builtin.ldtk` with `Capability::Importers` in the importer registry.
//! The concrete `LdtkImporter` implementation lives in `editor_core::importer::ldtk`.

use editor_core::importer::LdtkImporter;
use editor_model::extension::{Capability, CapabilityDescriptor, ExtensionId, ExtensionManifest, Permission, PermissionArea, PermissionScope, SemVer};
use std::sync::Arc;

/// Manifest for `builtin.ldtk`.
///
/// Declares `Capability::Importers` so the importer registry routes
/// LDtk import requests to the `LdtkImporter` implementation.
pub fn manifest() -> ExtensionManifest {
    ExtensionManifest::new(
        ExtensionId::new("builtin.ldtk"),
        SemVer::new(0, 93, 0),
        vec![CapabilityDescriptor {
            kind: Capability::Importers,
            description: Some("LDtk JSON project importer for 2D level design".to_string()),
        }],
        vec![Permission::new(PermissionArea::Importers, PermissionScope::Read)],
    )
}

/// Construct the `LdtkImporter` concrete implementation for registration.
pub fn importer() -> Arc<dyn editor_model::importer::Importer> {
    Arc::new(LdtkImporter::new())
}
