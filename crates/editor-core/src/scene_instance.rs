//! Scene Instance — placed use of a Scene Asset (reference + overrides + id_map).
//! Per ADR-0005 §Overrides, §Versioning.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::document::StableId;
use crate::scene_asset::LocalId;

/// Override health (ADR-0005 §Overrides, §Versioning).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverrideStatus {
    Active,
    Orphaned,
    Stale,
    Conflict,
}

/// A single non-destructive patch on a placed Scene Instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverridePatch {
    pub target_local_id: LocalId,
    pub field_path: Vec<String>,
    pub value: serde_json::Value,
    pub status: OverrideStatus,
}

/// A placed use of a Scene Asset: reference + patches, NOT a deep clone (ADR-0005).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneInstance {
    pub instance_id: StableId,
    pub asset_ref: crate::scene_asset::AssetReference,
    pub asset_version_seen: u32,
    pub id_map: BTreeMap<LocalId, StableId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<OverridePatch>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub orphaned_overrides: Vec<OverridePatch>,
}

/// Pure helper: returns `Stale` if any field_path segment equals renamed_field.0
/// (the old name) AND the patch status is currently `Active`; otherwise returns
/// the patch's current status unchanged.
pub fn patch_status_after_field_rename(
    patch: &OverridePatch,
    renamed_field: (&str, &str),
) -> OverrideStatus {
    let (old_name, _new_name) = renamed_field;
    if patch.status == OverrideStatus::Active && patch.field_path.iter().any(|s| s == old_name) {
        OverrideStatus::Stale
    } else {
        patch.status
    }
}
