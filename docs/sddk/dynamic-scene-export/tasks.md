# DynamicScene Export — Tasks

## Cycle: `dynamic-scene-export`
**Branch:** `feat/dynamic-scene-export`
**Review budget:** 12 atomic commits, ~600 LOC across Rust + TS + tests.

Each task has:
- Scope (files touched)
- Implementation hint
- Acceptance criteria (verifiable)
- Commit boundary (where this task ends — atomic commit)

---

## Task 1: Add ADR-0004 to docs/adr/

**Scope:** `docs/adr/0004-dynamic-scene-export-bevy-native-anchor.md` (already written in Fase 4).

**Commit:** `docs(adr): ADR-0004 DynamicScene Export — Bevy native anchor`

**AC:**
- File exists at `docs/adr/0004-dynamic-scene-export-bevy-native-anchor.md`
- Has Status, Context, Decision, Consequences, Alternatives, References sections

---

## Task 2: Create `dynamic_scene.rs` module skeleton

**Scope:** `crates/editor-core/src/dynamic_scene.rs` (new file).

Create the module with empty module-level docs, type declarations (`DynamicSceneExport`,
`EntityExport`, `ExportWarning`, `ExportError`, `EXPORT_VERSION`), and a stub `export_dynamic_scene`
function that returns an empty `DynamicSceneExport`.

Register module in `crates/editor-core/src/lib.rs`: `mod dynamic_scene;` and `pub use dynamic_scene::*;`.

**Implementation hint:** Copy the structure from `crates/editor-core/src/schema.rs`. Re-export
public types.

**Commit:** `feat(dynamic-scene): module skeleton with public types`

**AC:**
- `cargo build` succeeds
- `DynamicSceneExport`, `EntityExport`, `ExportWarning` are public from `editor_core`
- 1 placeholder unit test passes

---

## Task 3: Implement `map_name` and Scenario 2 test

**Scope:** `crates/editor-core/src/dynamic_scene.rs`.

Implement `map_name` and integrate into `map_components`. Add test for Scenario 2.

**Commit:** `feat(dynamic-scene): export editor.Name → bevy.Name`

**AC:**
- Test `test_export_name_component` passes
- `cargo test` passes

---

## Task 4: Implement `map_transform` and Scenarios 3-5, 15-16 tests

**Scope:** `crates/editor-core/src/dynamic_scene.rs`.

Implement `map_transform` including quaternion calculation, Vec2 parsing with defaults, scale
mapping. Add tests for Scenarios 3, 4, 5, 15, 16.

**Implementation hint:** Use `(half_rad).sin_cos()` for quaternion.

**Commit:** `feat(dynamic-scene): export editor.Transform2D → bevy.Transform`

**AC:**
- Tests pass for translation (z=0), rotation (quaternion), scale (z=1), invalid Vec2, default values
- `cargo test` passes

---

## Task 5: Implement `map_sprite` and `anchor_str_to_bevy` and Scenarios 6-10 tests

**Scope:** `crates/editor-core/src/dynamic_scene.rs`.

Implement sprite mapping with all 9 anchors, empty-asset warning, missing color default, invalid
anchor default. Add tests for Scenarios 6, 7, 8, 9, 10.

**Commit:** `feat(dynamic-scene): export editor.Sprite2D → bevy.Sprite with anchor mapping`

**AC:**
- All 9 anchors serialize correctly
- Empty asset produces warning + omitted sprite
- Missing color defaults to white + warning
- Invalid anchor defaults to Center + warning

---

## Task 6: Implement editorial skip + unknown component warning + Scenarios 11-12 tests

**Scope:** `crates/editor-core/src/dynamic_scene.rs`.

Add the silent skip for `editor.Visible` / `editor.Locked` and the unknown-component warning in
`map_components`. Tests for Scenarios 11, 12.

**Commit:** `feat(dynamic-scene): skip editorial and unknown components`

**AC:**
- Visible/Locked not in output, no warning
- Unknown type_id is skipped with warning

---

## Task 7: Implement parent resolution + Scenarios 13-14 tests

**Scope:** `crates/editor-core/src/dynamic_scene.rs`.

Implement `resolve_parent` in `export_dynamic_scene`. Tests for Scenarios 13 (parent-child) and
14 (orphan → root + warning).

**Commit:** `feat(dynamic-scene): resolve parent_stable_id with orphan handling`

**AC:**
- Parent-child hierarchy preserved
- Orphan promotes to root with warning

---

## Task 8: Determinism test + Scenarios 17, 20, 21 tests

**Scope:** `crates/editor-core/src/dynamic_scene.rs` tests only.

Add tests for:
- Scenario 17: same input → identical bytes
- Scenario 20: 50 entities all present
- Scenario 21: component order independent of input order

