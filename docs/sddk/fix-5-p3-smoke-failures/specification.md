# Specification — `fix-5-p3-smoke-failures`

> **Cycle:** `p-28fce7028ac3c497/fix-5-p3-smoke-failures`
> **Path:** A-lite
> **Sequence:** 412 → 413 (`phase.specify.complete.a-lite`)
> **Phase:** Specify

---

## 1. Goal

Close 4 of 5 pre-existing `@smoke` cohort Playwright failures
(`app-characterization.spec.ts` tests P2, P4, P5, P8) carried as P3
since v0.110.6. The 5th failure (S2 in `editor-ready.spec.ts`) is
explicitly out of scope and tracked for cycle 400.

## 2. Requirements (SHALL)

### REQ-1 — P2 multi-select test bridge uses the typed command path

`frontend/tests/app-characterization.spec.ts:84-87` (P2's
`hasSelectionController` assertion) MUST assert the existence of a
selection control that production code actually uses, NOT a phantom
test bridge.

#### Scenario SPEC-S1

- **GIVEN** `window.__bevyEngineStarted === true` (engine ready)
- **WHEN** the P2 test asserts that some selection mechanism is reachable
- **THEN** the asserted mechanism MUST be `window.dispatch_command`
  (the typed command dispatcher that production code uses) AND the
  test MUST dispatch a benign `SelectEntity` command against it
  without throwing

#### Acceptance

- `npx playwright test --project=smoke tests/app-characterization.spec.ts -g 'P2:'`
  reports **1 passed**.
- The change MUST NOT add `__selectEntity` to `engine-bridge.ts` (we
  do not introduce dead bridges).

### REQ-2 — P4 scene operations test uses snake_case

`frontend/tests/app-characterization.spec.ts:133-140` (P4's
`hasSceneOps` assertion) MUST use `scene_create`, `scene_switch`,
`scene_delete` (snake_case matching `wasm_bindgen` convention and
`engine-bridge.ts:299-303`).

#### Scenario SPEC-S2

- **GIVEN** `window.__bevyEngineStarted === true`
- **WHEN** the P4 test asserts that scene-create, scene-switch, and
  scene-delete are reachable
- **THEN** all three MUST be checked via their snake_case names
  (`scene_create`, `scene_switch`, `scene_delete`) AND the assertion
  MUST pass

#### Acceptance

- `npx playwright test --project=smoke tests/app-characterization.spec.ts -g 'P4:'`
  reports **1 passed**.
- This MUST match the pattern already used in
  `tests/multi-scene.spec.ts:16-18`.

### REQ-3 — `__getEditorMode` test bridge exposed

`frontend/src/engine-bridge.ts` MUST expose `(window as any).__getEditorMode`
as a function returning the current `editorMode` string. This
complements the existing `__setEditorMode` setter (exposed by
`useEditorWorkspaceController.ts:177-178`).

#### Scenario SPEC-S3

- **GIVEN** `__setEditorMode("asset-authoring")` has just been called
  from the page
- **WHEN** the page evaluates `__getEditorMode()`
- **THEN** the return value MUST be `"asset-authoring"`

#### Acceptance

- `npx playwright test --project=smoke tests/app-characterization.spec.ts -g 'P5:'`
  reports **1 passed**.
- The new bridge is a **getter** (no-arg, returns string). It MUST
  NOT mutate React state.
- The bridge MUST be exposed AFTER `init_project_store()` per the
  test-bridge ordering convention (parallels `__setEditorMode`).

### REQ-4 — P8 fixture uses `page` instead of `browser`

`frontend/tests/app-characterization.spec.ts:211-242` (P8) MUST NOT
reference `browser` (out of scope). The test MUST use the standard
`page` fixture + the proven OPFS-clear + reload pattern from
`ux-welcome.spec.ts:46-61` (cycle 397/398).

#### Scenario SPEC-S4

- **GIVEN** a fresh Playwright `page` (no `browser.newContext()`)
- **AND** OPFS state from any prior test is cleared via the existing
  `clearWelcomeDismissed` helper or equivalent
- **WHEN** the P8 test reloads to `/` and waits for ready
- **THEN** the welcome overlay `<div data-testid="welcome-overlay">`
  MUST be visible (first-visit path)
- **AND** dismissing it (Skip or Take the tour) MUST remove the
  overlay from the DOM

#### Acceptance

- `npx playwright test --project=smoke tests/app-characterization.spec.ts -g 'P8:'`
  reports **1 passed**.
