# BJ-3 — UI entity creation test (explore-report)

**Cycle**: `p-28fce7028ac3c497/bj3-ui-entity-creation`
**Sequence**: 265 (2026-09-08)
**Path**: A-min
**Phase**: explore

## 1. Origin

BJ-3 (UI-creation Playwright test) is the last open sub-deliverable
of G1. The G1 step (v1.0-stabilization P1) requires proving that
**a complete 2D game can be created in the editor without
hand-editing editor data** — i.e. via the editor's UI surface
(buttons, panels, inputs).

`frontend/tests/e2e-game-creation.spec.ts` (existing, 272 lines)
already covers the **loading** path: mount the committed sample
into OPFS, hydrate, and verify the editor renders it. That's
load-time, not create-time.

BJ-3 specifically asks: "can a user **create** a new entity from
scratch via the UI?"

## 2. Hypothesis

A Playwright spec that:
1. Loads a clean editor (no OPFS-mounted sample).
2. Clicks the `+ Add Entity` button (`data-testid="add-entity-btn"`).
3. Verifies that `window.get_scene_snapshot()` now reports exactly
   one entity.
4. Verifies that the entity appears in the Hierarchy panel
   (`[data-testid="hierarchy-entity-${id}"]`).
5. (optional) Renames the entity via inline rename.
6. (optional) Sets a `Name` field via the Inspector.

This proves the editor's create-entity UI flow works end-to-end
without any backend hand-edit.

## 3. Investigation

### Existing patterns

`frontend/tests/selected-entity.spec.ts` already uses the
`add-entity-btn` pattern (lines 18-31) — clicks the button, waits
for `get_scene_snapshot()?.entities?.length === 1`, then queries
`hierarchy-entity-${id}`.

The helper `frontend/tests/helpers/waitForEditorReady.ts` provides
`waitForEditorReady(page)` that waits for `window.__bevyEngineStarted
=== true` (the single readiness contract).

### Test placement

The new spec should live in `frontend/tests/` next to
`e2e-game-creation.spec.ts` (the canonical sample e2e). Naming:
`ui-entity-creation.spec.ts`.

### Scope

Two tests:
1. `clicking Add Entity creates an entity and renders it in the
   hierarchy` — covers the basic create flow.
2. `creating two entities via UI yields two entities in the snapshot`
   — covers the multi-create path (the button should be re-clickable).

We do NOT need to:
- Set components (BJ-3 says "create", not "configure").
- Save to OPFS (a separate gate, G5).
- Load a scene (covered by `e2e-game-creation.spec.ts`).

### Run posture

The spec is `@full` (not `@smoke`) because it loads the WASM
engine. Following the convention from
`e2e-game-creation.spec.ts`:
```ts
test.describe("BJ-3 UI entity creation", { tag: ["@full"] }, () => {
  ...
});
```

## 4. Decision

- **New file**: `frontend/tests/ui-entity-creation.spec.ts`.
- **Two tests**:
  - `add_entity_button_creates_one_entity`
  - `add_entity_button_can_be_clicked_multiple_times`
- **Reuse**: `waitForEditorReady` helper, `add-entity-btn` testid,
  `get_scene_snapshot()` bridge export, `hierarchy-entity-${id}`
  testid.

## 5. Validation status

Pending implementation. Expected:
- New spec compiles (`tsc --noEmit` clean).
- Both tests pass under `@full` Playwright cohort.

## 6. Open questions

None. The pattern is well-established by `selected-entity.spec.ts`
and the e2e-game-creation suite.
