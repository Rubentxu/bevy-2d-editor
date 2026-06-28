# Design: scene-asset-document

> Change: `scene-asset-document` · Phase: design · Path: A-lite

## Module Layout

Three new files under `crates/editor-core/src/`, wired into `lib.rs`:

```
crates/editor-core/src/
├── scene_asset.rs     (new — LocalId, AssetReference, SceneAssetRole, SceneAssetDocument, ...)
├── scene_instance.rs  (new — OverrideStatus, OverridePatch, SceneInstance, ...)
├── bsn_ir.rs          (new — BsnIr, BsnIrNode, BsnIrRelationship, BsnPatch, ...)
└── lib.rs             (add pub mod + pub use)
```

### lib.rs wiring

```rust
pub mod bsn_ir;
pub mod scene_asset;
pub mod scene_instance;

pub use bsn_ir::{
    BsnIr, BsnIrNode, BsnIrRelationship, BsnPatch, BsnPatchOp, bsn_ir_from_scene_asset,
};
pub use scene_asset::{
    AssetReference, ExposedProperty, LocalId, RelationshipKind, RoleWarning,
    SceneAssetDocument, SceneAssetEntity, SceneAssetMetadata, SceneAssetRelationship,
    SceneAssetRole, SceneAssetRole::*, validate_role,
};
pub use scene_instance::{
    patch_status_after_field_rename, OverridePatch, OverrideStatus, OverrideStatus::*,
    SceneInstance,
};
```

---

## Type Signatures

### scene_asset.rs

```rust
/// Opaque stable identity of an entity *inside* a Scene Asset.
/// Never appears as a SceneDocument StableId. Overrides target this.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LocalId(pub String);

impl LocalId {
    pub fn new(id: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

/// Logical Project path (human-readable), e.g. "assets/characters/player".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssetReference(pub String);

impl AssetReference {
    pub fn new(path: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

/// Soft validation policy, not a separate asset type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneAssetRole {
    Actor,
    Fragment,
    Screen,
    Level,
    Ui,
    Effect,
}

/// Editor-owned durable authoring document for a Scene Asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetDocument {
    pub asset_id: String,
    pub logical_path: String,
    pub role: SceneAssetRole,
    pub version: u32,
    pub entities: Vec<SceneAssetEntity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<SceneAssetRelationship>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exposed_properties: Vec<ExposedProperty>,
    #[serde(default)]
    pub metadata: SceneAssetMetadata,
}

/// One entity inside a Scene Asset.
/// NOTE: NO children_local_ids — hierarchy lives only in relationships (spec S9).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetEntity {
    pub local_id: LocalId,
    pub local_path: String,
    pub name: String,
    pub components: Vec<ComponentInstance>,
}

/// Typed relationship between entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationshipKind {
    Child,
    #[serde(rename = "custom")]
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetRelationship {
    pub from_local_id: LocalId,
    pub to_local_id: LocalId,
    pub kind: RelationshipKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_path: Option<Vec<String>>,
}

/// A property the asset exposes for instance overriding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExposedProperty {
    pub name: String,
    pub target_local_id: LocalId,
    pub field_path: Vec<String>,
    pub default_value: serde_json::Value,
}

/// Spike-simple metadata: all Option<String>.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SceneAssetMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleWarning {
    pub code: String,
    pub message: String,
}

/// Soft role-validation warnings (NOT errors).
/// Returns Vec<RoleWarning>, not Result.
pub fn validate_role(role: SceneAssetRole, doc: &SceneAssetDocument) -> Vec<RoleWarning>;
```

### scene_instance.rs

```rust
/// Override health.
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

/// A placed use of a Scene Asset: reference + patches, NOT a deep clone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneInstance {
    pub instance_id: StableId,
    pub asset_ref: AssetReference,
    pub asset_version_seen: u32,
    pub id_map: BTreeMap<LocalId, StableId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<OverridePatch>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub orphaned_overrides: Vec<OverridePatch>,
}

/// Returns `Stale` if any field_path segment equals renamed_field.0 AND
/// the patch status is currently `Active`; otherwise returns patch.status unchanged.
pub fn patch_status_after_field_rename(
    patch: &OverridePatch,
    renamed_field: (&str, &str),
) -> OverrideStatus;
```

