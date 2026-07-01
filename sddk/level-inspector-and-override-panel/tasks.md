# Tasks: Level Inspector & Override Panel

> Change: `level-inspector-and-override-panel` · Path: A-full · Mode: auto
> Sources: [`spec.md`](./spec.md), [`design.md`](./design.md)
> Project: bevy-2d-editor

## Coherence Concern Notes (affect ordering)

- **C1 (design extra binding)**: `override_field_status_wasm` is referenced in design §File Changes but missing from spec §6 acceptance criteria. Resolved by making `4.1` an explicit Phase 4 task — it ships with the WASM bridge and is testable via S6/S7.
- **C2 (TS naming overlap)**: `ComponentEditor` (per-field widget in `ComponentEditor.tsx`) and `ComponentCard` (per-component container in `ComponentCard.tsx`) are distinct. The override indicator lives on `ComponentCard` per design — `ComponentEditor` stays type-aware only. `6.1` adds the new prop on `ComponentCard`, not `ComponentEditor`.

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 700–900 (Rust ~350, TS ~250, tests ~200, doc ~50) |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 → PR 2 → PR 3 (feature-branch-chain off `feature/inspector-override`) |
| Delivery strategy | auto-chain |
| Chain strategy | feature-branch-chain |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: feature-branch-chain
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Rust write-side foundation | PR 1 → `feature/inspector-override` | Pure helpers + Command variants + processor + Rust tests. Self-contained, validates S1–S4 + S11–S13. |
| 2 | WASM bridge | PR 2 → `feature/inspector-override` | New bindings (`override_field_status_wasm`, `upsert_override_wasm`, `revert_override_wasm`). Validates S5. |
| 3 | Frontend integration | PR 3 → `feature/inspector-override` | TS wrappers + `ComponentCard` prop + `InspectorPanel` instance branch + Playwright tests. Validates S6–S10. |

## Phase 1: Rust Pure Helpers (foundation for §3 + §4)

- [ ] 1.1 `feat(scene_instance_overrides)`: add `upsert_override(&mut SceneInstance, patch)` in `crates/editor-core/src/scene_instance_overrides.rs` — key = `(local_id, type_id, field_path)`, force `status=Active`, no mutation of `id_map`/`instance_components`. Tests S11 in `#[cfg(test)] mod`.
- [ ] 1.2 `feat(scene_instance_overrides)`: add `remove_override(&mut SceneInstance, local_id, type_id, field_path) -> Option<ComponentOverride>` — idempotent, returns captured patch. Tests S12 + S13.
- [ ] 1.3 `feat(scene_instance_overrides)`: add `FieldOverrideEntry { local_id, component_type_id, field_path, status }` pub struct + `field_override_index(&SceneInstance) -> Vec<FieldOverrideEntry>` — covers both `component_overrides` and `orphaned_component_overrides`. Unit test: ordering + status pass-through.

## Phase 2: Command Variants (depends on 1.1 for inverse semantics)

- [ ] 2.1 `feat(command)`: extend `Command` enum in `crates/editor-core/src/command.rs` with `UpsertOverride { instance_id, target_local_id, component_type_id, field_path: Vec<String>, value }` and `RevertOverride { instance_id, target_local_id, component_type_id, field_path: Vec<String> }`. Serde `PascalCase` tag already in place.
- [ ] 2.2 `test(command)`: add serde round-trip tests for both new variants in `command.rs` `#[cfg(test)]` mod — mirrors `test_*_serializes` pattern.

## Phase 3: Processor Apply/Inverse (depends on Phase 2 + 1.1)

- [ ] 3.1 `feat(processor)`: add `validate` + `apply` arms for `UpsertOverride` in `crates/editor-core/src/processor.rs` — validate instance exists, call `upsert_override`, return inverse (`RevertOverride` if no prior, else `UpsertOverride{old}`). Test S1.
- [ ] 3.2 `feat(processor)`: add `validate` + `apply` arms for `RevertOverride` — idempotent no-op when absent. Inverse = re-insert via `upsert_override`. Test S3.
- [ ] 3.3 `test(processor)`: add forward/inverse round-trip tests `test_forward_inverse_roundtrip_upsert_override` + `test_revert_override_noop` — mirrors `PlaceInstance` roundtrip test pattern.

## Phase 4: WASM Bridge (depends on Phase 3 + 1.3) — addresses C1

- [ ] 4.1 `feat(lib)`: add `override_field_status_wasm(instance_json: &str) -> Result<String, JsValue>` in `crates/editor-core/src/lib.rs` — wraps `field_override_index`, returns JSON array. **C1: explicit task; closes the spec gap.**
- [ ] 4.2 `feat(lib)`: add `upsert_override_wasm(instance_id, local_id, type_id, field_path_json, value_json) -> Result<String, JsValue>` — builds `Command::UpsertOverride` envelope + calls `dispatch_command`.
- [ ] 4.3 `feat(lib)`: add `revert_override_wasm(instance_id, local_id, type_id, field_path_json) -> Result<String, JsValue>` — same shape for `RevertOverride`.

## Phase 5: Frontend Types & Wrappers (depends on Phase 4)

