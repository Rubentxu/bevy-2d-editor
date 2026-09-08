# Implementation Receipt — `fix-5-p3-smoke-failures`

> **Cycle:** `p-28fce7028ac3c497/fix-5-p3-smoke-failures`
> **Path:** A-lite
> **Phase:** Build
> **Sequence:** 414

---

## Files changed

| File | Status | LOC | Reason |
|------|--------|-----|--------|
| `frontend/src/editorModeBridge.ts` | NEW | +40 | Single source of truth for the `editorMode` test-bridge surface (Decision D1) |
| `frontend/src/engine-bridge.ts` | MODIFIED | +6/-0 | Wire `window.__getEditorMode` to `getEditorMode()` |
| `frontend/src/hooks/useEditorWorkspaceController.ts` | MODIFIED | +5/-1 | Notify `editorModeBridge` after every React state update in `__setEditorMode` |
| `frontend/tests/helpers/welcome-state.ts` | NEW | +53 | Extract `clearWelcomeDismissed` helper from `ux-welcome.spec.ts` (Decision D3) |
| `frontend/tests/ux-welcome.spec.ts` | MODIFIED | -27/+3 | Replace inline `clearWelcomeDismissed` with import (zero behaviour change) |
| `frontend/tests/app-characterization.spec.ts` | MODIFIED | +30/-25 | 4 surgical test fixes (P2, P4, P5 zero-change, P8) |
| **Total** | | **+107/-24** | |

## Per-REQ evidence

### REQ-1 — P2 phantom `__selectEntity` → real `__setSelectedEntityId`

**Decision (Decision D3 from design.md)**: assert `__setSelectedEntityId`
instead of the phantom `__selectEntity`. P2's intent is "selection
control exists and is reachable", which the real bridge satisfies.

**Verification**: `app-characterization.spec.ts:88-93`:

```ts
const hasSelectionController = await page.evaluate(
  () => typeof (window as any).__setSelectedEntityId === "function"
);
expect(hasSelectionController).toBe(true);
```

`__setSelectedEntityId` is declared in
`useEditorWorkspaceController.bindTestHooks:184-186`.

### REQ-2 — P4 snake_case (Decision D4)

**Decision**: switch from `sceneCreate/Switch/Delete` (camelCase, never
existed) to `scene_create/switch/delete` (the actual wasm_bindgen
exports).

**Verification**: `app-characterization.spec.ts:148-150`:

```ts
typeof (window as any).scene_create === "function" &&
typeof (window as any).scene_switch === "function" &&
typeof (window as any).scene_delete === "function"
```

All three are declared in `engine-bridge.ts:299-305`.

### REQ-3 — P5 composition-root getter `__getEditorMode`

**Decision (D1)**: extract `editorMode` to a leaf module
(`frontend/src/editorModeBridge.ts`) so the bridge reader and React
state writer can both observe the same value without cyclic imports.

**Wiring**:

```ts
// editorModeBridge.ts
let currentMode: EditorMode = "scene";
export function setEditorMode(mode: EditorMode): void { currentMode = mode; }
export function getEditorMode(): EditorMode { return currentMode; }
```

```ts
// engine-bridge.ts:310-315 (NEW)
(window as any).__getEditorMode = (): string => getEditorMode();
```

```ts
// useEditorWorkspaceController.ts:178-183 (MODIFIED)
.__setEditorMode = (mode: EditorMode) => {
  setEditorMode(mode);
  setEditorModeForBridge(mode); // NEW: bridge sync
};
```

**Verification**: P5's disjunct
`typeof (window as any).__getEditorMode === "function"` now evaluates
to `true`.

### REQ-4 — P8 OPFS-clear + reload pattern

**Decision (D3)**: extract `clearWelcomeDismissed` to
`tests/helpers/welcome-state.ts`; replace fixture-misuse with the
3-phase pattern from `ux-welcome.spec.ts:46-61`.

**Files**: helper at
`frontend/tests/helpers/welcome-state.ts:23-50`; P8 at
`frontend/tests/app-characterization.spec.ts:223-256`.

### REQ-5 — `ux-welcome.spec.ts` refactor (zero behaviour)

**Decision**: replace inline `clearWelcomeDismissed` with import from
the new helper. Behaviour byte-identical.

**Files**: `frontend/tests/ux-welcome.spec.ts:1-49`.

### REQ-6 — Build phase invariant

| Check | Result |
|-------|--------|
| `tsc --noEmit` | exit 0 (6.3s, no output) |
| `cargo check -p editor-wasm` | exit 0 (4.54s, 158 pre-existing warnings) |
| `app-characterization.spec.ts` (smoke) | **8/8 passed (61.6s)** |
| `ux-welcome.spec.ts` (full) regression | **3/3 passed (35.6s)** |
| `multi-scene.spec.ts` (persistence) regression | **4/4 passed (41.4s)** |

## Cross-cycle invariants preserved

- `ux-welcome.spec.ts` 3/3 ✅ — helper extraction is byte-identical
- `multi-scene.spec.ts` 4/4 ✅ — snake_case invariant intact
- `app-characterization.spec.ts` 8/8 ✅ — all 4 originally-failing
  tests now pass; the 4 originally-passing tests still pass

## Decisions captured in design.md

- **D1**: editorModeBridge leaf module (implemented)
- **D2**: extract clearWelcomeDismissed helper (implemented)
- **D3**: P2 assertion pivot to `__setSelectedEntityId` (implemented)
  — supersedes design.md D3 (typed-command path); reasoning in
  §"Why D3 was overridden" below
- **D4**: snake_case + regression-guard comment (implemented)

### Why D3 was overridden

Design D3 anticipated sending `dispatch_command({type:"SelectEntity",
id:""})` via the typed command path. During apply we discovered
**`Command::SelectEntity` does not exist** in the `editor_model`
command enum (verified via
`crates/editor-model/src/command.rs:33-188`). The typed command path
is reserved for document mutations; selection is a UI-level state,
not a document mutation. P2's intent — "selection control exists" —
is correctly satisfied by asserting the real bridge
`__setSelectedEntityId` (a setter) or, equivalently, by exercising
the workspace controller's `selectEntity` handler. We chose the
bridge assertion because it's a one-liner that doesn't require
loading a scene.

The selection command catalog (`workspaceCommands.selectEntity` in
`commands/catalog.ts:96-104`) is reachable via
`controller.commands.selectEntity` and is exercised by
`schema-authoring.spec.ts` directly. No production change is needed
to "add selection support"; the support exists, just not via the
phantom `__selectEntity` window export the test assumed.

## Open carry-forwards (unchanged by this cycle)

- **S2 pre-ready action observable feedback** — out of scope; deferred
  to cycle 400. The S2 test fails with the same root cause as before
  (`data-testid="tab-scenes"` not present + click-before-ready race);
  no progress expected until cycle 400 introduces the observable
  feedback contract.

## Notes for verify phase

- All test commands listed under REQ-6 use the `frontend/`
  working directory.
- Smoke cohort for this cycle: `tests/app-characterization.spec.ts`
  (8 tests, 8/8 expected green).
- Regression cohorts: `tests/ux-welcome.spec.ts` (3 tests,
  `@full`) and `tests/multi-scene.spec.ts` (4 tests, `@persistence`).
- The 3/3 + 4/4 + 8/8 invariants prove that the helper extraction
  and snake_case pivots are non-disruptive.
