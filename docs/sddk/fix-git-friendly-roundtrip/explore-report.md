# Explore Report — fix-git-friendly-roundtrip (cycle 397)

**Cycle:** `p-28fce7028ac3c497/fix-git-friendly-roundtrip`
**Path:** A-min
**Sequence:** 397 (explore)
**Date:** 2026-09-08
**Base:** `01b0e5028e675e917a614a09c0da47c9fb0e1366` (v0.110.8 archive)

## Problem statement

`frontend/tests/git-friendly-roundtrip.spec.ts` has 2 tests failing
in the `@full` cohort at base `01b0e50` (pre-existing since v0.109.7,
isolated via `git stash` at `47117df` per the v0.110.8 archive):

| # | Test | Error | Root cause |
|---|------|-------|------------|
| 1 | `project_json_round_trip_is_byte_identical` | `Error: page.evaluate: project.json not found` at `tests/git-friendly-roundtrip.spec.ts:40:16` | Engine-side mirror is stale after `mountSampleInOpfs` |
| 2 | `every_opfs_file_is_parseable_json_with_version_field` | `schemas/game.PlayerController.schema.json: missing top-level 'version' field` + `schemas/game.EnemyPatrol.schema.json: missing top-level 'version' field` | Sample schemas lack `version` field per ADR-0045 |

## Root-cause analysis

### Failure 1: stale in-memory mirror

**The data flow during the test:**

1. `init_project_store()` runs once at WASM startup. It calls
   `OpfsProjectStore::hydrate()` which lists all paths under
   `bevy-2d-editor/`, reads each file, and populates the in-memory
   mirror (`crates/editor-storage-web/src/opfs_core.rs:243`).
2. After `hydrate()` returns, the mirror is **frozen** — there is no
   re-hydration path. Any subsequent write to raw OPFS via
   `window.opfs_save_file` (the JS-side bridge at
   `frontend/src/engine-bridge.ts:85`) does NOT update the mirror.
3. The test calls `mountSampleInOpfs(page)` which writes the 9 sample
   files to raw OPFS via `window.opfs_save_file`. The mirror is unaware.
4. The test calls `window.load_project()` (line 41 in the spec).
   Rust-side `load_project` (`crates/editor-bevy/src/lib.rs:2062`):
   ```rust
   pub async fn load_project() -> Result<(), JsValue> {
       if !js_exists(PROJECT_FILE).await {
           return Err(JsValue::from_str("project.json not found"));
       }
       ...
   }
   ```
   `js_exists` reads from the mirror (line 1160):
   ```rust
   pub(crate) async fn js_exists(path: &str) -> bool {
       with_project_store()
           .map(|s| s.exists(path).unwrap_or(false))
           .unwrap_or(false)
   }
   ```
   The mirror is empty (Playwright fresh context), so `js_exists` returns
   false → `load_project` throws `"project.json not found"`.
5. The test's `page.evaluate(async () => { await window.load_project(); })`
   propagates the error as a `page.evaluate` exception.

**Architectural asymmetry:** Raw OPFS reads/writes (`window.opfs_*`)
bypass the mirror, but engine reads (`js_exists`, `js_load_file`,
`js_save_file`, `load_project`) go through the mirror. Any test that
hydrates raw OPFS without re-hydrating the mirror will fail at
`load_project`.

### Failure 2: schemas missing `version` field

ADR-0045 declares: "every persisted document has an explicit
schema/format version." The 9 OPFS files in
`examples/platformer-minimal/` are the canonical sample project. The
test asserts every file has a top-level `version` field. Inspection of
the sample:

- `examples/platformer-minimal/project.json` — has `"version": "0.1"` ✅
- `examples/platformer-minimal/scenes/main.scene.json` — has `version` ✅
- `examples/platformer-minimal/schemas/game.PlayerController.schema.json` —
  starts with `"type_id"`, no `version` ❌
- `examples/platformer-minimal/schemas/game.EnemyPatrol.schema.json` —
  starts with `"type_id"`, no `version` ❌

The 4 scene-asset JSONs also need verification (only 2 schemas fail in
the test report, so the other files are OK — but we should check
during implementation).

## Scope decision

The cycle is bounded and contained. Two minimal fixes, both
behaviour-preserving:

1. **Engine bridge for mirror re-hydration** (1 new WASM export + 1
   bridge line + 1 test call):
   - Add `rehydrate_project_store()` to `crates/editor-wasm/src/lib.rs`
     that calls the registered `OpfsProjectStore::hydrate()` again.
   - Expose it as `window.__rehydrateProjectStore` in
     `frontend/src/engine-bridge.ts` (test-only bridge, parallels
     `__setEditorMode` / `__setActiveBottomTab` / `__loadSampleProject`).
   - Call it in the test before `load_project`.

2. **Sample schema version fields** (2 file edits):
   - Add `"version": "0.1"` to the 2 schema JSONs (matches
     `project.json` and the scene/asset JSONs).
   - Verify scene-asset and logic-graph JSONs already have `version`
     (test currently passes for them — must check why they pass but the
     schemas fail).

## Out of scope

- **Rehydrate from UI**: Adding a "Reload from disk" UI button would be
  a product feature; out of cycle scope. The cycle exposes the bridge
  as test-only; a future cycle can promote it to a UI affordance.
- **Schema auto-versioning on register**: The Rust `register_schema`
  flow may or may not add `version` automatically — needs verification
  but is out of scope for this cycle (which fixes the test, not the
  engine's auto-versioning contract).
- **The 2 pre-existing `git-friendly-roundtrip.spec.ts` failures are
  NOT caused by anything in the v0.110.8 cycle** (verified via
  `git stash` at `01b0e50` + `01b0e50~`).

## Risk assessment

- **Rehydrate side-effects**: `OpfsProjectStore::hydrate()` is idempotent
  and overwrites mirror entries from current OPFS. Any data in the
  mirror that is NOT in OPFS is preserved. Schema/scenes registered in
  the engine session (via `EditorSession`) are NOT cleared by rehydrate
  — the mirror is just a read cache. Calling `rehydrate_project_store`
  twice in a row is safe.
- **Concurrency**: `hydrate()` takes the mirror lock briefly. No race
  with `load_project` because tests are sequential.
- **Backward compat**: New bridge is additive. Existing tests don't
  call it, so they're unaffected.

## Confidence

**High.** Both failures are isolated, root-caused, and have minimal,
non-invasive fixes. The cycle should not regress any existing test.