- [ ] 5.1 `feat(scene-assets)`: add `FieldOverrideEntry` TS interface + `overrideFieldStatus(instance)`, `upsertOverride(...)`, `revertOverride(...)` wrappers in `frontend/src/services/scene-assets.ts` — mirror existing `effectiveValues` wrapper shape (JSON.stringify in/out).

## Phase 6: Frontend UI (depends on Phase 5) — addresses C2

- [ ] 6.1 `feat(ComponentCard)`: extend `Props` in `frontend/src/components/ComponentCard.tsx` with optional `fieldOverrideStatus: Record<string, OverrideStatus>` + `onRevertField: (fieldPath: string) => void`. Render per-field indicator dot (blue/warning/error/dimmed) + revert button when status present. **C2: prop lives on `ComponentCard`, not `ComponentEditor`.**
- [ ] 6.2 `feat(InspectorPanel)`: branch in `frontend/src/components/InspectorPanel.tsx` when `selectedId.startsWith("inst_")` — load effective values via `effectiveValues` and render `ResolvedEntity.components` instead of raw `entity.components`. Test S6.
- [ ] 6.3 `feat(InspectorPanel)`: add per-field override indicator lookup from `overrideFieldStatus(selectedInstance)` — pass map into `ComponentCard` per component. Test S7.
- [ ] 6.4 `feat(InspectorPanel)`: replace ad-hoc override counts block with normalized "Overrides" section header showing `active | stale | orphaned | conflict` badges (reuse existing `overrideCounts` derivation). Test S8.
- [ ] 6.5 `feat(InspectorPanel)`: wire per-field revert button → `revertOverride(iid, local_id, type_id, field_path)` then re-poll `effectiveValues` + `overrideFieldStatus`. Test S9.
- [ ] 6.6 `feat(InspectorPanel)`: add resync warning banner — render when `getResyncReports()` for selected instance has `stale>0 || conflict>0`; banner shows count + "Open Workbench" button (placeholder href OK for now). Test S10.

## Phase 7: Integration & E2E Tests

- [ ] 7.1 `test(integration)`: add wasm-bindgen test for `upsert_override_wasm` → `revert_override_wasm` round-trip in `crates/editor-core/tests/` (or existing wasm test harness) — verify `effective_values_wasm` restores asset value. Test S5.
- [ ] 7.2 `test(e2e)`: extend existing Playwright suite in `frontend/e2e/` with `inspector-override.spec.ts` — place instance, select child, verify effective value + indicator, click revert, verify cleared.
- [ ] 7.3 `test(e2e)`: same spec — `Ctrl+Z` after upsert restores prior value.

## Phase 8: Cleanup & Documentation

- [ ] 8.1 `docs(CONTEXT)`: add `Override Count Badge`, `Per-field Override Indicator`, `Resync Warning Banner` terms to `CONTEXT.md` Glossary (or relevant section). Update `Scene Instance` term to mention write-side command surface.
- [ ] 8.2 `chore(verify)`: run full `cargo test -p editor-core` + `cargo build --target wasm32-unknown-unknown` + `pnpm playwright test` — must pass with no regression to existing 112+ Rust / 27+ Playwright tests.

## Suggested Commit Boundaries

1. `feat(scene_instance_overrides): add upsert/remove helpers and field_override_index` (Phase 1)
2. `feat(command): add UpsertOverride and RevertOverride variants` (Phase 2)
3. `feat(processor): apply/inverse for override mutation commands` (Phase 3)
4. `feat(lib): add override_field_status_wasm, upsert_override_wasm, revert_override_wasm` (Phase 4)
5. `feat(scene-assets): FieldOverrideEntry type and override wrappers` (Phase 5)
6. `feat(InspectorPanel): effective values render with override indicators` (Phase 6.1–6.4)
7. `feat(InspectorPanel): per-field revert affordance and resync warning banner` (Phase 6.5–6.6)
8. `test(inspector-override): wasm round-trip + Playwright e2e` (Phase 7)
9. `docs: CONTEXT.md glossary update` (Phase 8)

## Forecast per Task

| Task | Effort | Risk |
|------|--------|------|
| 1.1 | S | low — pure fn, isolated |
| 1.2 | S | low |
| 1.3 | S | low |
| 2.1 | S | low — additive enum variants |
| 2.2 | XS | low — boilerplate tests |
| 3.1 | M | medium — inverse logic correctness |
| 3.2 | M | medium — idempotent semantics |
| 3.3 | S | low |
| 4.1 | S | low — thin wrapper |
| 4.2 | S | low |
| 4.3 | XS | low |
| 5.1 | S | low — type + 3 wrappers |
| 6.1 | S | low — prop-only change |
| 6.2 | M | medium — branch + data shape swap |
| 6.3 | S | low |
| 6.4 | XS | low |
| 6.5 | M | medium — async + re-poll |
| 6.6 | M | medium — banner + workbench hook |
| 7.1 | M | medium — wasm test harness |
| 7.2 | L | high — full e2e coverage |
| 7.3 | S | low — Ctrl+Z wiring test |
| 8.1 | XS | low |
| 8.2 | S | low — verification run |