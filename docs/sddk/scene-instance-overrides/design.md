# Design: Scene Instance Override Resolution (Fase 3)

> Change: `scene-instance-overrides` · Phase: design · Mode: engram
> Source: [`explore-report.md`](./explore-report.md), [`proposal.md`](./proposal.md)
> topic_key: `sddk/scene-instance-overrides/design` · capture_prompt: false

## Technical Approach

A new **pure-functions module** (`scene_instance_overrides.rs`) delivers the override
lifecycle algorithms contracted by ADR-0005 §Overrides/§Versioning: merge
(`effective_values`), re-validate-on-bump (`resync`), id-map durability
(`mint_id_map`/`reconcile_id_map`), status machinery (`classify_overrides`/
`try_rebind`), and read-only issue scan (`validate_overrides`). Every function
takes `&SceneAssetDocument` explicitly (the catalog stores metadata only —
explore-report §Current State). No commands, no UI, no persistence, no codegen.
Non-destructive invariant: `resync` moves patches between `overrides` and
`orphaned_overrides`; never drops.

**Field-path convention (locked in proposal)**: `field_path[0]` = full namespaced
`type_id` (e.g. `"editor.Sprite2D"`), matching `ComponentInstance.type_id` exactly.
Segments `[1..]` navigate `values` JSON. Short form (`"Sprite2D"`) is retired.

## Architecture Decisions

### Decision: `ResolvedScene` is a distinct projection, not `SceneAssetDocument` reuse

**Choice**: New `ResolvedScene { entities: BTreeMap<LocalId, ResolvedEntity>, ... }`.
**Alternatives**: Reuse `SceneAssetDocument` as the output.
**Rationale**: Effective values are a post-override mutable view, not authoring
truth. Reusing the source-of-truth type blurs the boundary (ADR-0005 §Decision;
explore §Risk 7). `ResolvedEntity` drops `metadata`/`exposed_properties` (editor-UI
fields, not runtime).

### Decision: Conflict detection = `serde_json::Value` kind compare (coarse)

**Choice**: Compare `json_kind(existing)` vs `json_kind(patch.value)` where
`json_kind` returns `"number" | "string" | "boolean" | "array" | "object" | "null"`.
`Number(42)` ≡ `Number(42.0)` (both `"number"`).
**Alternatives**: Type-aware compare via `ComponentSchemaRegistry` field types.
**Rationale**: Spike discipline — schema registry integration is out of scope.
Coarse kind compare catches the common case (String vs number) without a schema
lookup. Full type-aware detection deferred (proposal §Risks).

### Decision: `StableId` must gain `Ord, PartialOrd` derives

**Choice**: Add `PartialOrd, Ord` to `StableId`'s derive list in `document.rs:13`.
**Alternatives**: Change `minted_stable_ids` to `HashSet<StableId>` (StableId has
`Hash`), or `Vec<StableId>`.
**Rationale**: `BTreeSet<StableId>` (task contract) requires `Ord`. `String` (inner)
has `Ord`. Adding two derive tokens is non-breaking and future-proof (sorted output,
deterministic iteration). `LocalId` already has `Ord`.

### Decision: `try_rebind` = exact `target_local_id` match only (spike)

**Choice**: Return `Some(target_local_id)` iff `find_entity(asset, &target_local_id)`
returns `Some`. No `local_path` suffix fallback.
**Alternatives**: `local_path` suffix match via `build_path_index` + `suffix_match`.
**Rationale**: The orphaned `OverridePatch` stores only `target_local_id`, not a
prior `local_path`. Without storing the old path, suffix matching has no input.
Path-based rebind is deferred to a future change that records `local_path_at_orphan`
on `OverridePatch`. Helpers (`build_path_index`, `suffix_match`) are scaffolded but
unused by the spike's `try_rebind`.

## Data Flow

```
  SceneAssetDocument ─┬── effective_values(asset, instance, mint) ──→ ResolvedScene
                      │         │                                      (effective values
  SceneInstance ──────┘         │                                       + id_map + minted)
                                │
  SceneAssetDocument ──── resync(asset, &mut instance, new_ver)
                      │         │
                      │    classify_overrides ──→ patches w/ updated status
                      │         │
                      │    ┌────┴──────────────────────┐
                      │    ▼                           ▼
                      │  overrides              orphaned_overrides
                      │  (Active/Stale/         (Orphaned)
                      │   Conflict)                │
                      │                    try_rebind ──→ Option<LocalId>
                      │                         │ rebound
                      │    reconcile_id_map ←────┘
                      │         │
                      └──── instance.id_map updated
                                │
                          ResyncReport
```

