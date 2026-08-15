//! Scene Instance — placed use of a Scene Asset (reference + instance components +
//! component overrides + id_map).
//!
//! PR2: SceneInstance defined locally using editor_core types (document::StableId,
//! document::ComponentInstance) to avoid type mismatches with editor_model types.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Re-export ComponentInstance from document (local) — same crate, no circular issue.
pub use crate::document::ComponentInstance;

// Import AssetReference and SceneAssetLocalId from editor_model (they're the same
// string-newtypes as the local types, just in a different crate).
pub use editor_model::ids::SceneAssetLocalId;
pub use editor_model::scene_asset::AssetReference;

use crate::document::StableId;
#[allow(deprecated)]
use crate::scene_asset::LocalId;
use crate::schema::ComponentTypeId;

/// Component override health (ADR-0005 §Overrides, §Versioning; ADR-0009).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentOverrideStatus {
    Active,
    Orphaned,
    Stale,
    Conflict,
}

/// A single non-destructive component field patch on a placed Scene Instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentOverride {
    pub target_local_id: SceneAssetLocalId,
    pub component_type_id: ComponentTypeId,
    pub field_path: Vec<String>,
    pub value: serde_json::Value,
    pub status: ComponentOverrideStatus,
}

/// A placed use of a Scene Asset: reference + instance components + component overrides,
/// NOT a deep clone (ADR-0005/ADR-0009/level-design-layers-research).
///
/// Three distinct concept groups coexist on a `SceneInstance`:
/// 1. **Asset components** live in the referenced `SceneAssetDocument` and are
///    composed at projection time.
/// 2. **Instance components** (`instance_components`) are owned by the placed
///    occurrence itself — e.g. `editor.Transform2D` placement, future
///    `editor.Name` for local labels.
/// 3. **Component Overrides** (`component_overrides` / `orphaned_component_overrides`)
///    are non-destructive patches against asset-local Entity components only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneInstance {
    pub instance_id: StableId,
    pub asset_ref: AssetReference,
    pub asset_version_seen: u32,
    pub id_map: BTreeMap<SceneAssetLocalId, StableId>,
    /// Components owned by this placed occurrence (placement-time).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instance_components: Vec<ComponentInstance>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_overrides: Vec<ComponentOverride>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub orphaned_component_overrides: Vec<ComponentOverride>,
}

/// Pure helper: returns `Stale` if any field_path segment equals renamed_field.0
/// (the old name) AND the patch status is currently `Active`; otherwise returns
/// the patch's current status unchanged.
pub fn component_override_status_after_field_rename(
    patch: &ComponentOverride,
    renamed_field: (&str, &str),
) -> ComponentOverrideStatus {
    let (old_name, _new_name) = renamed_field;
    if patch.status == ComponentOverrideStatus::Active
        && patch.field_path.iter().any(|s| s == old_name)
    {
        ComponentOverrideStatus::Stale
    } else {
        patch.status
    }
}
