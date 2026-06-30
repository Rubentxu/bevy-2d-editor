//! Scene Instance — placed use of a Scene Asset (reference + component overrides + id_map).
//! Per ADR-0005 §Overrides, §Versioning.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::document::StableId;
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
    pub target_local_id: LocalId,
    pub component_type_id: ComponentTypeId,
    pub field_path: Vec<String>,
    pub value: serde_json::Value,
    pub status: ComponentOverrideStatus,
}

/// A placed use of a Scene Asset: reference + component overrides, NOT a deep clone (ADR-0005/ADR-0009).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneInstance {
    pub instance_id: StableId,
    pub asset_ref: crate::scene_asset::AssetReference,
    pub asset_version_seen: u32,
    pub id_map: BTreeMap<LocalId, StableId>,
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
    if patch.status == ComponentOverrideStatus::Active && patch.field_path.iter().any(|s| s == old_name) {
        ComponentOverrideStatus::Stale
    } else {
        patch.status
    }
}
