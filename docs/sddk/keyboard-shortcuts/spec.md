# Spec: Keyboard Shortcuts for Undo/Redo

> Change: `keyboard-shortcuts` · Phase: sddk-spec (draft) · Path: A-lite

## §1. Spec Metadata

- **Change:** `keyboard-shortcuts`
- **Phase:** spec (draft, awaiting design)
- **Path:** A-lite
- **Capabilities (NEW):**
  - `keyboard-shortcuts` — browser keydown listener that triggers undo/redo with input-focus guard and Operation Log state gating
- **Capabilities (MODIFIED):** None
- **Source proposal:** [`docs/sddk/keyboard-shortcuts/proposal.md`](../keyboard-shortcuts/proposal.md)
- **Authoritative references:**
  - [Hito 0 §6.4 (Reversible Operation Log)](../../hito-0-spec.md)
  - [`docs/sddk/operation-log/spec.md`](../operation-log/spec.md) — provides `undo()`/`redo()`/`can_undo`/`can_redo` semantics consumed here
  - [CONTEXT.md — Operation Log definition](../../CONTEXT.md)

---

## §2. Capability: `keyboard-shortcuts`

### Requirement: Ctrl/Cmd+Z triggers undo when the editor surface has focus

The system MUST invoke the existing `undo()` action when the user presses `Ctrl+Z` (Windows/Linux) or `Cmd+Z` (macOS) and focus is not inside an `<input>`, `<textarea>`, or `[contenteditable="true"]` element. The native browser undo MUST be suppressed for this gesture.

#### Scenario: Ctrl+Z undoes on Windows/Linux

- GIVEN the editor surface is focused (no input/textarea/contenteditable has focus)
- WHEN the user presses `Ctrl+Z`
- THEN the editor's `undo()` is invoked
- AND the Operation Log applies the inverse of the entry at the cursor
- AND the SceneDocument reflects the pre-command state
- AND the browser's native undo is NOT triggered

#### Scenario: Cmd+Z undoes on macOS

- GIVEN the editor surface is focused
- WHEN the user presses `Cmd+Z` (`metaKey` true, `shiftKey` false)
- THEN the editor's `undo()` is invoked

### Requirement: Ctrl+Y and Ctrl/Cmd+Shift+Z trigger redo when the editor surface has focus

The system MUST invoke the existing `redo()` action when the user presses `Ctrl+Y` (Windows/Linux), `Ctrl+Shift+Z` (Windows/Linux), or `Cmd+Shift+Z` (macOS) and focus is not inside an input-like element. The native browser redo MUST be suppressed.

#### Scenario: Ctrl+Y redoes on Windows/Linux

- GIVEN the editor surface is focused
- WHEN the user presses `Ctrl+Y`
- THEN the editor's `redo()` is invoked

#### Scenario: Ctrl+Shift+Z redoes on Windows/Linux

- GIVEN the editor surface is focused
- WHEN the user presses `Ctrl+Shift+Z`
- THEN the editor's `redo()` is invoked

#### Scenario: Cmd+Shift+Z redoes on macOS

- GIVEN the editor surface is focused
- WHEN the user presses `Cmd+Shift+Z` (`metaKey` true, `shiftKey` true)
- THEN the editor's `redo()` is invoked

### Requirement: Browser-native undo preserved when focus is in an editable field

When focus is in an `<input>`, `<textarea>`, or `[contenteditable="true"]` element, the shortcut listener MUST NOT intercept `Ctrl+Z`, `Cmd+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`, or `Cmd+Shift+Z`. The browser's native undo/redo MUST continue to operate on the editable field.

#### Scenario: Undo in input does not trigger editor undo

- GIVEN focus is in an `<input>` inside the Inspector panel
- WHEN the user presses `Ctrl+Z` inside that input
- THEN the browser's native input undo runs
- AND the editor's `undo()` is NOT invoked
- AND the SceneDocument is unchanged

#### Scenario: Undo in textarea does not trigger editor undo

- GIVEN focus is in a `<textarea>` inside the editor surface
- WHEN the user presses `Ctrl+Z`
- THEN the browser's native textarea undo runs
- AND the editor's `undo()` is NOT invoked

