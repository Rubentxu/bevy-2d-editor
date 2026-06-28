# Proposal: Scene Instance Override Resolution (Fase 3)

> Change: `scene-instance-overrides` · Phase: propose · Mode: engram
> Source explore: [`explore-report.md`](./explore-report.md)

## Intent

Fase 0 shipped the Scene Instance **type layer** (`OverridePatch`, `OverrideStatus`,
`SceneInstance`, `patch_status_after_field_rename`) as inert data with exactly one
`Active → Stale` transition helper. Nothing computes effective values, detects
orphaned/stale/conflict overrides, resyncs against a new asset version, or mints
the `id_map` that binds asset-local IDs to scene Stable IDs.

This change adds a **pure-functions module** that delivers the override lifecycle
algorithms contracted by ADR-0005 §Overrides and §Versioning: merge, classify,
resync, rebind, mint, reconcile. No commands, no UI, no persistence, no codegen.
Orphaned/stale/conflict data is **surfaced, never silently deleted** — the
Godot anti-pattern ADR-0005 explicitly rejects.

## Scope

### In Scope
- New module `crates/editor-core/src/scene_instance_overrides.rs` with 7 pure functions.
- Types: `ResolvedScene`, `ResolvedEntity`, `ResyncReport`, `OverrideIssue`, `OverrideError`.
- `pub mod scene_instance_overrides;` wired into `lib.rs`.
- Integration test file `crates/editor-core/tests/scene_instance_overrides.rs` (11 tests).
- Non-destructive resync: patches move between `overrides` and `orphaned_overrides`, never dropped.

### Out of Scope
- No commands / undo / operation-log integration.
- No frontend / inspector / UI surfacing of orphaned overrides.
- No `SceneAssetDocument` body I/O (OPFS load/save).
- No `bsn!` codegen from a `SceneInstance` (Fase 4+).
- No Scene Asset Variants / inheritance.
- No async / parallel resync (single-threaded WASM target).
- No auto-resolution of `Stale`/`Conflict` (detect + surface only).
- No value-based rebinding (local_path suffix only for spike).

## Capabilities

> CONTRACT with sddk-spec. Research `docs/sddk/` before filling in.

### New Capabilities
- `scene-instance-overrides`: pure-function override lifecycle — `effective_values`
  (merge), `resync` (re-validate on asset version bump), `mint_id_map` /
  `reconcile_id_map` (id_map durability), `validate_overrides` (read-only issue scan),
  `classify_overrides` + `try_rebind` (status machinery). Returns `ResolvedScene`,
  `ResyncReport`, `OverrideIssue`. Non-destructive retention invariant enforced.

### Modified Capabilities
- `scene-asset-document`: **field-path convention lock.** Existing scenarios S3/S4
  (`spec.md:59,66`) use the short form `["Sprite2D", "asset"]`. The spike locks
  segment-0 = **full namespaced `type_id`** (`["editor.Sprite2D", "asset"]`).
  S3/S4 fixtures update to the namespaced form; requirement `override-patch-targeting`
  gains a note that segment-0 matches `ComponentInstance.type_id` exactly.

## Approach

New module owns all algorithms; `scene_asset.rs` and `scene_asset_catalog.rs` are
**read-only consumers** (no edits). `scene_instance.rs` keeps its types; the new
module imports them. `patch_status_after_field_rename` is retained (built on, not
replaced). `ResolvedScene` is a distinct projection type — it is NOT
`SceneAssetDocument` reused — to preserve the source-of-truth boundary
(ADR-0005 §Decision; explore §Risk 7).

Public API (signatures locked, bodies deferred to design/spec):

