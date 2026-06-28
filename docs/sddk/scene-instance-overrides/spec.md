# Spec: Scene Instance Override Resolution (Fase 3)

> Change: `scene-instance-overrides` · Phase: sddk-spec · Path: A-lite
> Companion to [`proposal.md`](./proposal.md); builds on [`explore-report.md`](./explore-report.md)
> and Fase 0/1/2 archive-reports.

## §1. Spec Metadata

- **Change:** `scene-instance-overrides`
- **Phase:** spec
- **Source proposal:** [`docs/sddk/scene-instance-overrides/proposal.md`](../sddk/scene-instance-overrides/proposal.md)
- **Source explore:** [`docs/sddk/scene-instance-overrides/explore-report.md`](../sddk/scene-instance-overrides/explore-report.md)
- **Authoritative references:**
  - [ADR-0005 — Scene Asset as the BSN-Aligned Reusable Scene Model](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md) §Overrides, §Versioning and Resync
  - Fase 0 spec: `docs/sddk/scene-asset-document/spec.md` (S3/S4 fixture locked here)

---

## §2. Capability: `scene-instance-overrides`

A new pure-functions module `crates/editor-core/src/scene_instance_overrides.rs`
delivers the override lifecycle algorithms contracted by ADR-0005 §Overrides
and §Versioning: merge, classify, resync, rebind, mint, reconcile.

### Requirement: override-patch-field-path-convention

**Field-path segment-0 is the FULL namespaced `type_id`** (e.g. `"editor.Sprite2D"`)
matching `ComponentInstance.type_id` exactly. The short form (e.g. `"Sprite2D"`)
MUST NOT match. Fase 0 S3/S4 fixtures MUST be updated to the namespaced form.

#### Scenario: S1 — Segment-0 full `type_id` classifies Active

- **Given** an asset with one entity whose components include `ComponentInstance { type_id: "editor.Sprite2D", values: { "asset": "player.png", ... } }`, and a patch `OverridePatch { target_local_id: LocalId("root"), field_path: ["editor.Sprite2D", "asset"], value: "cannon.png", status: Active }`
- **When** `classify_overrides(&asset, std::slice::from_ref(&patch))` runs
- **Then** the returned vector has length 1
- **AND** the returned patch's `status` equals `OverrideStatus::Active`

#### Scenario: S2 — Segment-0 short form does NOT match

- **Given** the same asset as S1, and a patch `OverridePatch { target_local_id: LocalId("root"), field_path: ["Sprite2D", "asset"], value: "cannon.png", status: Active }`
- **When** `classify_overrides(&asset, std::slice::from_ref(&patch))` runs
- **Then** the returned vector has length 1
- **AND** the returned patch's `status` equals `OverrideStatus::Orphaned` (no entity's component matches the short form `Sprite2D`; `classify_overrides` is conservative and surfaces the orphan even when the underlying entity exists)

### Requirement: rename-preserves-override-by-local-id

A human-facing rename of the asset entity does NOT redirect overrides.
Patches bind to `local_id`, which MUST remain stable across renames.

#### Scenario: S3 — Entity rename preserves patch via `local_id`

- **Given** an instance with `id_map: { "abc" -> StableId("ent_a") }` and `OverridePatch { target_local_id: LocalId("abc"), field_path: ["editor.Sprite2D", "asset"], value: "cannon.png", status: Active }`
- **When** the asset's entity with `local_id: "abc"` is renamed (`name: "Weapon"` → `name: "Cannon"`, `local_id` and `local_path` unchanged), `asset.version` increments to 2, and `resync(&asset, &mut instance, 2)` runs
- **Then** the patch remains in `instance.overrides`
- **AND** the patch's `status` equals `OverrideStatus::Active`
- **AND** `instance.id_map["abc"]` still equals `StableId("ent_a")`

#### Scenario: S4 — Resync advances `asset_version_seen` while patch stays Active

- **Given** the same setup as S3 with `instance.asset_version_seen: 1` and `asset.version: 2`
- **When** `resync(&asset, &mut instance, 2)` runs
- **Then** `instance.asset_version_seen` equals `2`
- **AND** `report.active` equals `1`
- **AND** `report.orphaned` equals `0`, `report.stale` equals `0`, `report.conflict` equals `0`, `report.rebound` equals `0`

### Requirement: resync-non-destructive-orphaning

`resync` MUST move patches to `orphaned_overrides` when the target entity is
removed from the asset. Patches MUST NEVER be silently deleted (ADR-0005
§Overrides, non-destructive invariant).

#### Scenario: S5 — Removing asset entity routes patch to `orphaned_overrides`