#### Scenario: Undo in contenteditable does not trigger editor undo

- GIVEN focus is in an element with `contenteditable="true"`
- WHEN the user presses `Ctrl+Z`
- THEN the browser's native contenteditable undo runs
- AND the editor's `undo()` is NOT invoked

### Requirement: Undo and redo are gated by Operation Log state

The system MUST NOT invoke `undo()` when `can_undo` is `false`, and MUST NOT invoke `redo()` when `can_redo` is `false`. A no-op in this case MUST NOT surface an error toast or change application state.

#### Scenario: Undo no-op when log is empty or at start

- GIVEN the Operation Log reports `can_undo = false`
- WHEN the user presses `Ctrl+Z` or `Cmd+Z`
- THEN `undo()` is NOT invoked
- AND no error message appears in the UI

#### Scenario: Redo no-op when cursor is at the newest entry

- GIVEN the Operation Log reports `can_redo = false`
- WHEN the user presses `Ctrl+Y`, `Ctrl+Shift+Z`, or `Cmd+Shift+Z`
- THEN `redo()` is NOT invoked
- AND no error message appears in the UI

---

## §3. Capability: `keyboard-shortcuts` — Visual Feedback

### Requirement: Hierarchy and Inspector panels reflect post-shortcut state

After a successful undo or redo triggered by keyboard, the Hierarchy panel MUST render the new entity set and the Inspector panel MUST reflect the new selection state (selection cleared, as with the existing button handler).

#### Scenario: Undo removes an entity from the Hierarchy panel

- GIVEN a SceneDocument containing one Entity `e1`
- AND `e1` is visible in the Hierarchy panel
- WHEN the user triggers Ctrl+Z and the inverse removes `e1`
- THEN the Hierarchy panel no longer renders `e1`
- AND the Inspector panel shows no selection

#### Scenario: Redo re-adds an entity to the Hierarchy panel

- GIVEN the Operation Log has been undone once after creating Entity `e1`
- WHEN the user triggers Ctrl+Y and the forward re-creates `e1`
- THEN the Hierarchy panel renders `e1` again

### Requirement: No-op shortcuts produce no visual change and no error

When the Operation Log reports `can_undo = false` (resp. `can_redo = false`), pressing the corresponding shortcut MUST NOT change the Hierarchy panel, the Inspector panel, or any error indicator.

#### Scenario: Pressing Ctrl+Z with empty log leaves UI unchanged

- GIVEN the Operation Log has no entries
- WHEN the user presses `Ctrl+Z`
- THEN the Hierarchy panel renders the same content as before
- AND no error toast appears

---

## §4. Capability: `keyboard-shortcuts` — Screenshot Verification (E2E)

### Requirement: Ctrl+Z produces a non-zero screenshot diff vs. baseline

A Playwright E2E test MUST load a scene with at least one Entity, capture a baseline screenshot of the editor surface, press `Ctrl+Z`, capture a post-action screenshot, and assert the pixel diff between the two is non-zero. Baseline screenshots MUST be stored under `frontend/tests/baselines/`.

#### Scenario: Undo changes the visual scene

- GIVEN a scene with at least one Entity is loaded via `load_scene_json`
- AND a baseline screenshot is written to `frontend/tests/baselines/keyboard-shortcuts-before.png`
- WHEN the user presses `Ctrl+Z` while the editor surface is focused
- THEN a post-action screenshot is captured to `frontend/tests/baselines/keyboard-shortcuts-after-undo.png`
- AND the pixel diff between the two screenshots has a non-zero changed-pixel count

### Requirement: Undo followed by Redo restores the baseline visual state

A Playwright E2E test MUST perform undo then redo and assert the final screenshot matches the baseline within tolerance.

#### Scenario: Round-trip undo+redo matches baseline

- GIVEN a scene with at least one Entity is loaded
- AND the baseline screenshot `keyboard-shortcuts-before.png` exists
- WHEN the user presses `Ctrl+Z` then `Ctrl+Y`
- THEN a post-roundtrip screenshot is captured
- AND the pixel diff against the baseline has a changed-pixel count below the configured tolerance (≤ 0.1% of pixels)