**Commit:** `test(dynamic-scene): determinism and multi-entity coverage`

**AC:**
- All 3 tests pass

---

## Task 9: Add `export_dynamic_scene_wasm` WASM binding + Scenarios 18, 19, 22 tests

**Scope:** `crates/editor-core/src/lib.rs`.

Add the WASM binding. Handle parse errors and serialize errors as JsValue.

**Implementation hint:** `serde_wasm_bindgen::to_value(&export)` for the success path. Fall back
to JSON string + `JSON.parse` on the JS side if it fails (verify with a smoke test).

**Commit:** `feat(dynamic-scene): WASM binding export_dynamic_scene_wasm`

**AC:**
- `cargo build --target wasm32-unknown-unknown` succeeds
- WASM exposes `window.export_dynamic_scene_wasm` function
- Returns `{ json: String, warnings: ExportWarning[] }` on success
- Throws JsValue error on parse failure

---

## Task 10: Frontend `engine-bridge.ts` helper

**Scope:** `frontend/src/engine-bridge.ts`.

Add `exportDynamicScene(sceneJson: string): Promise<DynamicSceneExportResult>` typed helper. Add
TypeScript interfaces for `DynamicSceneExportResult` and `ExportWarning`.

**Implementation hint:** Same pattern as `getSceneSnapshot()` — wait for `window.X` if needed.

**Commit:** `feat(engine-bridge): exportDynamicScene helper`

**AC:**
- `tsc --noEmit` clean
- Helper exported from `engine-bridge.ts`

---

## Task 11: Playwright test `frontend/tests/export.spec.ts`

**Scope:** `frontend/tests/export.spec.ts` (new file).

3 tests:
- `export_dynamic_scene on empty document` — calls helper with empty SceneDocument, asserts
  `entities.length === 0`, `warnings.length === 0`
- `export_dynamic_scene with all 3 components` — calls helper with Name+Transform2D+Sprite2D,
  asserts the output has all 3 `bevy.*` keys
- `export_dynamic_scene with empty asset emits warning` — calls helper with Sprite2D empty asset,
  asserts the response includes a warning

**Implementation hint:** Use the same Playwright pattern as `engine.spec.ts` — `await page.waitForFunction(...)` to ensure wasm loaded before calling.

**Commit:** `test(e2e): export dynamic scene WASM binding`

**AC:**
- All 3 tests pass
- Total Playwright tests = 29 (was 26)

---

## Task 12: Verify regression — all existing tests pass

**Scope:** no code changes. Run full test suite.

Run:
- `cargo test --workspace` (expect 112 + new dynamic_scene tests = ~130+ passing)
- `cd frontend && npm run typecheck` (expect clean)
- `cd frontend && npx playwright test` (expect 26 existing + 3 new = 29 passing)

If any regression, fix in a follow-up commit before PR.

**Commit:** `chore(dynamic-scene): regression check` (only if a fix is needed; otherwise no commit)

**AC:**
- All existing tests pass
- New tests pass

---

## Commit Sequence (one PR)

```
docs(adr): ADR-0004 DynamicScene Export — Bevy native anchor
feat(dynamic-scene): module skeleton with public types
feat(dynamic-scene): export editor.Name → bevy.Name
feat(dynamic-scene): export editor.Transform2D → bevy.Transform
feat(dynamic-scene): export editor.Sprite2D → bevy.Sprite with anchor mapping
feat(dynamic-scene): skip editorial and unknown components
feat(dynamic-scene): resolve parent_stable_id with orphan handling
test(dynamic-scene): determinism and multi-entity coverage
feat(dynamic-scene): WASM binding export_dynamic_scene_wasm
feat(engine-bridge): exportDynamicScene helper
test(e2e): export dynamic scene WASM binding
```

PR title: `feat(dynamic-scene-export): DynamicScene JSON adapter with Bevy runtime mapping`

PR body template:
- Summary: New module + WASM binding that exports SceneDocument to a Bevy-compatible JSON
  artifact (Hito 0 §9.5).
- ADR-0004 documents the Bevy native anchor decision.
- 18 unit tests cover all 22 spec scenarios.
- 3 Playwright tests cover the WASM surface.
- All existing tests continue to pass (regression).

Tag: `v0.6.0` after merge.

---

## Forecast vs Budget

- Estimated time: 4–6 hours.
- Estimated LOC: ~350 Rust + ~50 TS + ~80 Playwright + ~30 ADR + ~100 docs.
- Risk buffer: +1 hour for WASM serialization quirks (the `serde_json::Value` nested inside
  `BTreeMap` may need fallback to `JsValue::from_str(&json)` like `get_scene_snapshot`).

Total: ~12 commits, 1 PR, 1 tag.
