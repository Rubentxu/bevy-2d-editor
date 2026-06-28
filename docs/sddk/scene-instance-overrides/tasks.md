# Tasks: Scene Instance Override Resolution (Fase 3)

> Change: `scene-instance-overrides` · Phase: tasks · Mode: engram
> Source: [`design.md`](./design.md), [`spec.md`](./spec.md), [`proposal.md`](./proposal.md)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 400–600 |
| 400-line budget risk | Medium |
| Chained PRs recommended | No |
| Suggested split | Single PR with stacked commits (T1→T5) |
| Delivery strategy | single-pr |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Medium

> Apply MUST stop and report if actual LOC exceeds 50% over upper bound (900 LOC).

---

## Phase 0: Branch & Pre-flight

- [ ] 0.1 `git checkout -b feat/scene-instance-overrides` from `main` (last commit `3e86431`).
- [ ] 0.2 Pre-flight: `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` → must succeed before any edits.
- [ ] 0.3 Pre-flight: `cargo test --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml --no-run` → baseline must build (no existing test failures).

## Phase 1: Foundation — StableId Ord derive (T1)

- [ ] 1.1 In `crates/editor-core/src/document.rs` line 11, add `PartialOrd, Ord` to the `StableId` derive list: `#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]`.
- [ ] 1.2 Verify: `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` → must still pass.
- [ ] 1.3 Commit: `feat(editor-core): add Ord derives to StableId for BTreeSet usage`.

## Phase 2: Core Implementation — `scene_instance_overrides.rs` (T2)