---

## §5. Out-of-Scope Behaviors (explicit non-goals)

- Customizable keybindings / settings UI
- Shortcuts beyond undo/redo (save, copy, paste, etc.)
- Menu or tooltip hints displaying the shortcut (already present on buttons as `title`)
- Cross-platform detection beyond matching both `metaKey` and `ctrlKey`
- Capturing screenshots for `Ctrl+Y` or `Cmd+Shift+Z` separately — covered by the same baseline via undo+redo roundtrip

---

## §6. Acceptance Criteria

1. Ctrl+Z (Windows/Linux) and Cmd+Z (macOS) trigger the editor's `undo()` and reflect the change in the UI.
2. Ctrl+Y AND Ctrl/Cmd+Shift+Z trigger the editor's `redo()` and reflect the change in the UI.
3. When focus is in an `<input>`, `<textarea>`, or `[contenteditable="true"]` element, the shortcut listener does NOT intercept the gesture and the browser's native undo runs.
4. When `can_undo = false`, pressing Ctrl/Z produces no editor action and no error toast.
5. When `can_redo = false`, pressing Ctrl/Y / Ctrl+Shift+Z produces no editor action and no error toast.
6. After a successful undo/redo via shortcut, the Hierarchy panel and Inspector panel reflect the new SceneDocument state.
7. A Playwright test loads a scene, captures `keyboard-shortcuts-before.png`, presses Ctrl+Z, captures `keyboard-shortcuts-after-undo.png`, and asserts a non-zero pixel diff.
8. A Playwright test performs undo then redo and asserts the post-roundtrip screenshot matches the baseline within ≤ 0.1% pixel tolerance.
9. Existing TopBar undo/redo buttons continue to function (no regression in the `undo-redo` capability).
10. All existing Playwright tests still pass (smoke, engine, export, anchor-sync).

---

## §7. Test Plan

| Section | Scenarios | Test type | Rough count |
|---|---|---|---|
| §2.1 Ctrl/Cmd+Z undo | focus-guard + gesture match | TypeScript unit (hook) | 2 |
| §2.2 Ctrl+Y / Ctrl/Cmd+Shift+Z redo | focus-guard + gesture match | TypeScript unit | 3 |
| §2.3 Input-guard | input/textarea/contenteditable | TypeScript unit + Playwright | 3 |
| §2.4 can_undo / can_redo gating | no-op on false | TypeScript unit | 2 |
| §3.1 Visual feedback | hierarchy/inspector updates | Playwright assertion | 2 |
| §3.2 No-op visual | no change on false | Playwright assertion | 1 |
| §4.1 Screenshot diff: undo | pixelmatch non-zero | Playwright (`pixelmatch`) | 1 |
| §4.2 Screenshot diff: round-trip | pixelmatch ≤ 0.1% | Playwright (`pixelmatch`) | 1 |
| **Total** | | | **~15 tests** |

Dev cycle: `just wasm && cd frontend && npx playwright test`.

Baselines live under `frontend/tests/baselines/`:
- `keyboard-shortcuts-before.png` — scene loaded, ready
- `keyboard-shortcuts-after-undo.png` — after one Ctrl+Z
- `keyboard-shortcuts-after-roundtrip.png` — after undo then redo

---

## §8. Notes on Implementation Surfaces (non-binding)

> Not part of the spec contract — captured here for the design phase.

- Listener is registered once on `window` via `keydown` event.
- Gesture matching: `(metaKey || ctrlKey) && key === "z" && !shiftKey` → undo; with `shiftKey` → redo; `(metaKey || ctrlKey) && key === "y"` → redo.
- Input-guard predicate: `e.target.closest("input,textarea,[contenteditable=\"true\"]")` returns non-null → bail before any action.
- After gesture match and before invoking the callback: `e.preventDefault()` to suppress browser-native undo/redo.
- Reuses the existing `handleUndo` / `handleRedo` defined in `App.tsx`; no logic duplication.
- Screenshot diff library: `pixelmatch` + `pngjs` (add to `frontend/devDependencies`); thresholds: non-zero for §4.1, ≤ 0.1% for §4.2.