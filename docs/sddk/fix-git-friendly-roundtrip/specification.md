# Specification — fix-git-friendly-roundtrip (cycle 397)

**Cycle:** `p-28fce7028ac3c497/fix-git-friendly-roundtrip`
**Path:** A-min
**Phase:** Specify (sequence 398)
**Date:** 2026-09-08
**Base:** `01b0e50` (v0.110.8 archive)

## Goal

Close the P2 carry-forward: 2 pre-existing
`git-friendly-roundtrip.spec.ts` failures at base `01b0e50`. After the
cycle, both tests must pass without regressing any other test.

## Requirements

### REQ-1: Mirror re-hydration bridge

**Statement.** The engine MUST expose a test bridge
`window.__rehydrateProjectStore` that re-hydrates the in-memory
project-store mirror from current OPFS state. Calling the bridge
multiple times MUST be safe and idempotent.

**Acceptance.**
- `window.__rehydrateProjectStore` is a function (no args, returns
  `Promise<void>`).
- After the bridge resolves, `js_exists("project.json")` returns `true`
  iff `project.json` is currently present in raw OPFS.
- Calling the bridge before any `opfs_save_file` is a no-op (mirror
  remains empty, no errors thrown).
- The bridge is registered AFTER `init_project_store()` (parallels the
  deferral pattern of `__setEditorMode`, `__loadSampleProject`, etc.).

**Implementation.**
- `crates/editor-wasm/src/lib.rs`: add a `#[wasm_bindgen] pub async fn
  rehydrate_project_store()` that calls
  `OpfsProjectStore::hydrate()` on the registered project store. Or
  expose the underlying `hydrate()` via the existing `EditorSession`
  if that owns the store (preferred — keeps ownership semantics).
- `frontend/src/engine-bridge.ts`: add `(window as any).__rehydrateProjectStore =
  () => wasm.rehydrate_project_store();` after the `init_project_store()`
  call, alongside the other test bridges.

### REQ-2: Sample schemas declare format version

**Statement.** Every JSON file in
`examples/platformer-minimal/` MUST have a top-level `"version"` field
per ADR-0045. The 2 schema JSONs MUST be updated to add this field.

**Acceptance.**
- `examples/platformer-minimal/schemas/game.PlayerController.schema.json`
  starts with `{ "version": "0.1", ... }` (any position; convention is
  first key after the opening brace).
- `examples/platformer-minimal/schemas/game.EnemyPatrol.schema.json`
  starts with `{ "version": "0.1", ... }`.
- All other 7 sample files are verified to already have `version` (no
  changes needed beyond the 2 schemas — verified at explore-time: only
  these 2 fail the test, so the other 7 are already conformant).

**Value choice.** `"0.1"` matches the format version used by the
existing schema JSON files. Per ADR-0046 v1-format-manifest.md,
schema documents use the engine's schema format version, which is
`0.1` for the current sample.

### REQ-3: Test calls rehydrate before load_project

**Statement.** The test
`project_json_round_trip_is_byte_identical` MUST call
`window.__rehydrateProjectStore` after `mountSampleInOpfs(page)` and
before `window.load_project()`, so that the engine's mirror reflects
the freshly-written OPFS files.

**Acceptance.**
- `frontend/tests/git-friendly-roundtrip.spec.ts:34-42` is updated to
  include the rehydrate call. The new flow is:
  ```typescript
  await mountSampleInOpfs(page);
  await page.evaluate(async () => {
    await (window as any).__rehydrateProjectStore?.();
  });
  await page.evaluate(async () => {
    await (window as any).load_project();
  });
  ```
- The test must pass on a fresh OPFS context.

### REQ-4: Both failing tests pass

**Statement.** Both `git-friendly-roundtrip.spec.ts` tests pass in the
`@full` cohort after the cycle:
1. `project_json_round_trip_is_byte_identical` — proves byte-identical
   round-trip of `project.json`.
2. `every_opfs_file_is_parseable_json_with_version_field` — proves
   every persisted OPFS file is parseable JSON with top-level
   `version`.

**Acceptance.**
- `npx playwright test --config=playwright.config.ts
  tests/git-friendly-roundtrip.spec.ts --reporter=line` exits 0 with
  2/2 tests passing.
- 12/12 in-scope regressions (8 tutorial-walkthrough + 2
  tour-completed-persistence + 2 e2e-game-creation) still pass.
- 2/2 new tests (load-sample-real-loader) still pass.
- `npx tsc --noEmit` clean.
- `npx eslint --max-warnings=0` clean on all touched files.

## Out of scope

- Rehydrate from UI affordance (a future cycle can promote the test
  bridge to a UI "Reload from disk" button).
- Auto-versioning contract for new schemas written via the editor's
  schema-authoring UI. Current cycle fixes the sample; production
  authoring tools may need a separate review.
- The 2 pre-existing `git-friendly-roundtrip.spec.ts` failures are
  the ONLY thing in scope. Any other test failure elsewhere (e.g., the
  P3 `ux-welcome.spec.ts @full` cohort) is out of scope.

## Risk assessment

- **Backward compat**: New bridge is additive. Existing tests don't
  call it, so they're unaffected.
- **Idempotency**: `OpfsProjectStore::hydrate()` is idempotent — safe
  to call multiple times. Concurrent calls may race on the mutex but
  both will succeed (last write wins, content is identical).
- **Ordering**: Bridge is registered AFTER `init_project_store()` to
  avoid races during startup (parallels all other test bridges).