### bsn_ir.rs

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BsnIrRelationship {
    pub kind: String,
    pub target_identifier: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BsnIrNode {
    pub identifier: String,
    pub components: BTreeMap<String, serde_json::Value>,
    pub children: Vec<BsnIrNode>,
    pub relationships: Vec<BsnIrRelationship>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BsnPatchOp {
    Replace,
    AddChild,
    RemoveChild,
    #[serde(rename = "custom")]
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BsnPatch {
    pub target_identifier: String,
    pub op: BsnPatchOp,
    pub value: serde_json::Value,
}

/// Lossy semantic projection of a SceneAssetDocument.
/// Drops: metadata, exposed_properties, logical_path, asset_id, version.
/// Root = first entity; children built from RelationshipKind::Child.
/// asset_refs/patches are empty for a pure document (they come from instances).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BsnIr {
    pub scene_root: BsnIrNode,
    pub asset_refs: Vec<String>,
    pub patches: Vec<BsnPatch>,
}

/// One-way projection. Does NOT round-trip faithfully.
pub fn bsn_ir_from_scene_asset(doc: &SceneAssetDocument) -> BsnIr;
```

---

## Derive Decisions

| Type | Debug | Clone | PartialEq | Eq | Hash | Serialize | Deserialize | Notes |
|------|-------|-------|-----------|----|------|-----------|--------------|-------|
| `LocalId` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (transparent) | `Ord` for BTreeMap key |
| `AssetReference` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (transparent) | |
| `SceneAssetRole` | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | `Copy`; no `Hash` (enum with data) |
| `SceneAssetDocument` | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | no `Eq` (contains `Vec`) |
| `SceneAssetEntity` | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | no `Eq` (contains `Vec`) |
| `RelationshipKind` | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | |
| `SceneAssetRelationship` | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | no `Eq` (contains `Vec`) |
| `ExposedProperty` | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | |
| `SceneAssetMetadata` | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | `Default` (serde requirement) |
| `RoleWarning` | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | |
| `OverrideStatus` | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | `Copy` |
| `OverridePatch` | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | |
| `SceneInstance` | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | |
| `BsnIr` | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | |
| `BsnIrNode` | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | |
| `BsnIrRelationship` | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | |
| `BsnPatchOp` | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | |
| `BsnPatch` | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | |

---

## Round-Trip Loss Matrix

| Field | SceneAssetDocument | SceneInstance | BsnIr |
|-------|------------------|--------------|-------|
| `asset_id` | ✅ Preserved | N/A | ❌ Dropped |
| `logical_path` | ✅ Preserved | N/A | ⚠️ Flattened to `asset_refs[0]` if non-empty |
| `role` | ✅ Preserved | N/A | ❌ Dropped |
| `version` | ✅ Preserved | N/A | ❌ Dropped |
| `entities` | ✅ Preserved | N/A | ✅ Projected (as `BsnIrNode` tree) |
| `relationships` | ✅ Preserved | N/A | ✅ Projected (as `BsnIrRelationship`) |
| `exposed_properties` | ✅ Preserved | N/A | ❌ Dropped |
| `metadata` | ✅ Preserved | N/A | ❌ Dropped |
| `instance_id` | N/A | ✅ Preserved | N/A |
| `asset_ref` | N/A | ✅ Preserved | N/A |
| `asset_version_seen` | N/A | ✅ Preserved | N/A |
| `id_map` | N/A | ✅ Preserved (BTreeMap round-trips) | N/A |
| `overrides` | N/A | ✅ Preserved | N/A |
| `orphaned_overrides` | N/A | ✅ Preserved | N/A |

---

## Test Strategy

| Scenario | Test File | Test Name |
|----------|-----------|-----------|
| S1 — SceneAssetDocument round-trip | `scene_asset_roundtrip.rs` | `s1_scene_asset_document_roundtrip` |
| S2 — SceneInstance round-trip | `scene_asset_roundtrip.rs` | `s2_scene_instance_roundtrip` |
| S3 — Override targets LocalId | `override_targets.rs` | `s3_override_targets_local_id` |
| S4 — Rename marks Stale | `override_targets.rs` | `s4_rename_marks_stale` |
| S5 — OverrideStatus closed enum | `override_status_and_identity.rs` | `s5_override_status_is_closed_enum` |
| S6 — BsnIr round-trip | `scene_asset_roundtrip.rs` | `s6_bsn_ir_roundtrip` |
| S7 — Role validation warnings | `role_validation.rs` | `s7_fragment_standalone_warning` |
| S8 — local_path/name independent of local_id | `override_status_and_identity.rs` | `s8_local_path_and_name_independent_of_local_id` |
| S9 — Hierarchy via relationships only | `role_validation.rs` | `s9_hierarchy_via_relationships_only` |
| S10 — LocalId/StableId distinct types | `override_status_and_identity.rs` | `s10_local_id_and_stable_id_are_distinct_types` |

---

## Open Design Risks

### Risk 1: Handle Resolution Gap

`AssetReference` (logical path string) has no defined mapping to Bevy's `Handle<Image>`. This gap exists between `SceneInstance` placement and Bevy's scene spawner. The `id_map` resolves `LocalId → StableId`, but there is no `AssetReference → Handle<Image>` resolution step designed yet.

**Mitigation**: defer handle resolution to a separate future change. `AssetReference` remains a string.

### Risk 2: OverrideStatus Extensibility

`OverrideStatus` is a closed enum. If Bevy BSN introduces new states (e.g., `Inherited`), the enum must be extended. This requires a new Rust release and migration.

**Mitigation**: document the closed-enum constraint. When Bevy adds states, add them with a migration path.

### Risk 3: BsnIr Write-back Unknowns

The one-way projection is stable, but write-back (BSN IR → SceneAssetDocument) is blocked on Bevy's write-back APIs. When those stabilize, the reverse mapping may reveal gaps in the current `BsnIr` design (e.g., metadata fields that must be preserved but were dropped).

**Mitigation**: document the lossy nature explicitly. Write-back is a separate future change with its own design.
