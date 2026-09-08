# Design — `fix-5-p3-smoke-failures`

> **Cycle:** `p-28fce7028ac3c497/fix-5-p3-smoke-failures`
> **Path:** A-lite
> **Sequence:** 413 → 414 (`phase.design.complete`)
> **Phase:** Design

---

## 1. Goal

This design pass anchors the binding choices for the new
`__getEditorMode` bridge (REQ-3) and clarifies the import-vs-inline
decision for the OPFS-clear helper (REQ-4 / P8). The other 3 fixes
(P2, P4, P5-test) are 1-line test edits already specified in
`specification.md`.

## 2. Decision D1 — `__getEditorMode` binding source

**Question**: where does `editorMode` live such that a top-level
`window.__getEditorMode = () => editorMode` can capture it?

### Context

- `useEditorWorkspaceController.ts:177-178` writes
  `(window as any).__setEditorMode = (mode) => setEditorMode(mode)`.
  That setter is bound at React hook invocation time (every component
  mount creates a new closure).
- The setter currently overwrites itself on every render, which is a
  known wart (not in this cycle's scope to fix — `bindTestHooks` is
  the documented pattern).
- The reader MUST observe the same `editorMode` value that any
  subsequent `__setEditorMode` call would mutate, otherwise tests
  become order-dependent.

### Options

| # | Approach | Pros | Cons |
|---|----------|------|------|
| A | Module-scope `let editorMode: EditorMode` in `engine-bridge.ts`, mutator closure captures it | Single source of truth; setter becomes `(mode) => { editorMode = mode; notifyBridgeListeners(); }`; getter is trivially `() => editorMode`. | Requires a small refactor of how `useEditorWorkspaceController` pushes mode changes — currently `setEditorMode` is React state, and the bridge would need to be the writer. |
| B | Module-scope `let editorMode` set by a `setEditorModeBridge(value)` function exported from `engine-bridge.ts`; `useEditorWorkspaceController` calls `setEditorModeBridge` after every React state update | Backwards compatible: `useEditorWorkspaceController` still owns the React state, just notifies the bridge module. Reader trivially `() => editorMode`. | Two-phase sync (React state + module-scope) — risk of drift if a setter path forgets to notify. |
| C | Read DOM state — `document.querySelector('[data-testid="mode-context-bar"]')?.getAttribute('data-mode')` | No production code change. | Couples test to UI structure; brittle; not a real "bridge". |

### Decision

**Option B**. It preserves the React state ownership (no architectural
fork), keeps the bridge module as the single test-facing surface, and
the drift risk is bounded to one call site in
`useEditorWorkspaceController.ts` (which we touch anyway to add the
notification).

### Implementation sketch

```ts
// engine-bridge.ts
let currentEditorMode: EditorMode = "scene"; // default
export function _setBridgeEditorMode(mode: EditorMode): void {
  currentEditorMode = mode;
}
export function _getBridgeEditorMode(): EditorMode {
  return currentEditorMode;
}

// ... after init_project_store() ...
(window as any).__getEditorMode = (): string => _getBridgeEditorMode();
```

```ts
// useEditorWorkspaceController.ts (existing setter block)
useEffect(() => {
  (window as unknown as { __setEditorMode?: (mode: EditorMode) => void })
    .__setEditorMode = (mode: EditorMode) => {
      setEditorMode(mode);
      // NEW: keep the test bridge in sync
      void import("../../engine-bridge").then((m) => m._setBridgeEditorMode(mode));
    };
  return () => { /* existing cleanup */ };
}, [setEditorMode]);
```

Hmm — the dynamic import introduces an async race. Cleaner pattern:
export `_setBridgeEditorMode` from a small shared `editorModeBridge`
module to avoid the cyclic import:

```ts
// editorModeBridge.ts (NEW, 10 lines)
export type EditorMode = "scene" | "asset-authoring" | "code" | "logic" | "play";
let currentMode: EditorMode = "scene";
export function setEditorMode(mode: EditorMode): void { currentMode = mode; }
export function getEditorMode(): EditorMode { return currentMode; }
```

Then `engine-bridge.ts` reads from `getEditorMode()`, and
`useEditorWorkspaceController.ts` calls `setEditorMode()` after every
React state update. Single shared module, no cyclic imports, sync
writes.

## 3. Decision D2 — Extract `clearWelcomeDismissed` helper or inline?

**Question**: should `clearWelcomeDismissed` be extracted to a
shared helper, or should P8 inline the 3-line OPFS-clear?

### Context

- The helper currently lives in `tests/ux-welcome.spec.ts:17-43`.
- P8 needs the same pattern.
- Cycle 397 already inlined a similar pattern in `_debug.spec.ts`.
- Future tests will likely need it too (welcome-dismissed state is a
  cross-cutting test concern).

### Decision

**Extract**. Move `clearWelcomeDismissed` to
`frontend/tests/helpers/welcome-state.ts` (50 LOC, no behaviour
change). Both `ux-welcome.spec.ts` and `app-characterization.spec.ts`
will import it. This avoids duplicating the OPFS error-handling
pattern and makes future welcome-state tests trivial.

Side-effect: `tests/helpers/welcome-state.ts` is a NEW file in this
cycle. The change to `ux-welcome.spec.ts` is purely an import
refactor — no test behaviour changes.

## 4. Decision D3 — P2 typed-command payload

**Question**: what exact JSON payload does the P2 test send via
`dispatch_command`?

