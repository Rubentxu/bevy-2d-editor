//! Scene Instance — placed use of a Scene Asset (reference + instance components +
//! component overrides + id_map).
//! Per ADR-0005 §Overrides, §Versioning; ADR-0009; level-design-layers-research design.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::component::ComponentInstance;
use crate::ids::{SceneAssetLocalId, StableId};
use crate::scene_asset::AssetReference;
use crate::schema::ComponentTypeId;

/// Component override health (ADR-0005 §Overrides, §Versioning; ADR-0009).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentOverrideStatus {
    /// Override is active and applies normally.
    Active,
    /// Override target (field or component) no longer exists in the asset.
    Orphaned,
    /// Override is active but the asset has been edited since the override was created.
    Stale,
    /// Override conflicts with a concurrent asset edit.
    Conflict,
}

/// A single non-destructive component field patch on a placed Scene Instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentOverride {
    /// Local ID of the asset entity targeted by this override.
    pub target_local_id: SceneAssetLocalId,
    /// Component type being overridden.
    pub component_type_id: ComponentTypeId,
    /// Dot-separated path to the specific field within the component.
    pub field_path: Vec<String>,
    /// Overridden value as JSON.
    pub value: serde_json::Value,
    /// Current health status of this override.
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
    /// Stable identifier for this placed instance.
    pub instance_id: StableId,
    /// Logical path of the source Scene Asset.
    pub asset_ref: AssetReference,
    /// Version of the source Scene Asset when this instance was placed.
    pub asset_version_seen: u32,
    /// Maps asset-local IDs to scene-level stable IDs.
    pub id_map: BTreeMap<SceneAssetLocalId, StableId>,
    /// Components owned by this placed occurrence (placement-time).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instance_components: Vec<ComponentInstance>,
    /// Non-stale, non-conflicting overrides targeting live asset entities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_overrides: Vec<ComponentOverride>,
    /// Overrides whose target no longer exists in the source asset.
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