- The test MUST NOT reference the `browser` global.
- The test MUST end by closing only what Playwright provided (no
  manual `context.close()` since we're using `page` directly).

### REQ-5 — All 4 fixed tests + regression set green

The full smoke run MUST report all 5 originally-failing tests fixed
(4 in this cycle, 1 deferred to cycle 400) AND all previously-passing
tests still pass.

#### Acceptance

- `npx playwright test --project=smoke tests/app-characterization.spec.ts`
  reports **8 passed** (was: 4 passed + 4 failed).
- `npx playwright test --project=smoke tests/editor-ready.spec.ts`
  reports **5 passed + 1 failed** (S2 deferred to cycle 400; S1 ×2,
  S3, S4, module-contract all still pass).
- `cargo test --workspace --lib` reports **871/871 passed**
  (unchanged — no Rust changes).
- `npx tsc --noEmit` reports 0 errors.
- `npx eslint --max-warnings=0` reports 0 errors, 0 warnings on
  `engine-bridge.ts` and `app-characterization.spec.ts`.
- `npx playwright test --project=full tests/ux-welcome.spec.ts tests/tutorial-walkthrough.spec.ts tests/tour-completed-persistence.spec.ts`
  reports **8/8 passed** (regression intact — the new
  `__getEditorMode` bridge must not perturb the higher cohort).

### REQ-6 — Comment hygiene

Each test fix MUST carry an inline comment explaining why the API
name or fixture pattern changed, so the next maintainer does not
re-introduce the bug.

#### Acceptance

- P2 carries a comment explaining that `__selectEntity` was a
  designed-but-never-implemented bridge, and the test now exercises
  the typed command path that production actually uses.
- P4 carries a comment naming `tests/multi-scene.spec.ts` as the
  reference for snake_case scene operations.
- P8 carries a comment naming `ux-welcome.spec.ts:46-61` as the
  reference for the OPFS-clear + reload first-visit pattern.

## 3. Non-goals (will NOT be addressed in this cycle)

- **S2 pre-ready feedback test** (`editor-ready.spec.ts:78`) —
  separate cycle 400. The phantom `data-testid="tab-scenes"` and
  the click-before-ready timing race require their own explore
  decision.
- `__selectEntity` bridge introduction — out of scope; production
  uses the typed command path, so the test should follow suit.
- Bevy↔JS mutex architectural cleanup (P2 carry-forward from
  v0.110.9).
- Production-build sample fallback (P3, tracked from v0.110.8).

## 4. Production code delta summary

A single new test bridge in `engine-bridge.ts`:

```ts
// Test bridge: read the current editor mode (companion to the
// window.__setEditorMode setter exposed by useEditorWorkspaceController).
// Mirrors the existing test-bridge pattern (see __setEditorMode,
// __loadSampleProject, __rehydrateProjectStore) and is exposed AFTER
// init_project_store() per the convention documented in
// docs/specs/editor-test-bridge-ordering.md.
(window as any).__getEditorMode = (): string => /* current editorMode */;
```

The binding source depends on whether `editorMode` lives at module
scope or inside the React hook closure. Cycle 399's apply phase will
inspect `useEditorWorkspaceController.ts` and pick whichever binding
is cleanest (module-scope local that the hook mutates, OR a
subscription pattern).

## 5. File-level delta

| File | LOC | Reason |
|------|-----|--------|
| `frontend/src/engine-bridge.ts` | +5/-0 | New `__getEditorMode` bridge (lines + comment) |
| `frontend/tests/app-characterization.spec.ts` | ~+15/-15 | 4 surgical test fixes (P2, P4, P5 no-change, P8) |
| **Total** | **+20/-15** | |

## 6. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| `__getEditorMode` reads stale state | Reuse the same `editorMode` source-of-truth that `__setEditorMode` mutates; bind to the same module-scope local |
| `dispatch_command` for SelectEntity requires WASM to be ready | Test already waits for `__bevyEngineStarted === true`; safe |
| `clearWelcomeDismissed` is in `ux-welcome.spec.ts`, not exported | Inline the OPFS-clear logic in P8 (3-line snippet) OR extract it to a shared helper file (preferred for readability) |
| S2 fails again on next cycle run (test-pollution from cycle 399 changes) | Cycle 400 is a separate sddk cycle with independent lease and snapshot |

## 7. Acceptance check

```bash
cd /var/home/rubentxu/Proyectos/rust/bevy-2d-editor/frontend
timeout 240 npx playwright test --project=smoke \
    tests/app-characterization.spec.ts \
    -g 'P2:|P4:|P5:|P8:' \
    --reporter=line
```

Expected:

```text
  4 passed (XX.Xs)
```