- **Given** an asset (v=1) with entity `local_id: "abc"`; an instance with `OverridePatch { target_local_id: LocalId("abc"), field_path: ["editor.Sprite2D", "asset"], value: "cannon.png", status: Active }` and empty `orphaned_overrides`
- **When** the asset is updated: entity `"abc"` removed, `version` increments to 2, then `resync(&asset_v2, &mut instance, 2)` runs
- **Then** `instance.overrides` is empty
- **AND** `instance.orphaned_overrides` has length 1
- **AND** the moved patch's `status` equals `OverrideStatus::Orphaned`
- **AND** the returned `ResyncReport` has `orphaned: 1`, `active: 0`, `stale: 0`, `conflict: 0`, `rebound: 0`

### Requirement: resync-stale-on-field-rename

When the asset renames a field that a patch targets, the patch MUST become
`Stale` (ADR-0005 §Overrides: `Active → Stale`).

#### Scenario: S6 — Asset field rename marks patch Stale

- **Given** an asset (v=1) whose component `editor.Sprite2D.values` contains key `"asset"`, and a patch `OverridePatch { target_local_id: LocalId("root"), field_path: ["editor.Sprite2D", "asset"], value: "cannon.png", status: Active }`
- **When** the asset is updated: `editor.Sprite2D.values.asset` is renamed to `editor.Sprite2D.values.image` (component unchanged, field key replaced), `version` increments to 2, then `resync(&asset_v2, &mut instance, 2)` runs
- **Then** the patch remains in `instance.overrides`
- **AND** the patch's `status` equals `OverrideStatus::Stale`
- **AND** `report.stale` equals `1`, `report.active` equals `0`

### Requirement: resync-conflict-on-type-mismatch

When the resolved terminal value's `serde_json::Value` kind differs from the
override's `value` kind, the patch MUST become `Conflict`.

#### Scenario: S7 — Asset field type change marks patch Conflict

- **Given** an asset (v=1) whose component `editor.Health.values.current` is `serde_json::Value::Number(100.0)`, and a patch `OverridePatch { target_local_id: LocalId("player"), field_path: ["editor.Health", "current"], value: serde_json::Value::Number(42.0), status: Active }`
- **When** the asset is updated: `editor.Health.values.current` changes from `Number` to `String("full")` (kind change), `version` increments to 2, then `resync(&asset_v2, &mut instance, 2)` runs
- **Then** the patch's `status` equals `OverrideStatus::Conflict`
- **AND** `report.conflict` equals `1`, `report.active` equals `0`
- **AND** the patch remains in `instance.overrides` (NOT auto-resolved; user action required, out of scope for Fase 3)

### Requirement: resync-rebind-orphaned-by-local-path-suffix

`resync` MUST attempt to rebind orphaned patches whose target entity has
reappeared in the asset, matching by `local_path` suffix against the
previously-recorded path of the orphaned entity.

#### Scenario: S8 — Rebind restores Orphaned patch via `local_path` suffix

- **Given** `instance.orphaned_overrides` contains a patch with `target_local_id: LocalId("old_abc")`, `field_path: ["editor.Sprite2D", "asset"]`, `status: Orphaned`, and the previous entity's `local_path` ended with the suffix `"weapons/cannon"`
- **When** the asset now contains a NEW entity `local_id: "new_abc"` whose `local_path` is `"root/player/weapons/cannon"`, `version` increments, and `resync(&asset, &mut instance, new_version)` runs
- **Then** `instance.orphaned_overrides` does NOT contain that patch
- **AND** `instance.overrides` contains the patch with `target_local_id: LocalId("new_abc")`, `status: Active`
- **AND** `report.rebound` equals `1`, `report.orphaned` equals `0`

### Requirement: effective-values-no-op-when-empty

`effective_values` is a pure read-only merge: with no overrides, the resolved
scene mirrors the asset and mints fresh `StableId`s.

#### Scenario: S9 — `effective_values` mirrors asset when no overrides apply

- **Given** an asset with 2 entities (each with one component); an instance with empty `overrides`, empty `orphaned_overrides`, empty `id_map`; and a counter `mint_counter: usize = 0` that returns `StableId(format!("sid_{}", mint_counter))` then increments
- **When** `effective_values(&asset, &instance, &mut mint_counter)` runs
- **Then** the returned `Result::Ok(ResolvedScene)` is `Ok`
- **AND** `resolved.entities.len()` equals `2` and matches `asset.entities` field-for-field
- **AND** `resolved.unresolved_patches` is empty
- **AND** `resolved.id_map` has 2 entries (`sid_0`, `sid_1`)
- **AND** `mint_counter` equals `2` after the call (caller persists `id_map`; module does NOT mutate `instance`)

