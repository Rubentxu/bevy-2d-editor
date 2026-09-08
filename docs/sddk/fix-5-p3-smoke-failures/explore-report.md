# Explore Report — `fix-5-p3-smoke-failures`

> **Cycle:** `p-28fce7028ac3c497/fix-5-p3-smoke-failures`
> **Path:** A-lite (2-phase: spec → tasks → apply → verify → release → archive)
> **Sequence:** 411 → 412 (`phase.explore.complete`)
> **Phase:** Explore
> **Subject:** Diagnose 5 pre-existing `@smoke` cohort failures (P3 carry-forward from v0.110.6)

---

## 1. Scope

The cycle covers 5 pre-existing Playwright failures that have been
carried forward as P3 since v0.110.6 (the smoke cohort was excluded
from the v1.0 product-gate matrix at v0.110.3 — G6 closure). They are
NOT v1.0 blockers but they ARE the last visible red in the local
smoke run.

| # | Test | File | Line | Symptom |
|---|------|------|------|---------|
| 1 | P2 multi-select | `app-characterization.spec.ts` | 56 | `expect(received).toBe(true)` failed |
| 2 | P4 scene operations | `app-characterization.spec.ts` | 126 | `expect(received).toBe(true)` failed |
| 3 | P5 composition root | `app-characterization.spec.ts` | 149 | `expect(received).toBeTruthy()` failed |
| 4 | P8 welcome overlay dismissal | `app-characterization.spec.ts` | 211 | `ReferenceError: browser is not defined` |
| 5 | S2 pre-ready action feedback | `editor-ready.spec.ts` | 78 | Test timeout 120 s exceeded |

The cycle will close **#1–#4** in a single A-lite cycle (4 fixes in
1 file, all `app-characterization.spec.ts`) and **#5** in a separate
cycle (cycle 400) — the S2 case is structurally different (a
"phantom testid" timing issue against an `__bevyEngineStarted`
contract that may require a fix in `engine-bridge.ts` or in the test
itself, deserving its own explore/decision).

This explore-report documents cycle 399's scope: **#1–#4 only**.

## 2. Reproduction

```bash
cd frontend && timeout 240 npx playwright test --project=smoke \
    tests/app-characterization.spec.ts \
    -g 'P2:|P4:|P5:|P8:' \
    --reporter=line
```

Observed: 4 failed (the 4 targets above). 0 passed. Wall time ~30 s.

```text
Running 1 test using 1 worker
[1/1] [smoke] › tests/app-characterization.spec.ts:56:3 › ... P2: multi-select @smoke @app
  1) ... P2: multi-select ... Error: expect(received).toBe(true)
  2) ... P4: scene operations ... Error: expect(received).toBe(true)
  3) ... P5: composition root ... Error: expect(received).toBeTruthy()
  4) ... P8: welcome overlay ... ReferenceError: browser is not defined
```

## 3. Per-test diagnosis

### #1 — P2 multi-select (`app-characterization.spec.ts:56`)

The test expects `window.__selectEntity` to be a function:

```ts
const hasSelectionController = await page.evaluate(
  () => typeof (window as any).__selectEntity === "function"
);
expect(hasSelectionController).toBe(true);
```

**Production reality**: the engine-bridge exposes the SCENE-level
selection via `dispatch_command({type:"SelectEntity", id:"..."})` (a
typed command), NOT a direct `__selectEntity` setter on `window`. There
is no test bridge for selection at all.

Decision: **this is a test bug** — the test was written against a
designed-but-never-implemented `__selectEntity` test bridge. The fix is
to switch the test to use the existing `dispatch_command` typed command
path (which is what production code uses anyway). This preserves the
test's INTENT (assert selection control exists) without introducing a
dead bridge into the production surface.

### #2 — P4 scene operations (`app-characterization.spec.ts:126`)

The test expects `window.sceneCreate`, `window.sceneSwitch`,
`window.sceneDelete` (camelCase):