## Module Placement & Imports

New file: `crates/editor-core/src/scene_instance_overrides.rs`.
Modify `lib.rs`: add `pub mod scene_instance_overrides;` (after line 19,
alongside `scene_instance`).

```rust
use std::collections::{BTreeMap, BTreeSet};
use crate::document::{ComponentInstance, StableId};
use crate::scene_asset::{
    LocalId, SceneAssetDocument, SceneAssetEntity,
    SceneAssetRelationship, RelationshipKind,
};
use crate::scene_instance::{SceneInstance, OverridePatch, OverrideStatus};
```

> **Note**: `SceneAssetRelationship` and `RelationshipKind` are imported per task
> contract but are unused by the spike's public API (no function reads
> relationships). They scaffold future hierarchy-aware resolution. `#[allow(unused_imports)]`
> or removal at implementation time — see Open Questions.

## Public Types

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedScene {
    pub entities: BTreeMap<LocalId, ResolvedEntity>,
    pub id_map: BTreeMap<LocalId, StableId>,
    pub minted_stable_ids: BTreeSet<StableId>,
    pub unresolved: Vec<OverridePatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedEntity {
    pub local_id: LocalId,
    pub local_path: String,
    pub name: String,
    pub components: Vec<ComponentInstance>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResyncReport {
    pub active: usize,
    pub orphaned: usize,
    pub stale: usize,
    pub conflict: usize,
    pub rebound: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverrideIssue {
    pub code: String,
    pub patch: OverridePatch,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OverrideError {
    #[error("empty asset: no entities")]
    EmptyAsset,
    #[error("multiple roots: only single-root assets supported in spike")]
    MultipleRoots,
}
```

`OverrideIssue.code` uses flat `String` (proposal open-question #1 resolved:
flat string per task contract). Codes: `"missing_entity"`, `"missing_component"`,
`"missing_field"`, `"type_conflict"`, `"duplicate_field"`.

## Public Functions

```rust
pub fn effective_values(
    asset: &SceneAssetDocument,
    instance: &SceneInstance,
    mint_stable_id: &mut dyn FnMut() -> StableId,
) -> Result<ResolvedScene, OverrideError>

pub fn resync(
    asset: &SceneAssetDocument,
    instance: &mut SceneInstance,
    new_asset_version: u32,
) -> ResyncReport

pub fn mint_id_map(
    asset: &SceneAssetDocument,
    mint_stable_id: &mut dyn FnMut() -> StableId,
) -> BTreeMap<LocalId, StableId>

pub fn reconcile_id_map(
    asset: &SceneAssetDocument,
    existing: &BTreeMap<LocalId, StableId>,
    mint_stable_id: &mut dyn FnMut() -> StableId,
) -> BTreeMap<LocalId, StableId>

pub fn validate_overrides(
    asset: &SceneAssetDocument,
    instance: &SceneInstance,
) -> Vec<OverrideIssue>

pub fn classify_overrides(
    asset: &SceneAssetDocument,
    patches: &[OverridePatch],
) -> Vec<OverridePatch>

pub fn try_rebind(
    asset: &SceneAssetDocument,
    orphaned: &OverridePatch,
) -> Option<LocalId>
```

## Private Helpers

```rust
fn find_entity<'a>(asset: &'a SceneAssetDocument, local_id: &LocalId)
    -> Option<&'a SceneAssetEntity>
fn find_component_mut(entity: &mut SceneAssetEntity, type_id: &str)
    -> Option<&mut ComponentInstance>
fn apply_field_path(component: &mut ComponentInstance,
    field_path: &[String], value: serde_json::Value) -> Result<(), ()>
fn detect_kind_mismatch(existing: &serde_json::Value, patch: &serde_json::Value) -> bool
fn json_kind(v: &serde_json::Value) -> &'static str   // see note below
fn build_path_index(asset: &SceneAssetDocument) -> BTreeMap<String, LocalId>
fn suffix_match(orphan_path: &str, candidate_path: &str) -> bool
```

**`json_kind`** — `serde_json::Value` has no `.kind()` method. Implement via
`is_null/is_bool/is_number/is_string/is_array/is_object` → return
`"null" | "boolean" | "number" | "string" | "array" | "object"`.

**`apply_field_path`** — operates on `field_path[1..]` (post-type_id). `len == 1`:
`values.as_object_mut()?.insert(seg, value)`. `len > 1`: walk nested objects, `Err`
if any intermediate segment is missing or not an object.

**`classify_overrides`** (pure): for each patch — find entity by `target_local_id`
(miss → `Orphaned`); find component by `field_path[0]` == `type_id` (miss →
`Orphaned`); walk `field_path[1..]` in `values` (miss → `Stale`); compare
`json_kind` (mismatch → `Conflict`); else `Active`.

## `resync` Algorithm

1. Set `instance.asset_version_seen = new_asset_version`.
2. Clone `instance.overrides`; reclassify each via `classify_overrides`.
3. For each patch: compare old vs new status.
   - `Active → Orphaned`: move to `orphaned_overrides`; `report.orphaned += 1`.
   - `Active → Stale`/`Conflict`: update in place; `report.stale`/`conflict += 1`.
   - `Active → Active`: `report.active += 1`.
   - Non-Active → new status: update in place; count accordingly.
4. Walk `orphaned_overrides`: if `try_rebind` returns `Some(id)`, set
   `target_local_id = id`, `status = Active`, move back to `overrides`,
   `report.rebound += 1`.
5. `instance.id_map = reconcile_id_map(asset, &instance.id_map, &mut mint)`.
6. Return `report`.

**FnMut borrow note**: `resync` does not take a `mint` parameter — it mints
internally via a counter or delegates to `reconcile_id_map`. If a caller needs
deterministic IDs, `reconcile_id_map` is the injection point. Document the call
ordering: never hold a `&mut` to `mint` across `resync`.

## `effective_values` Algorithm

1. `asset.entities.is_empty()` → `Err(EmptyAsset)`.
2. Build `entities: BTreeMap<LocalId, ResolvedEntity>` from asset (clone components).
3. For each patch in `instance.overrides` where `status != Orphaned`:
   - Find entity; if missing → push to `unresolved`, skip.
   - Find component by `field_path[0]`; walk `field_path[1..]`; if miss → `unresolved`.
   - If `detect_kind_mismatch` → `unresolved`; else `apply_field_path`.
4. `id_map = mint_id_map(asset, mint)`; `minted_stable_ids = id_map.values().collect()`.
5. Return `Ok(ResolvedScene { entities, id_map, minted_stable_ids, unresolved })`.

`MultipleRoots` is declared but **not actively triggered** in the spike (single-root
assumed). Reserved for future guard: count entities not appearing as `to_local_id`
in any `Child` relationship; if > 1 → `Err(MultipleRoots)`.

## Round-Trip Loss Matrix

| Function | Drops | Preserves | Mutates |
|----------|-------|-----------|---------|
| `effective_values` | `metadata`, `exposed_properties` | entity values (effective), components, `id_map` | nothing (read-only compute) |
| `resync` | nothing | all patches | moves `overrides`↔`orphaned_overrides`; may rebind `target_local_id`; updates `asset_version_seen`; extends `id_map` |

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/editor-core/src/scene_instance_overrides.rs` | Create | 7 public fns + 5 types + 7 private helpers |
| `crates/editor-core/src/lib.rs` | Modify | `+pub mod scene_instance_overrides;` after line 19 |
| `crates/editor-core/src/document.rs` | Modify | `StableId` derive: add `PartialOrd, Ord` (1 line) |
| `crates/editor-core/tests/scene_instance_overrides.rs` | Create | 10 tests |

> `document.rs` change is mandatory: `BTreeSet<StableId>` requires `Ord`.
> `StableId(String)` — `String` has `Ord`, so adding the derive is sound and
> non-breaking.

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit (integration file) | Algorithm correctness per spec scenario | `#[test]` with hand-built assets/instances; deterministic `mint` closure (`AtomicU32` counter) |
| Property | `resync` report sum == total patches | Assert `active + orphaned + stale + conflict + rebound == initial_override_count` |

Tests to write (10):

1. `effective_values_minimal` — single entity, single Active override applied.
2. `effective_values_short_form_field_path_orphans` (S2) — short-form `field_path[0]`
   doesn't match namespaced `type_id` → patch lands in `unresolved`.
3. `classify_overrides_namespaced_active` (S1) — full `type_id` segment-0 resolves →
   `Active`.
4. `resync_detects_rename_preserves_override` (S3+S4) — field renamed → `Stale`,
   patch NOT deleted.
5. `resync_moves_to_orphaned_on_entity_removed` (S5) — entity removed → patch moves
   to `orphaned_overrides`.
6. `resync_marks_stale_on_field_rename` (S6) — `field_path` segment missing in
   values → `Stale`.
7. `resync_marks_conflict_on_type_change` (S7) — value kind mismatch → `Conflict`.
8. `resync_rebinds_via_local_path` (S8) — entity reappears with same `local_id` →
   `try_rebind` returns `Some`, `report.rebound == 1`.
9. `effective_values_with_no_overrides` (S9) — empty overrides → asset unchanged.
10. `resync_extends_id_map_on_new_entity` (S10) — new entity in asset →
    `id_map` gains one entry.

## Migration / Rollout

No migration. Additive module + one derive change + one `pub mod` line. Revert =
delete module, remove `pub mod`, revert `StableId` derive, delete test file.

## Open Questions

- [ ] `SceneAssetRelationship`/`RelationshipKind` imports are unused by the spike
      API. Keep with `#[allow(unused_imports)]` or drop? — Recommend drop at impl
      time, re-add when hierarchy-aware resolution arrives.
- [ ] `MultipleRoots` guard: implement root-count check now, or leave untriggered
      for spike? — Recommend leave untriggered (documented); add guard when
      multi-root assets are in scope.
- [ ] `validate_overrides` `DuplicateField` code: task lists it but no scenario
      exercises it. Implement basic later-wins detection or defer? — Defer to
      impl; flag in tasks.

## ADR Candidates

- **Coarse `serde_json` kind-based conflict detection** — hard to reverse (stored
  status `Conflict` depends on this semantics), surprising (f32≡i64), real
  trade-off (accuracy vs schema-registry dependency). → ADR-NNN. The proposal
  captured the decision; an ADR crystallizes the rationale for future schema-aware
  migration.

---

## Standard Envelope

- **status**: `success`
- **executive_summary**: Concrete Rust design for 7 pure functions + 5 types in a
  new `scene_instance_overrides.rs` module. One mandatory micro-change found
  (`StableId` needs `Ord`). Builds directly on Fase 0 types and ADR-0005. 10 tests
  mapped to spec scenarios.
- **context_quality**: `C2`
- **approach**: Pure-functions module with non-destructive override lifecycle.
- **key_decisions**: 4 (distinct projection type, coarse kind-compare conflict,
  `StableId` Ord derive, exact-match rebind).
- **files_affected**: 2 new, 2 modified, 0 deleted.
- **key_type_signatures**:
  - `ResolvedScene { entities, id_map, minted_stable_ids, unresolved }`
  - `ResolvedEntity { local_id, local_path, name, components }`
  - `ResyncReport { active, orphaned, stale, conflict, rebound }`
  - `OverrideIssue { code, patch, message }`
  - `OverrideError { EmptyAsset, MultipleRoots }`
  - `effective_values(asset, instance, mint) -> Result<ResolvedScene, OverrideError>`
  - `resync(asset, &mut instance, new_ver) -> ResyncReport`
  - `mint_id_map(asset, mint) -> BTreeMap<LocalId, StableId>`
  - `reconcile_id_map(asset, existing, mint) -> BTreeMap<LocalId, StableId>`
  - `classify_overrides(asset, patches) -> Vec<OverridePatch>`
  - `try_rebind(asset, orphaned) -> Option<LocalId>`
  - `validate_overrides(asset, instance) -> Vec<OverrideIssue>`
- **helpers_to_implement**: `find_entity`, `find_component_mut`, `apply_field_path`,
  `detect_kind_mismatch`, `json_kind`, `build_path_index`, `suffix_match`.
- **tests_to_write**: `effective_values_minimal`,
  `effective_values_short_form_field_path_orphans`, `classify_overrides_namespaced_active`,
  `resync_detects_rename_preserves_override`, `resync_moves_to_orphaned_on_entity_removed`,
  `resync_marks_stale_on_field_rename`, `resync_marks_conflict_on_type_change`,
  `resync_rebinds_via_local_path`, `effective_values_with_no_overrides`,
  `resync_extends_id_map_on_new_entity`.
- **next_recommended**: `tasks`
- **engram_save_topic_key**: `sddk/scene-instance-overrides/design`
- **capture_prompt**: false
- **risks**:
  - `resync` is O(N×M) per call (N=entities, M=patches); acceptable for spike.
  - `ComponentInstance.values` keys must match `field_path` segments exactly.
  - `mint_stable_id: &mut dyn FnMut` — never hold borrow across nested calls.