### Requirement: id-map-reconciliation-on-asset-growth

`reconcile_id_map` MUST preserve existing `id_map` entries and extend it for
newly added asset entities (non-destructive).

#### Scenario: S10 — `id_map` extends when asset gains a new entity

- **Given** asset (v=1) with 2 entities, instance with `id_map: { "a" -> StableId("sid_0"), "b" -> StableId("sid_1") }`
- **When** asset gains a 3rd entity `local_id: "c"`, `version` increments to 2, `resync(&asset_v2, &mut instance, 2)` runs, and inside `resync` the module calls `reconcile_id_map(&asset_v2, &instance.id_map, &mut mint_counter)` then assigns the returned map back to `instance.id_map`
- **Then** `instance.id_map.len()` equals `3`
- **AND** `instance.id_map["a"]` equals `StableId("sid_0")` (preserved)
- **AND** `instance.id_map["b"]` equals `StableId("sid_1")` (preserved)
- **AND** `instance.id_map["c"]` equals a fresh, distinct `StableId` (newly minted)

---

## §3. Out-of-Scope Behaviors

The following are NOT part of this change:

1. Commands, operation log, undo/redo integration.
2. Frontend / inspector / UI surfacing of orphaned or stale overrides.
3. `SceneAssetDocument` body I/O (OPFS load/save).
4. `bsn!` codegen from a `SceneInstance` (Fase 4+).
5. Scene Asset variants / inheritance.
6. Async / parallel resync (single-threaded WASM target).
7. Auto-resolution of `Stale` / `Conflict` (detect + surface only; user resolves).
8. `AssetReference → Handle<Image>` resolution.
9. `RenamedField` event stream consumption (Fase 3 scans per-resync).
10. Type-aware conflict detection via `ComponentSchemaRegistry` (Fase 3 uses coarse `serde_json::Value` kind compare).

---

## §4. Acceptance Criteria

1. New module `crates/editor-core/src/scene_instance_overrides.rs` exists with
   the 7 functions (`effective_values`, `resync`, `mint_id_map`, `reconcile_id_map`,
   `validate_overrides`, `classify_overrides`, `try_rebind`) and the 5 types
   (`ResolvedScene`, `ResyncReport`, `OverrideIssue`, `OverrideError`,
   `ResolvedEntity`).
2. `lib.rs` exports the module via `pub mod scene_instance_overrides;` plus
   re-exports of the public types.
3. New integration test file `crates/editor-core/tests/scene_instance_overrides.rs`
   contains the 10 Given/When/Then scenarios above as passing tests on
   `wasm32-unknown-unknown`.
4. `S3/S4` in `docs/sddk/scene-asset-document/spec.md` are updated to use
   the namespaced `type_id` form (`["editor.Sprite2D", "asset"]`).
5. For every `resync` call, `report.active + report.orphaned + report.stale
   + report.conflict + report.rebound` equals the total patch count from
   `instance.overrides` plus `instance.orphaned_overrides` pre-resync.
6. `effective_values` never mutates `instance`; the caller persists `id_map`
   from the returned `ResolvedScene`.
7. WASM build passes: `cargo check`, `cargo test --no-run`, `cargo fmt --check`.

---

## §5. Open Questions for Design

1. **`OverrideIssue.code`** — typed enum (`MissingEntity` / `MissingComponent`
   / `MissingField` / `TypeConflict` / `DuplicateField`) vs flat `String` code?
   The task brief uses `String`; the Fase 0 explore sketch used a typed enum.
   Design phase must pick. **Recommendation:** typed enum for exhaustive
   matching; a `code: String` accessor satisfies any string-shaped consumers.

2. **`effective_values` error path** — when does it return `Err(OverrideError)`
   vs. routing per-patch failures into `resolved.unresolved_patches`?
   **Recommendation:** `Err` only for structurally-invalid assets
   (e.g. empty `entities`, malformed components). All per-patch failures route
   to `unresolved_patches`. Status mutation is exclusively `resync`'s job.

3. **`minted_stable_ids` persistence model** — does `effective_values` write
   through `instance.id_map` (in-place) or return a set for the caller to
   persist? **Recommendation:** module never mutates `instance`; caller
   assigns `instance.id_map = resolved.id_map` after the call. This keeps
   the module free of `&mut` on `SceneInstance` for the read-only path and
   preserves a clear separation between `effective_values` (read) and
   `resync` (mutate).