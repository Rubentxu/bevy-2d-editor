# Spec: SceneAssetDocument + SceneInstance + BSN IR

> Change: `scene-asset-document` · Phase: sddk-spec · Path: A-lite

## §1. Spec Metadata

- **Change:** `scene-asset-document`
- **Phase:** spec
- **Source proposal:** [`docs/sddk/scene-asset-document/proposal.md`](../sddk/scene-asset-document/proposal.md)
- **Source explore:** [`docs/sddk/scene-asset-document/explore-report.md`](../sddk/scene-asset-document/explore-report.md)
- **Authoritative references:**
  - [ADR-0005 — Scene Asset as the BSN-Aligned Reusable Scene Model](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md)
  - Bevy 0.19 release notes — Next Generation Scenes / BSN
  - Bevy PR #23413 — core scene system, `bsn!`, templates

---

## §2. Capability: `scene-asset-document`

### Requirement: scene-asset-document-model

`SceneAssetDocument`, `SceneInstance`, `BSN IR` types round-trip through serde faithfully.

#### Scenario: S1 — SceneAssetDocument serde round-trip

**Given** a `SceneAssetDocument` with `asset_id: "asset-001"`, `logical_path: "assets/player.bsn"`, `role: SceneAssetRole::Actor`, `version: 3`, 2 entities, 1 `RelationshipKind::Child` relationship, 1 `ExposedProperty`, and a populated `SceneAssetMetadata`

**When** the document is serialized to JSON via `serde_json::to_string` and deserialized back via `serde_json::from_str`

**Then** the round-tripped document equals the original in all fields
- AND `asset_id` equals `"asset-001"`
- AND `logical_path` equals `"assets/player.bsn"`
- AND `role` equals `SceneAssetRole::Actor`
- AND `version` equals `3`
- AND `entities.len()` equals `2`
- AND `relationships.len()` equals `1`
- AND `exposed_properties.len()` equals `1`
- AND the JSON does NOT contain the key `"children_local_ids"`

#### Scenario: S2 — SceneInstance serde round-trip

**Given** a `SceneInstance` with `instance_id: StableId("inst-001")`, `asset_ref: AssetReference("assets/player.bsn")`, `asset_version_seen: 7`, an `id_map` containing 2 entries, and 1 `OverridePatch` with `status: OverrideStatus::Active`

**When** the instance is serialized to JSON and deserialized back

**Then** the round-tripped instance equals the original
- AND `asset_ref.as_str()` equals `"assets/player.bsn"`
- AND `asset_version_seen` equals `7`
- AND `id_map.len()` equals `2`
- AND `overrides.len()` equals `1`
- AND `overrides[0].status` equals `OverrideStatus::Active`

### Requirement: override-patch-targeting

Override patches target `LocalId`, not display name. Rename of a human name does not redirect a patch.

#### Scenario: S3 — OverridePatch targets LocalId, not name

**Given** an `OverridePatch` with `target_local_id: LocalId("weapon")`, `field_path: ["Sprite2D", "asset"]`, `value: "cannon.png"`, `status: OverrideStatus::Active`

**When** the entity's `name` field is changed from `"Weapon"` to `"Cannon"` and the patch is re-serialized

**Then** `patch.target_local_id.as_str()` still equals `"weapon"`
- AND the patch's identity is unchanged (targets by `LocalId`, not by `name`)

#### Scenario: S4 — Rename of a component field marks patch Stale

**Given** an `OverridePatch` with `target_local_id: LocalId("weapon")`, `field_path: ["Sprite2D", "asset"]`, `value: "cannon.png"`, `status: OverrideStatus::Active`

**When** `patch_status_after_field_rename(&patch, ("Sprite2D", "Sprite"))` is called

**Then** the returned `OverrideStatus` equals `OverrideStatus::Stale`
- AND if `patch.status` is `OverrideStatus::Orphaned` before the rename call, it remains `OverrideStatus::Orphaned` (non-Active patches are not marked Stale)

### Requirement: override-status-closed-enum

`OverrideStatus` is a closed enum with exactly 4 variants.

#### Scenario: S5 — OverrideStatus is a closed enum

**Given** an `OverridePatch` with `status: OverrideStatus::Active`

**When** serialized to JSON and deserialized back

**Then** the status equals `OverrideStatus::Active` (serde uses `#[serde(rename_all = "snake_case")]`, producing lowercase `"active"`)
- AND the enum has exactly 4 variants: `Active`, `Orphaned`, `Stale`, `Conflict`
- AND a `match` expression on `OverrideStatus` is exhaustive with only these 4 arms

### Requirement: bsn-ir-lossy

`BsnIr` is a derived, lossy representation. It does not round-trip faithfully to `SceneAssetDocument`.

#### Scenario: S6 — BsnIr is a derived lossy representation

**Given** a `SceneAssetDocument` with `asset_id`, `logical_path`, `role`, `version`, `metadata`, `exposed_properties`, `entities`, and `relationships`

**When** `bsn_ir_from_scene_asset(&doc)` is called to produce a `BsnIr`

