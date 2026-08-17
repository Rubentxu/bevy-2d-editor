//! Built-in Logic Bricks recipe extension.
//!
//! Declares `Capability::Recipes` for the three built-in `LogicGraphAsset` recipes
//! and `Permission { area: Recipes, scope: Write }`.

use editor_model::extension::{
    Capability, CapabilityDescriptor, ExtensionId, ExtensionManifest, Permission,
    PermissionArea, PermissionScope, SemVer,
};

/// Manifest for `builtin.logic-recipes`.
pub fn manifest() -> ExtensionManifest {
    ExtensionManifest::new(
        ExtensionId::new("builtin.logic-recipes"),
        SemVer::new(0, 92, 0),
        vec![CapabilityDescriptor {
            kind: Capability::Recipes,
            description: Some("Built-in LogicGraphAsset recipes: platformer_jump, health_damage, proximity_trigger".to_string()),
        }],
        vec![Permission::new(PermissionArea::Recipes, PermissionScope::Write)],
    )
}
