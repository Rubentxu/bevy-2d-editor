# Proposal: Scene Asset Document Types

> Change: `scene-asset-document` · Phase: propose · Status: accepted

## Status

Accepted.

## Goal

Introduce Rust types for `SceneAssetDocument`, `SceneInstance`, and `BSN IR` in `crates/editor-core/src/`. No commands, no undo, no UI, no migration. Pure type + serde layer.

## Deliverables

### Types

1. **`LocalId`** — opaque stable identity for entities inside a Scene Asset. `#[serde(transparent)]` String wrapper. Never a Bevy Entity index.
2. **`AssetReference`** — logical project path string. `#[serde(transparent)]` wrapper.
3. **`SceneAssetRole`** — soft lifecycle policy enum: `Actor`, `Fragment`, `Screen`, `Level`, `Ui`, `Effect`. `#[serde(rename_all = "snake_case")]`.
4. **`SceneAssetDocument`** — editor-owned authoring document. Fields: `asset_id`, `logical_path`, `role`, `version`, `entities`, `relationships`, `exposed_properties`, `metadata`.
5. **`SceneInstance`** — placed use of a Scene Asset: `instance_id`, `asset_ref`, `asset_version_seen`, `id_map` (`BTreeMap<LocalId, StableId>`), `overrides`, `orphaned_overrides`.
6. **`OverridePatch`** — non-destructive override: `target_local_id: LocalId`, `field_path: Vec<String>`, `value: JsonValue`, `status: OverrideStatus`.
7. **`OverrideStatus`** — closed enum: `Active`, `Orphaned`, `Stale`, `Conflict`. `#[serde(rename_all = "snake_case")]`.

### Modules

1. **`scene_asset.rs`** — `LocalId`, `AssetReference`, `SceneAssetRole`, `SceneAssetDocument`, `SceneAssetEntity`, `SceneAssetRelationship`, `RelationshipKind`, `ExposedProperty`, `SceneAssetMetadata`, `RoleWarning`, `validate_role()`.
2. **`scene_instance.rs`** — `OverrideStatus`, `OverridePatch`, `SceneInstance`, `patch_status_after_field_rename()`.
3. **`bsn_ir.rs`** — `BsnIrNode`, `BsnIrRelationship`, `BsnPatchOp`, `BsnPatch`, `BsnIr`, `bsn_ir_from_scene_asset()`.

## Module Placement

```
crates/editor-core/src/
├── scene_asset.rs     (new)
├── scene_instance.rs  (new)
├── bsn_ir.rs          (new)
└── lib.rs             (add pub mod + pub use for the three modules)
```

## Non-Goals

The following are explicitly out of scope:
- Commands, operation log, undo/redo
- UI, inspector, override editing
- Migration from EntityTemplate
- Scene Asset Catalog (project-level registry)
- Resync/rebind/cleanup workflows
- Apply/revert tools
- Physical `.bsn` file import/export
- BSN IR write-back (one-way projection only)
- Auto-resync on open
- Scene Asset variants (inheritance)
- Handle resolution (logical path → Bevy Handle)

## Tests

### Round-trip serde

- `SceneAssetDocument` serializes to JSON and deserializes back to equal value
- `SceneInstance` serializes and deserializes with `BTreeMap` id_map preserved
- `BsnIr` serializes and deserializes back to equal value

### Lossy conversion

- `bsn_ir_from_scene_asset()` drops metadata, exposed_properties, logical_path, asset_id, version

### Override behavior

- `OverridePatch` targets `LocalId`, not name (rename does not redirect patch)
- `patch_status_after_field_rename()` returns `Stale` for active patches when field is renamed
- `OverrideStatus` is a closed enum with exactly 4 variants

### Identity independence

- `local_path` + `name` are independent of `local_id` (rename of name does not change local_id or local_path)
- `LocalId` and `StableId` are distinct opaque types (type-system guarantee)

### Hierarchy

- Hierarchy lives in `SceneAssetRelationship(RelationshipKind::Child)` only
- `SceneAssetEntity` has NO `children_local_ids` field

### Role validation

- `validate_role()` returns `Vec<RoleWarning>` (NOT `Result`), never Err
- Soft warnings only; no enforcement

## Design Tensions

### Tension 1: Closed vs. Open Override Status

ADR-0005 specifies 4 override statuses. Future Bevy BSN may introduce additional states (e.g., `Inherited`, `Resolved`). A closed enum is safer for now but may need extension later.

**Resolution**: closed enum now; explicit migration when Bevy adds states.

### Tension 2: Hierarchy Location

Options:
- **(A) Hierarchy in `SceneAssetEntity.children_local_ids`**: simple parent→children map
- **(B) Hierarchy via `relationships: Vec<Relationship>`**: typed, extensible relationships

ADR-0005 chose (B) — typed relationships allow `Child`, `Custom(String)`, and future `Parent`/`Reference` variants without changing entity structure.

### Tension 3: BsnIr Completeness

`BsnIr` intentionally loses metadata, exposed properties, version. This is documented as lossy. Future write-back will need a more complete representation. The tension is between a thin projection (simple) and a full projection (useful for more scenarios).

**Resolution**: one-way thin projection for now; full representation when write-back is designed.

## Token Budget

~1500 words. Under implementation budget.