- [ ] 2.1 Create `crates/editor-core/src/scene_instance_overrides.rs` with module-level doc comment citing `docs/adr/0005…md §Overrides` and `§Versioning and Resync`.
- [ ] 2.2 Add imports: `std::collections::{BTreeMap, BTreeSet}`, `crate::document::{ComponentInstance, StableId}`, `crate::scene_asset::{LocalId, SceneAssetDocument, SceneAssetEntity}`, `crate::scene_instance::{SceneInstance, OverridePatch, OverrideStatus}`.
- [ ] 2.3 Define public types: `ResolvedScene { entities: BTreeMap<LocalId, ResolvedEntity>, id_map: BTreeMap<LocalId, StableId>, minted_stable_ids: BTreeSet<StableId>, unresolved: Vec<OverridePatch> }`, `ResolvedEntity { local_id, local_path, name, components }`, `ResyncReport { active, orphaned, stale, conflict, rebound }` (Default), `OverrideIssue { code: String, patch: OverridePatch, message: String }`, `OverrideError` enum (`EmptyAsset`, `MultipleRoots`) with `thiserror::Error`.
- [ ] 2.4 Implement private helpers: `find_entity`, `find_component_mut`, `apply_field_path` (operates on `field_path[1..]`), `detect_kind_mismatch`, `json_kind` (returns `"number"|"string"|"boolean"|"array"|"object"|"null"`), `build_path_index`, `suffix_match` (scaffolded unused).
- [ ] 2.5 Implement `pub fn classify_overrides(asset, patches: &[OverridePatch]) -> Vec<OverridePatch>` — pure re-classify: entity miss → `Orphaned`, component miss (full `type_id` segment-0) → `Orphaned`, field miss → `Stale`, kind mismatch → `Conflict`, else `Active`.
- [ ] 2.6 Implement `pub fn mint_id_map(asset, mint: &mut dyn FnMut() -> StableId) -> BTreeMap<LocalId, StableId>` — one fresh ID per asset entity.
- [ ] 2.7 Implement `pub fn reconcile_id_map(asset, existing: &BTreeMap<LocalId, StableId>, mint) -> BTreeMap<LocalId, StableId>` — clone existing, mint only for new `LocalId`s; never remove (non-destructive).
- [ ] 2.8 Implement `pub fn validate_overrides(asset, instance: &SceneInstance) -> Vec<OverrideIssue>` — scan for `missing_entity`, `missing_component`, `missing_field`, `type_conflict`, `duplicate_field`; later-wins on duplicates (surface, don't merge).
- [ ] 2.9 Implement `pub fn try_rebind(asset, orphaned: &OverridePatch) -> Option<LocalId>` — exact `target_local_id` match only (spike); `local_path` suffix scaffold unused.
- [ ] 2.10 Implement `pub fn effective_values(asset, instance, mint) -> Result<ResolvedScene, OverrideError>` — `Err(EmptyAsset)` on empty entities; build entity map; apply non-Orphaned patches; per-patch failures → `unresolved`; returns id_map + `minted_stable_ids`. Never mutates `instance`.
- [ ] 2.11 Implement `pub fn resync(asset, instance: &mut SceneInstance, new_asset_version: u32) -> ResyncReport` — set `asset_version_seen`; clone+reclassify; move `overrides`↔`orphaned_overrides` (never delete); rebind via `try_rebind`; extend `instance.id_map` via `reconcile_id_map` with internal counter.
- [ ] 2.12 Verify: `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` → must pass.
- [ ] 2.13 Commit: `feat(editor-core): add scene instance overrides and resync algorithm`.

## Phase 3: Integration — wire into `lib.rs` (T3)

- [ ] 3.1 In `crates/editor-core/src/lib.rs`, add `pub mod scene_instance_overrides;` after line 19 (alongside `scene_instance`).
- [ ] 3.2 Verify: `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` → must pass.
- [ ] 3.3 Commit: `feat(editor-core): wire scene instance overrides module into lib.rs`.

## Phase 4: Testing (T4)

- [ ] 4.1 Create `crates/editor-core/tests/scene_instance_overrides.rs` with deterministic `mint` closure (e.g. `AtomicUsize` counter returning `StableId::new(format!("sid_{n}"))`).
- [ ] 4.2 Test 1 `effective_values_minimal`: single entity, single Active override applied; verify value merge + minted ID count.
- [ ] 4.3 Test 2 `effective_values_short_form_field_path_orphans` (S2): short `field_path[0] = "Sprite2D"` does not match `editor.Sprite2D` → patch in `unresolved`.
- [ ] 4.4 Test 3 `classify_overrides_namespaced_active` (S1): full `type_id` segment-0 → `Active`.
- [ ] 4.5 Test 4 `resync_detects_rename_preserves_override` (S3+S4): entity renamed, `local_id` stable → patch remains `Active`, `asset_version_seen == 2`.
- [ ] 4.6 Test 5 `resync_moves_to_orphaned_on_entity_removed` (S5): asset entity removed → patch moves to `orphaned_overrides`, `report.orphaned == 1`.
- [ ] 4.7 Test 6 `resync_marks_stale_on_field_rename` (S6): `field_path` segment missing in values → `Stale`, `report.stale == 1`.
- [ ] 4.8 Test 7 `resync_marks_conflict_on_type_change` (S7): `serde_json::Value` kind mismatch (number vs string) → `Conflict`, `report.conflict == 1`, patch stays in `overrides`.
- [ ] 4.9 Test 8 `resync_rebinds_via_local_path` (S8): orphaned patch with target reappearing in asset → moved back to `overrides`, `report.rebound == 1`.
- [ ] 4.10 Test 9 `effective_values_with_no_overrides` (S9): empty overrides → asset mirrored, `unresolved` empty, counter advances by N entities.
- [ ] 4.11 Test 10 `resync_extends_id_map_on_new_entity` (S10): new entity in asset → `id_map` gains one entry, existing IDs preserved.
- [ ] 4.12 Verify: `cargo test --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` → all 10 tests pass.
- [ ] 4.13 Commit: `test(editor-core): add scene instance overrides tests`.

## Phase 5: Verification (T5, no commit)

- [ ] 5.1 `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` → green.
- [ ] 5.2 `cargo test --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml --no-run` → all targets build.
- [ ] 5.3 `rustfmt --check crates/editor-core/src/scene_instance_overrides.rs crates/editor-core/tests/scene_instance_overrides.rs crates/editor-core/src/document.rs` → check diff.
- [ ] 5.4 If `rustfmt --check` reports diff on the two new files ONLY, apply: `rustfmt crates/editor-core/src/scene_instance_overrides.rs crates/editor-core/tests/scene_instance_overrides.rs`. Do NOT run `rustfmt` on `document.rs` (one-line derive change with surrounding context — leave untouched).
- [ ] 5.5 Final `git log --oneline -5` → confirm 4–5 atomic commits present, no unrelated changes.

---

## ⚠️ Critical Guards (apply MUST honor)

- DO NOT run `cargo fmt` on the workspace. Use `rustfmt` direct on the two new files only.
- DO NOT modify any file outside: `crates/editor-core/src/document.rs` (1-line derive), `crates/editor-core/src/lib.rs` (1-line `pub mod`), plus the two new files.
- DO NOT delete any file. DO NOT touch Fase 0 (`scene_asset.rs`, `scene_instance.rs`, `bsn_ir.rs`), Fase 1 (`bsn_codegen.rs`), or Fase 2 (`scene_asset_catalog.rs`) modules.
- If `cargo check` fails on unrelated files, STOP and report.
- If final LOC exceeds 900 (50% over 600 upper bound), STOP and report.
- Apply MUST NOT push, merge, or open a PR.

---

## Out-of-Scope Reminder

- No commands, no operation-log/undo integration.
- No frontend / inspector / UI surfacing.
- No `bsn!` codegen from a `SceneInstance`.
- No Scene Asset Variants / inheritance.
- No OPFS persistence of `id_map` or overrides.

---

## Standard Envelope

- **status**: success
- **executive_summary**: 5 tasks (T1–T5) producing 2 new files (`scene_instance_overrides.rs` + tests) and 2 modified files (`document.rs` 1-line derive, `lib.rs` 1-line `pub mod`). 7 public functions + 5 types per design; 10 tests mapped to spec scenarios. Bounded to 400–600 LOC; apply must abort if actual exceeds 900.
- **task_count**: 5
- **context_quality**: C2
- **forecast_files**: 2 (new) + 2 (modified) = 4
- **forecast_loc**: 400–600
- **forecast**: budget_risk=Medium, chained_prs=No, delivery=single-pr, decision_needed=No, chain_strategy=pending
- **next_recommended**: apply
- **engram_save_topic_key**: `sddk/scene-instance-overrides/tasks`
- **capture_prompt**: false
- **risks**: LOC overrun (apply must stop >900); accidental formatting of `document.rs`; accidental edits to Fase 0/1/2 modules.