//! Built-in Logic Bricks RustController extension.
//!
//! Declares `Capability::Commands` for controller evaluation and
//! `Permission { area: Commands, scope: Propose }`.

use editor_model::extension::{
    Capability, CapabilityDescriptor, ExtensionId, ExtensionManifest, Permission,
    PermissionArea, PermissionScope, SemVer,
};

/// Manifest for `builtin.logic-bricks.controllers`.
pub fn manifest() -> ExtensionManifest {
    ExtensionManifest::new(
        ExtensionId::new("builtin.logic-bricks.controllers"),
        SemVer::new(0, 92, 0),
        vec![CapabilityDescriptor {
            kind: Capability::Commands,
            description: Some("Built-in RustController evaluators for Logic Bricks".to_string()),
        }],
        vec![Permission::new(PermissionArea::Commands, PermissionScope::Propose)],
    )
}