```rust
pub fn effective_values(asset, instance, mint: &mut dyn FnMut() -> StableId)
    -> Result<ResolvedScene, OverrideError>
pub fn resync(asset, instance: &mut SceneInstance, new_asset_version: u32) -> ResyncReport
pub fn mint_id_map(asset, mint: &mut dyn FnMut() -> StableId) -> BTreeMap<LocalId, StableId>
pub fn reconcile_id_map(asset, existing, mint) -> BTreeMap<LocalId, StableId>
pub fn validate_overrides(asset, instance) -> Vec<OverrideIssue>
pub fn classify_overrides(asset, patches: &[OverridePatch]) -> Vec<OverridePatch>
pub fn try_rebind(asset, orphaned: &OverridePatch) -> Option<LocalId>
```

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/scene_instance_overrides.rs` | New | 7 functions + 5 types. |
| `crates/editor-core/src/lib.rs` | Modified | +`pub mod scene_instance_overrides;` + re-exports. |
| `crates/editor-core/tests/scene_instance_overrides.rs` | New | 11 tests (see explore §API). |
| `docs/sddk/scene-asset-document/spec.md` | Modified | S3/S4 field-path fixtures → namespaced `type_id`. |
| `crates/editor-core/src/scene_instance.rs` | Read-only consumer | Types only; no edits. |
| `crates/editor-core/src/scene_asset.rs` | Read-only consumer | Types only; no edits. |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Field-path ambiguity (short vs namespaced) — affects every algorithm. | High | **Locked here: segment-0 = full `type_id`.** Spec updates S3/S4. |
| Conflict detection via `serde_json::Value` kind is coarse (`f32`≡`i64`). | Medium | Documented acceptable for spike; full type-aware detection needs `ComponentSchemaRegistry`, deferred. |
| Rebind `local_path` suffix false-positive on common leaf names. | Medium | Exact `target_local_id` match first; suffix fallback opt-in. |
| No `RenamedField` event stream → O(N·M) full scan per resync. | Low | Acceptable for spike (hundreds of entities); revisit if asset grows. |
| `id_map` unbounded growth on entity churn (`reconcile` never removes). | Low | Non-destructive by design; documented limit; cleanup is a future explicit action. |

## Rollback Plan

This is additive (new module + one `pub mod` line + one new test file). Revert =
delete `scene_instance_overrides.rs`, remove the `pub mod` line + re-exports from
`lib.rs`, delete the test file, revert the S3/S4 fixture change in
`scene-asset-document/spec.md`. No data migration needed — the module produces
derived data; removing it returns the editor to the Fase 0 inert-types state.

## Dependencies
- Fase 0 types: `SceneAssetDocument`, `SceneAssetEntity`, `SceneInstance`,
  `OverridePatch`, `OverrideStatus`, `LocalId`, `ComponentInstance`, `StableId`
  — all shipped and tested.
- ADR-0005 §Overrides / §Versioning and Resync — the authoritative contract.
- `thiserror` crate (already a workspace dependency).

## Success Criteria
- [ ] `cargo test -p editor-core` passes with the 11 new tests green.
- [ ] `effective_values` with 0 overrides returns the asset unchanged.
- [ ] Removing an asset entity routes its patch to `unresolved` / `orphaned_overrides` — never panic, never drop.
- [ ] `resync` produces a `ResyncReport` whose counts sum to the total patch count (active + orphaned + stale + conflict + rebound = total).
- [ ] `mint_id_map` yields N distinct StableIds for N entities.
- [ ] No public function mutates state outside the `SceneInstance` passed in by `&mut`.
- [ ] S3/S4 in `scene-asset-document/spec.md` use the namespaced `type_id` segment-0.

---

## Standard Envelope

- **status**: `success`
- **executive_summary**: Fase 3 delivers the override lifecycle as pure functions
  (`effective_values`, `resync`, `mint_id_map`, `reconcile_id_map`,
  `validate_overrides`, `classify_overrides`, `try_rebind`) in a new module.
  Builds directly on Fase 0 types and ADR-0005 §Overrides/§Versioning. One
  contract decision locked (field-path = namespaced `type_id`); 5 risks documented.
- **capabilities**: new = 1 (`scene-instance-overrides`), modified = 1 (`scene-asset-document` field-path lock)
- **context_quality**: `C2`
- **risk_level**: Medium
- **taxonomy**: `override_lifecycle` (state machine), `non_destructive_retention`, `conservative_resync`, `id_map_durability`
- **next_recommended**: `spec`
- **decisions_made**:
  - Field-path segment-0 = **full namespaced `type_id`** (`"editor.Sprite2D"`), exact match against `ComponentInstance.type_id`. Short form retired.
  - Conflict detection = **`serde_json::Value` kind compare** (looser: `is_number` vs `is_string`), not exact-value compare. Type-aware detection deferred to `ComponentSchemaRegistry` integration.
  - Rebind = **`target_local_id` exact match first, `local_path` suffix fallback only**. Value-based rebinding out of scope for spike.
  - New standalone module `scene_instance_overrides.rs`; not folded into `scene_instance.rs`.
  - `ResolvedScene` is a **distinct projection type**, not `SceneAssetDocument` reuse (source-of-truth boundary).
  - Override ordering on duplicate `(target_local_id, field_path)` = **later-wins**; duplicates surfaced by `validate_overrides`, not silently merged.
  - Non-destructive invariant: `resync` moves patches between `overrides` ↔ `orphaned_overrides`; never deletes. `reconcile_id_map` never removes entries.
- **open_questions_for_spec**:
  1. `OverrideIssue.code`: typed enum (`MissingEntity`/`MissingComponent`/`MissingField`/`TypeConflict`) vs flat `String` code? Task brief uses `String`; explore sketch used typed enum. Spec must pick — recommend typed enum for exhaustive matching.
  2. `effective_values` error path: when does `OverrideError::{EmptyAsset, MultipleRoots}` fire (early `Err`) vs. routing bad patches to `unresolved` (`Vec`)? Spec must define the boundary — propose: `Err` only for structurally-invalid assets, all per-patch failures → `unresolved`.
  3. `ResolvedScene.minted_stable_ids`: does `effective_values` write through `instance.id_map` or return a set for the caller to persist? Task says caller persists; spec should state the ownership contract explicitly.
- **engram_save_topic_key**: `sddk/scene-instance-overrides/propose`
- **capture_prompt**: false