```ts
const hasSceneOps = await page.evaluate(
  () =>
    typeof (window as any).sceneCreate === "function" &&
    typeof (window as any).sceneSwitch === "function" &&
    typeof (window as any).sceneDelete === "function"
);
expect(hasSceneOps).toBe(true);
```

**Production reality**: `engine-bridge.ts:299-303` exposes them in
**snake_case** (`scene_create`, `scene_switch`, `scene_delete`) per
the Rust `wasm_bindgen` convention. The existing `tests/multi-scene.spec.ts:16-18`
uses snake_case and passes:

```ts
typeof (window as any).scene_create === "function" &&
typeof (window as any).scene_switch === "function" &&
typeof (window as any).scene_delete === "function"
```

Decision: **this is a test bug** — the camelCase expectation was a
copy-paste mistake from a JS-style API design. The fix is to switch
to snake_case. This is the same pattern the rest of the test suite
uses (see `multi-scene.spec.ts:51,96,141,228`).

### #3 — P5 composition root (`app-characterization.spec.ts:149`)

The test expects either `window.__getEditorMode` or
`window.__getSelectedIds` to be a function:

```ts
const controllerAvailable = await page.evaluate(
  () =>
    typeof (window as any).__getEditorMode === "function" ||
    typeof (window as any).__getSelectedIds === "function"
);
expect(controllerAvailable).toBeTruthy();
```

**Production reality**: `engine-bridge.ts` exposes
`window.__setEditorMode` (a setter), but does NOT expose a getter.
There is no `__getEditorMode` and no `__getSelectedIds`.

Decision: **this is a partial test bug + a tiny production gap**.
The test's INTENT is to verify the composition root delegates
controller state through a bridge. We have the setter
`__setEditorMode` but no matching getter — a one-sided API. The
cleanest fix is to add a small `__getEditorMode` getter bridge in
`engine-bridge.ts` (1 line: `(window as any).__getEditorMode = () => editorMode;`),
which makes the test pass and also gives the test suite (and any
downstream async introspection) a way to read the mode without
parsing the DOM.

Adding `__getSelectedIds` is NOT necessary — the test accepts EITHER
bridge, and `__getEditorMode` is enough.

### #4 — P8 welcome overlay (`app-characterization.spec.ts:211`)

```ts
test("P8: welcome overlay and onboarding dismissal", async ({ page }) => {
  const context = await browser.newContext();        // ← ReferenceError: browser is not defined
  const welcomePage = await context.newPage();
  ...
```

**Production reality**: `browser` is not in the test's lexical scope.
The fixture system in Playwright provides `page` (which has
`page.context()`); `browser` is only available when explicitly
imported via `_check_scene_field.spec.ts` or destructured from the
fixture:

```ts
import { test, expect, browser } from "@playwright/test";
```

This is a fixture-API misuse. The cleanest fix is to drop the
`browser.newContext()` (we already have a fresh `page` from the
fixture), and instead make the test exercise the actual welcome
overlay through the existing `clearWelcomeDismissed` pattern that
cycle 397/398 already established. This way we don't need a fresh
context — the fixture `page` is already isolated per test by
Playwright's defaults.

Decision: **this is a test bug** — replace `browser.newContext()`
with the standard `page` fixture + OPFS clear + reload pattern that
`ux-welcome.spec.ts` uses (proven by cycle 397/398). The test then
verifies the welcome overlay appears (it should — OPFS is fresh per
test) and dismissal works.

### #5 (out of cycle 399 scope) — S2 pre-ready feedback (`editor-ready.spec.ts:78`)

Tracked for cycle 400. Brief diagnosis (carried from
`v0.110.6/v0.110.7/v0.110.8/v0.110.9` merge-receipts):

- The test clicks `[data-testid="tab-scenes"]` BEFORE engine is ready,
  then waits for `__bevyEngineStarted === true`.
