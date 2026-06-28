# Kernel Exploration: scene-instance-overrides (Fase 3)

> Change: `scene-instance-overrides` · Phase: explore · Mode: engram
> topic_key: `sddk/scene-instance-overrides/explore` · capture_prompt: false

## Context Quality

- **Level: C2** — types from Fase 0 are in tree and tested; ADR-0005 §Overrides/§Versioning
  is the authoritative contract; cross-editor lessons are documented but the *algorithms*
  (`effective_values`, `resync`, `mint_id_map`) are greenfield. One ambiguity in the
  spike's `field_path` format must be resolved before spec (see Risks §1).
- **Evidence Present**: `crates/editor-core/src/{scene_asset,scene_instance,scene_asset_catalog,document,bsn_ir}.rs`;
  `docs/adr/0005…md` §Overrides, §Versioning and Resync; Fase 0/1/2 archive-reports;
  `patch_status_after_field_rename` helper (Fase 0 brainstorm artifact).
- **Missing Context**: no `RenamedField` event stream exists — renames are detected by
  scan, not by consuming an event (confirmed: no such type in any crate).
- **Recommended Effort: deepen** — algorithms are the deliverable; no external research
  needed beyond confirming the Unity/Blender patterns already cited in Fase 0 explore.

## Current State

Fase 0 shipped the **type layer only** (`scene_asset.rs:175`, `scene_instance.rs:55`,
`bsn_ir.rs:133`). The types are inert: `OverridePatch` carries a `status: OverrideStatus`
field but nothing mutates it after construction except the pure helper
`patch_status_after_field_rename` (`scene_instance.rs:45-54`), which only covers the
single transition `Active → Stale` triggered by an externally-supplied rename pair.

Fase 1 (`bsn_codegen.rs`) and Fase 2 (`scene_asset_catalog.rs:315`) shipped adjacent
machinery but explicitly deferred instance resolution (both archive-reports §What's Next
point at "Fase 3 — SceneInstance Override Resolution").

The `SceneAssetCatalog` (`scene_asset_catalog.rs:148-167`) resolves `logical_path →
asset_id` and indexes by role, but it stores **metadata only**
(`SceneAssetCatalogEntry`, no document body). So any algorithm that needs the actual
component values must receive a `&SceneAssetDocument` from a separate loader; the catalog
cannot serve it. This shapes the API: every function takes `asset: &SceneAssetDocument`
as an explicit parameter.

`ComponentInstance.values` is `serde_json::Value` (`document.rs:104-109`), so a
`field_path: Vec<String>` is a JSON-pointer traversal over an object tree — **not** a
Bevy reflected path, **not** a dotted string.

## Affected Areas

- `crates/editor-core/src/scene_instance.rs` — add `effective_values`, `resync`,
  `mint_id_map`, `reconcile_id_map`, `validate_overrides` plus `ResolvedScene`,
  `ResyncReport`, `OverrideIssue`, `OverrideError` types. Keep
  `patch_status_after_field_rename` (build on it, don't replace).
- `crates/editor-core/src/lib.rs` — one new `pub mod` line for the instance-resolution
  module (likely `pub mod scene_instance_resolution;` or fold into `scene_instance`).
- `crates/editor-core/src/scene_asset.rs` — **read-only consumer**; no edits expected.
- `crates/editor-core/src/scene_asset_catalog.rs` — **read-only consumer**; no edits.
- New test file `crates/editor-core/tests/scene_instance_resolution.rs`.

## Override Lifecycle (state machine)

Four states per ADR-0005 §Overrides (`scene_instance.rs:13-18`). Transitions:

```
                     mint/spawn
                        │
                        ▼
                   ┌─────────┐
        ┌───────── │ Active  │ ──────────┐
        │          └─────────┘            │
        │ field renamed/removed           │ entity removed
        │ (any segment fails              │ (target_local_id
        │  to resolve in values)          │  not in asset)
        ▼                                 ▼
   ┌────────┐                       ┌──────────┐
   │ Stale  │                       │ Orphaned │
   └────────┘                       └──────────┘
        │                                 │
        │ field type changed              │ rebind heuristic
        │ (Value kind mismatch)           │ succeeds (local_path
        │                                 │  suffix match)
        ▼                                 │
   ┌──────────┐                           │
   │ Conflict │ ◄─────────────────────────┘
   └──────────┘     (rebind found a field
        │           but value kind differs)
        │
        ▼
   [user resolves via explicit UI action — NOT auto in Fase 3]
```

**Triggers** (per ADR-0005 §Versioning and Resync):
- `Active → Stale`: a `field_path` segment no longer matches any key in the resolved
  `serde_json::Value` object (rename or removal of a field).
- `Active → Orphaned`: `target_local_id` not present in `asset.entities`.
- `Active → Conflict`: target resolves but the override's `value` `serde_json::Value` kind
  is incompatible with the asset's current value kind at that path (e.g. asset has
  `f32`, override carries `String`).