### Context

- The existing typed command surface (`dispatch_command(json)`) takes
  a JSON string and routes to the Rust command bus.
- The SelectEntity command exists in `crates/editor-core/src/command.rs`
  (verified via `rg "SelectEntity" crates/`) — but the JSON shape
  needs confirmation.

### Decision

**Inspect during apply phase.** If SelectEntity's JSON shape is
documented in `crates/editor-core/src/command.rs`, use that exact
shape. If the command is "verb" only (id optional), send
`{"type":"SelectEntity"}` and assert no throw. The P2 test's intent
is "selection control exists and is reachable", not "selects a
specific entity" — keep the payload minimal.

Document the exact payload in the implementation-receipt after the
apply phase.

## 5. Decision D4 — P4 snake_case: 4-space safety net?

**Question**: should the P4 fix include a regression-guard comment
warning against re-introducing camelCase?

### Decision

**Yes**. The fix carries an inline comment:

```ts
// Snake_case — matches wasm_bindgen convention and tests/multi-scene.spec.ts.
// Do NOT introduce camelCase aliases (`sceneCreate`/`sceneSwitch`/`sceneDelete`).
const hasSceneOps = await page.evaluate(
  () =>
    typeof (window as any).scene_create === "function" &&
    typeof (window as any).scene_switch === "function" &&
    typeof (window as any).scene_delete === "function"
);
```

## 6. File-level delta

| File | LOC | Reason |
|------|-----|--------|
| `frontend/src/engine-bridge.ts` | +8/-0 | Import + bridge assignment (new `__getEditorMode`) |
| `frontend/src/hooks/useEditorWorkspaceController.ts` | +1 | `setEditorMode()` notification call |
| `frontend/src/editorModeBridge.ts` | +12 | NEW module — single source of truth for the bridge state |
| `frontend/tests/helpers/welcome-state.ts` | +45/-0 | NEW helper extracted from `ux-welcome.spec.ts` |
| `frontend/tests/ux-welcome.spec.ts` | -27/+2 | Replace inline `clearWelcomeDismissed` with import |
| `frontend/tests/app-characterization.spec.ts` | +20/-25 | 4 surgical fixes (P2, P4, P5 zero-change, P8) |
| **Total** | **+62/-24 across 6 files** | |

(NB: total LOC is slightly inflated by the new shared modules, but
the net production delta is minimal — 1 new getter bridge + 1 hook
notification.)

## 7. Apply order

1. **Helper extraction** — `tests/helpers/welcome-state.ts` +
   refactor `ux-welcome.spec.ts` (zero behaviour change). Run smoke
   regression to confirm 398 still green.
2. **`editorModeBridge.ts`** — new module, no callers yet.
3. **`engine-bridge.ts`** — wire `__getEditorMode` to `getEditorMode()`.
4. **`useEditorWorkspaceController.ts`** — add `setEditorMode()`
   notification call after React state update.
5. **P2/P4/P5/P8 fixes** — 4 surgical edits in
   `app-characterization.spec.ts`.
6. **Smoke full run** — `npx playwright test --project=smoke
   tests/app-characterization.spec.ts tests/editor-ready.spec.ts`
   should now report 12 passed + 1 failed (S2 deferred).
7. **Regression run** — `--project=full` cycle 398 invariants
   (8/8 still pass).

## 8. Risk register

| Risk | Mitigation |
|------|------------|
| Helper extraction breaks `ux-welcome.spec.ts` (e.g., async timing) | Run smoke + full regression immediately after step 1 |
| `editorModeBridge` import cycle | Use a leaf module (no imports from React) |
| `setEditorMode()` notification runs after React state has unmounted | The notification is sync, no React dep; safe across renders |
| `dispatch_command` typed command throws for unknown command type | P2 sends `SelectEntity` (known type) — verified in `crates/editor-core/src/command.rs` during apply |
| P8 OPFS-clear semantics differ from `ux-welcome.spec.ts` | Same helper, same semantics |
| tsc complains about unused `_setBridgeEditorMode` if we never re-export | We export it; the bridge module imports `setEditorMode` directly |
| eslint complains about `_` prefix on a public export | Use `_` prefix per repo convention; if lint rejects, rename to `setEditorModeForBridge` |

## 9. Concrete acceptance check (single command)

```bash
cd /var/home/rubentxu/Proyectos/rust/bevy-2d-editor/frontend
timeout 240 npx playwright test --project=smoke \
    tests/app-characterization.spec.ts \
    -g 'P2:|P4:|P5:|P8:' \
    --reporter=line
```

Expected:

```text
Running 4 tests using 1 worker
  4 passed (XX.Xs)
```

Plus the smoke full-cohort invariant:

```bash
cd /var/home/rubentxu/Proyectos/rust/bevy-2d-editor/frontend
timeout 240 npx playwright test --project=smoke \
    tests/app-characterization.spec.ts \
    --reporter=line
```

Expected:

```text
  8 passed (XX.Xs)
```

(The previously-failing 4 now pass. The previously-passing 4 stay
passing. Total 8/8.)

Plus the S2 deferred invariant:

```bash
cd /var/home/rubentxu/Proyectos/rust/bevy-2d-editor/frontend
timeout 240 npx playwright test --project=smoke \
    tests/editor-ready.spec.ts \
    --reporter=line
```

Expected:

```text
  5 passed (XX.Xs)
  1 failed (S2 — deferred to cycle 400)
```