- That `data-testid` does NOT exist in the production source (verified
  via `grep -rnE 'data-testid="tab-scenes"' src/` → no match).
- The `.catch(() => {})` swallows the click failure, but Playwright's
  default click timeout (30 s) is added to the WASM init time
  (~30 s), blowing past the 120 s test timeout on slow CI.
- The cleanest fix: replace the phantom `tab-scenes` click with a
  real bridge interaction (e.g., `(window as any).dispatch_command`
  on a `__bevyEngineStarted`-gated queue) or simply drop the pre-ready
  click — the test's intent is "engine starts even after a noop
  attempt", which can be exercised with an idempotent noop call
  (e.g., `page.evaluate(() => 1+1)` or the existing `dispatch_command`
  typed command for a benign action).

Cycle 400 will scope this properly.

## 4. Production code delta (cycle 399)

A single new bridge in `frontend/src/engine-bridge.ts`:

```ts
// Test bridge: read the current editor mode (companion to
// window.__setEditorMode which is the setter). Used by app-characterization
// P5 to verify the composition root delegates controller state through the
// bridge. Reads the same `editorMode` state that useEditorWorkspaceController
// owns.
(window as any).__getEditorMode = (): string => editorMode;
```

(Where `editorMode` is the local variable in the
`useEditorWorkspaceController` hook, OR a closure over the React state
in a top-level module. Need to check the exact binding — if it's
state-local to the hook, the bridge has to be exposed via the
controller module instead.)

Plus 4 surgical edits in `frontend/tests/app-characterization.spec.ts`:

- P2: `__selectEntity` → `dispatch_command({type:"SelectEntity", id:""})`
  (typed command path)
- P4: `sceneCreate/Switch/Delete` → `scene_create/switch/delete`
  (snake_case match)
- P5: no test change — relies on the new `__getEditorMode` bridge
- P8: `browser.newContext()` → `page` fixture + OPFS clear + reload

Net delta: **2 files / ~+5/-5 LOC + 1 new bridge**.

## 5. Regression invariants

After the cycle, the following must still pass:

- 4/4 `app-characterization.spec.ts` previously-passing tests
  (P1, P3, P6, P7)
- 5/5 `editor-ready.spec.ts` other tests (S1 ×2, S3, S4, "module
  contract")
- 871/871 `cargo test --workspace --lib`
- `npx tsc --noEmit` clean
- `npx eslint --max-warnings=0` clean on touched files
- Re-test 8/8 in-scope from cycle 398 (`ux-welcome.spec.ts` + tutorial-
  walkthrough + tour-completed-persistence) under `--project=full` to
  confirm the engine-bridge additions don't regress the higher cohorts

## 6. Out of cycle 399 scope

- **#5 S2 pre-ready feedback** — separate cycle 400, separate explore.
- The 5 P3 smoke failures split into "spec bugs" (4) vs "spec/product
  ambiguity" (1). The cycle 400 explore will decide whether S2 is a
  spec bug (drop the phantom testid) or a product gap (add a queued
  pre-ready dispatch path).
- Bevy↔JS mutex architectural cleanup (still P2)
- Production-build sample fallback (still P3)

## 7. SDDK routing

Decision Model v2: **A-lite path**.

- **C2** (well-understood context, all 4 fixes already diagnosed)
- **Multi-file but single-concept**: 4 fixes in 1 file + 1 small
  bridge addition (the bridge counts as a tiny new module facet, not
  a fork).
- **No architectural change**: the bridge addition is a 1-line setter/
  getter pair.

A-min would skip the design phase, but with 4 distinct fixes + 1
production change, an A-lite design pass helps anchor the
"minimal-bridge" principle for #3 and the "use-existing-fixture"
principle for #4. A-full is overkill.

Sequence: 411 → 412 (explore) → 413 (specify) → 414 (build, A-lite
collapses design+tasks into build) → 415 (verify) → 416 (release) →
417 (archive-manifest) → 418 (archive.complete).