- `Orphaned → Active`: rebinding succeeds (entity reappeared with matching `local_path`
  suffix).
- `Stale/Conflict → Active`: only via explicit user action (re-target). **Out of scope
  for Fase 3 spike** — Fase 3 *detects* and *surfaces*, does not auto-resolve.

**Non-destructive invariant** (ADR-0005 §Overrides line 80): the editor must NOT silently
delete override data. `orphaned_overrides: Vec<OverridePatch>` (`scene_instance.rs:39`)
is the retention buffer; `resync` moves patches between `overrides` and
`orphaned_overrides` but never drops them.

## `effective_values(asset, instance)` — core merge

1. Clone the asset's entity/component tree into a `ResolvedScene` (same shape as
   `SceneAssetDocument` — entities + relationships — but values are the *effective*
   post-override values).
2. For each `OverridePatch` in `instance.overrides` where `status == Active`:
   - Resolve `target_local_id` → entity in `resolved.entities`.
   - Navigate `field_path` segments into the entity's `ComponentInstance.values` JSON
     (see Field-Path Format below).
   - If any segment misses → collect into `unresolved_patches`, do NOT apply, do NOT
     mutate status (this is a read-only compute; status mutation is `resync`'s job).
   - If the terminal value's `serde_json::Value` kind is incompatible with the override
     value kind → collect as `OverrideIssue::TypeConflict`, do NOT apply.
   - Else overwrite the terminal value with `patch.value`.
3. Return `ResolvedScene { entities, relationships, unresolved_patches }`.

**Field-Path Format** (must be locked in spec):
The spike's `field_path: ["Sprite2D", "asset"]` (`spec.md:59`, S3) is ambiguous because
`ComponentInstance.type_id` is namespaced (`"editor.Sprite2D"`, `document.rs:106`).
Two readings:
- **(A)** Segment 0 is a component selector (short suffix of `type_id`); segments 1..N
  navigate `values`.
- **(B)** Segment 0 is the full `type_id`; segments 1..N navigate `values`.

**Recommendation: lock (B)** — segment 0 is the full `type_id` string. It's unambiguous,
matches `ComponentInstance.type_id` exactly, and requires no normalization table. Update
S3/S4 fixtures accordingly in the Fase 3 spec (they currently use the short form, which
was a spike-era convenience). Flag this as a spec-time decision, not an explore-time one.

## `resync(asset, instance, prev_version, new_version)` — algorithm

Per ADR-0005 §Versioning and Resync (lines 99-107): conservative auto-resync on open when
`asset_version_seen < asset.version`.

```
for patch in instance.overrides (clone):
    entity = asset.entities.find(local_id == patch.target_local_id)
    if entity is None:
        move patch to orphaned_overrides; status = Orphaned; continue

    resolved_value = navigate(entity, patch.field_path)
    if resolved_value is None (any segment missed):
        status = Stale; continue

    if value_kind(resolved_value) != value_kind(patch.value):
        status = Conflict; continue

    status = Active  (was possibly Stale/Conflict from prior resync — re-validate)

for patch in instance.orphaned_overrides (drain into temp):
    entity = asset.entities.find(local_id == patch.target_local_id)
    if entity is None: keep in orphaned_overrides; continue
    # Rebind heuristic: entity reappeared. Re-validate field_path.
    resolved = navigate(entity, patch.field_path)
    if resolved is None: status = Stale; move to overrides; continue
    if kind mismatch: status = Conflict; move to overrides; continue
    status = Active; move to overrides; rebound += 1

instance.asset_version_seen = new_version
return ResyncReport { active, orphaned, stale, conflict, rebound }
```

**Conservative rule** (ADR-0005 line 103-106): safe changes apply automatically; uncertain
data becomes `stale` or `conflict`; **no destructive cleanup**; surface a visible report.

**Rebind heuristic**: match by `target_local_id` equality first (cheapest, exact). Only
fall back to `local_path` suffix matching if `target_local_id` is absent — and even then,
only against entities whose `local_path` suffix equals the orphaned entity's last
recorded `local_path` segment. This is deliberately conservative; aggressive fuzzy
matching is out of scope.

## `mint_id_map` and `reconcile_id_map`

```
mint_id_map(asset, mint: impl FnMut() -> StableId) -> BTreeMap<LocalId, StableId>
    for entity in asset.entities:
        map.insert(entity.local_id, mint())
    return map
```

```
reconcile_id_map(asset, existing, mint) -> BTreeMap<LocalId, StableId>
    result = existing.clone()
    for entity in asset.entities:
        if !result.contains_key(entity.local_id):
            result.insert(entity.local_id, mint())  # new entity in asset
    # Entities removed from asset: keep their entries (non-destructive).
    # The instance's orphaned_overrides retain their target_local_id references;
    # removing map entries would orphan them structurally.
    return result
```

The `mint` callback keeps both functions pure and testable (inject a counter or UUID
generator). `SceneInstance.id_map` (`scene_instance.rs:35`) is the storage site; the
caller assigns the returned map.

## Cross-Editor Lessons (concrete patterns, building on Fase 0 explore §Cross-Editor)

- **Unity** (`PrefabUtility`, `PropertyModification`): overrides survive source changes;
  "unused overrides" are retained and surfaced as "missing" in the inspector — never
  auto-deleted. This is the direct precedent for `orphaned_overrides` retention
  (ADR-0005 §Overrides line 80). Fase 3 mirrors this: `resync` moves to
  `orphaned_overrides`, never deletes.
- **Blender** (Library Overrides `residual_storage`): unmatched override data after
  resync is explicitly stored as "residual" rather than discarded. Maps to our
  `orphaned_overrides` buffer. Blender also makes resync an explicit operator, not
  automatic — ADR-0005 §Versioning chooses *conservative auto-resync on open*, a
  deliberate divergence.
- **Godot** (`PackedScene`, `editable_children`): inherited scene overrides are silently
  lost when source removes a node. This is the **anti-pattern** ADR-0005 avoids —
  Fase 3 must never silently drop.
- **Defold** (collection factory `id_map`): `collection[N]/object` → runtime ID mapping
  is explicit, not derived. Validates our `mint_id_map` / `reconcile_id_map` shape.

## API Shape Proposal (sketches; design locks)

```rust
pub struct ResolvedScene { pub entities: Vec<SceneAssetEntity>,
                           pub relationships: Vec<SceneAssetRelationship>,
                           pub unresolved_patches: Vec<OverrideIssue> }

pub struct ResyncReport { pub active: usize, pub orphaned: usize,
                          pub stale: usize, pub conflict: usize, pub rebound: usize }

pub enum OverrideIssue { FieldNotFound { local_id, field_path },
                         TypeConflict  { local_id, field_path, expected, got },
                         OrphanedEntity { local_id } }

pub enum OverrideError { AssetEntityMissing(LocalId),
                         FieldPathRootNotComponent(String) }

pub fn effective_values(asset: &SceneAssetDocument, instance: &SceneInstance)
    -> Result<ResolvedScene, OverrideError>;
pub fn resync(asset: &SceneAssetDocument, instance: &mut SceneInstance,
              prev_version: u32, new_version: u32) -> ResyncReport;
pub fn mint_id_map(asset: &SceneAssetDocument, mint: impl FnMut() -> StableId)
    -> BTreeMap<LocalId, StableId>;
pub fn reconcile_id_map(asset: &SceneAssetDocument,
                        existing: &BTreeMap<LocalId, StableId>,
                        mint: impl FnMut() -> StableId) -> BTreeMap<LocalId, StableId>;
pub fn validate_overrides(asset: &SceneAssetDocument, instance: &SceneInstance)
    -> Vec<OverrideIssue>;
```

## Out of Scope (spike discipline)

- No commands / undo / operation-log integration.
- No frontend / inspector / UI surfacing of orphaned overrides.
- No `SceneAssetDocument` body I/O (OPFS load/save).
- No `bsn!` codegen from a `SceneInstance` (Fase 4+; current codegen is document-only).
- No Scene Asset Variants / inheritance.
- No `SceneAssetCatalog` OPFS persistence.
- No async / parallel resync (single-threaded WASM target).
- No auto-resolution of `Stale`/`Conflict` (detect + surface only).
- No `AssetReference → Handle<Image>` resolution.

## Risks (top 7)

1. **Field-path format ambiguity** — spike uses short component names (`"Sprite2D"`),
   but `type_id` is namespaced (`"editor.Sprite2D"`). Must lock full-`type_id` segment 0
   in spec. Affects every algorithm.
2. **No `RenamedField` event stream** — renames detected by full scan per resync, not
   consumed from an event log. O(N·M) where N=patches, M=entity fields. Documented
   acceptable for spike; revisit if asset grows past hundreds of entities.
3. **Conflict detection via `serde_json::Value` kind** — comparing `Value::is_number()`
   vs `is_string()` etc. is coarse: `f32` vs `i64` both look "number". Acceptable for
   spike (component schemas are typed separately); full type-aware conflict detection
   requires consulting `ComponentSchemaRegistry` (`schema.rs:60`), deferred.
4. **Rebind heuristic aggressiveness** — `local_path` suffix match can false-positive on
   common leaf names ("root", "weapon"). Default to `target_local_id` exact match only;
   `local_path` fallback is opt-in and surfaces a warning in `ResyncReport`.
5. **Override ordering** — multiple `OverridePatch` entries with identical
   `(target_local_id, field_path)` in `instance.overrides`. Decision: **later wins**
   (Vec order), detected by `validate_overrides` as an `OverrideIssue::DuplicateField`.
   Do not silently merge.
6. **`id_map` growth on entity churn** — `reconcile_id_map` never removes entries
   (non-destructive). Long-lived instances with heavy asset churn accumulate stale map
   entries. Acceptable for spike; document the limit.
7. **`ResolvedScene` vs `SceneAssetDocument` shape duplication** — tempting to reuse
   `SceneAssetDocument` as the resolved output, but effective values are post-override
   (mutable view), not authoring truth. Keep `ResolvedScene` as a distinct projection
   type to preserve the source-of-truth boundary (ADR-0005 §Decision).

## Ready for Proposal

**Yes.** The algorithms are well-bounded by ADR-0005 §Overrides and §Versioning, the
types exist, and the only spec-time decision is the field-path segment-0 convention
(full `type_id` recommended). Orchestrator should tell the user: Fase 3 delivers the
override lifecycle algorithms as pure functions with no UI/commands/persistence — the
"meaty" logic without the integration surface.

---

## Standard Envelope

- **status**: `success`
- **executive_summary**: Fase 3 algorithms (`effective_values`, `resync`, `mint_id_map`,
  `reconcile_id_map`, `validate_overrides`) are well-defined by ADR-0005 §Overrides/§Versioning
  and build directly on the Fase 0 type layer. One spec-time decision (field-path
  segment-0 = full `type_id`) and seven risks documented. No external research needed.
- **context_quality**: `C2`
- **taxonomy**: dominant axes = `override_lifecycle` (state machine), `non_destructive_retention`
  (orphaned_overrides buffer), `conservative_resync` (no auto-cleanup), `id_map_durability`.
- **next_recommended**: `propose`
- **evidence_citations**:
  - `docs/adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md` §Overrides (L76-83),
    §Versioning and Resync (L97-108)
  - `crates/editor-core/src/scene_instance.rs` L10-54 (OverrideStatus, OverridePatch,
    patch_status_after_field_rename)
  - `crates/editor-core/src/scene_asset.rs` L52-103 (SceneAssetDocument, SceneAssetEntity,
    ExposedProperty)
  - `crates/editor-core/src/scene_asset_catalog.rs` L148-186 (resolve_path, broken_references)
  - `crates/editor-core/src/document.rs` L104-109 (ComponentInstance.values: serde_json::Value)
  - `docs/sddk/scene-asset-document/spec.md` S3/S4 (field_path spike convention)
  - `docs/sddk/scene-asset-document/explore-report.md` §Cross-Editor Lessons
  - `docs/sddk/scene-asset-catalog/archive-report.md` §What's Next (Fase 3 pointer)
- **risks**: see §Risks above (7 items)
- **out_of_scope**: see §Out of Scope above (9 items)
- **engram_save_topic_key**: `sddk/scene-instance-overrides/explore`
- **capture_prompt**: false
