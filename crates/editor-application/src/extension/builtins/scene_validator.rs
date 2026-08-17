//! Built-in scene-document validator extension.
//!
//! Declares `Capability::Validators` for document validation and
//! `Permission { area: Project, scope: Read }`.

use editor_model::extension::{
    Capability, CapabilityDescriptor, ExtensionId, ExtensionManifest, Permission,
    PermissionArea, PermissionScope, SemVer,
};

/// Manifest for `builtin.scene-validator`.
pub fn manifest() -> ExtensionManifest {
    ExtensionManifest::new(
        ExtensionId::new("builtin.scene-validator"),
        SemVer::new(0, 92, 0),
        vec![CapabilityDescriptor {
            kind: Capability::Validators,
            description: Some("Built-in scene document validator".to_string()),
        }],
        vec![Permission::new(PermissionArea::Project, PermissionScope::Read)],
    )
}