**Then** the `BsnIr` contains the scene graph structure from `entities` and `relationships`
- AND the `BsnIr` does NOT contain `metadata` fields (`tags`, `created_at`, `updated_at`, `notes`)
- AND the `BsnIr` does NOT contain `exposed_properties`
- AND the `BsnIr` does NOT preserve `logical_path` as a structured field
- AND the `BsnIr` does NOT contain `asset_id` or `version`

### Requirement: role-validation-soft-warnings

Role validation emits soft warnings, never errors. The `validate_role()` function returns `Vec<RoleWarning>`, not `Result`.

#### Scenario: S7 — Fragment role emits standalone warning when no Child relationships

**Given** a `SceneAssetDocument` with `role: SceneAssetRole::Fragment` and `relationships: []` (no Child relationships pointing away from it)

**When** `validate_role(SceneAssetRole::Fragment, &doc)` is called

**Then** the returned `Vec<RoleWarning>` is non-empty
- AND contains a warning with `code: "fragment_standalone"`
- AND calling `validate_role` does NOT return an `Err`

### Requirement: identity-independence

`local_path` and `name` are independent of `local_id`. Renaming the display name does not change the `local_id` or `local_path`.

#### Scenario: S8 — local_path and name are independent of local_id

**Given** a `SceneAssetEntity` with `local_id: LocalId("weapon")`, `local_path: "root/weapon"`, `name: "Weapon"`, and empty `components`

**When** the entity is serialized to JSON, then the deserialized copy's `name` field is changed from `"Weapon"` to `"Cannon"`

**Then** the `local_id` is unchanged (`"weapon"`)
- AND the `local_path` is unchanged (`"root/weapon"`)
- AND only `name` differs between the original and the mutated copy

### Requirement: hierarchy-via-relationships-only

Hierarchy lives ONLY in `SceneAssetRelationship(RelationshipKind::Child)`. `SceneAssetEntity` does NOT have a `children_local_ids` field.

#### Scenario: S9 — Hierarchy via relationships only

**Given** a `SceneAssetDocument` with 2 entities and a `RelationshipKind::Child` relationship from the first entity to the second

**When** the document is serialized to JSON

**Then** the JSON contains `"relationships"` with `"kind":"child"`
- AND the JSON does NOT contain the key `"children_local_ids"`
- AND deserializing JSON that contains `"children_local_ids"` returns a `Result::Err`

### Requirement: local-id-stable-id-distinction

`LocalId` and `StableId` are distinct opaque types with a type-system guarantee they cannot be confused.

#### Scenario: S10 — LocalId and StableId are distinct types

**Given** a `LocalId("root")` and a `StableId("ent_a")`

**When** `std::any::TypeId::of::<LocalId>()` and `std::any::TypeId::of::<StableId>()` are compared

**Then** the two `TypeId` values are NOT equal
- AND a function `fn accepts_local_id(_: LocalId)` cannot be called with a `StableId` argument (compile-time type safety)
- AND a function `fn accepts_stable_id(_: StableId)` cannot be called with a `LocalId` argument

---

## §3. Out-of-Scope Behaviors

The following are NOT part of this change:

- Commands, operation log, undo/redo
- UI, inspector, override editing panel
- Migration from EntityTemplate
- Scene Asset Catalog (project-level registry)
- Resync/rebind/cleanup workflows for orphaned/stale overrides
- Apply/revert tools
- Physical `.bsn` file import/export
- BSN IR write-back (projection is one-way only)
- Auto-resync when `asset_version_seen < asset.version`
- Scene Asset variants (inheritance from another Scene Asset)
- Handle resolution (logical `AssetReference` → Bevy `Handle<Image>`)

---

## §4. Acceptance Criteria

1. `SceneAssetDocument`, `SceneInstance`, and `BsnIr` types exist in `crates/editor-core/src/`
2. `SceneAssetDocument` round-trips through `serde_json` with all fields preserved
3. `SceneInstance` round-trips with `BTreeMap<LocalId, StableId>` preserved
4. `OverridePatch.target_local_id` is `LocalId`, not `String` or `name`
5. `OverrideStatus` is a closed enum with exactly 4 variants
6. `BsnIr` is produced by `bsn_ir_from_scene_asset()` and loses metadata fields
7. `validate_role()` returns `Vec<RoleWarning>`, never `Err`
8. `local_path` and `name` are independent of `local_id`
9. `SceneAssetEntity` has NO `children_local_ids` field
10. `LocalId` and `StableId` are distinct types at compile time
11. All 10 scenarios have passing tests for `wasm32-unknown-unknown` target
12. WASM builds pass: `cargo check`, `cargo test --no-run`, `cargo fmt --check`

---

## §5. Open Questions

1. **Handle resolution**: How does `AssetReference` (logical path string) map to Bevy's `Handle<Image>` at spawn time? Tracked separately.
2. **SceneInstance persistence location**: Should `SceneInstance` live inside the `SceneDocument` JSON, in a separate `instances.json`, or in the `SceneAsset` catalog? Deferred to OPFS persistence change.
3. **Bevy 0.20 inheritance**: If Bevy introduces formal scene inheritance, should `SceneAsset` support variants? Deferred to future design.
