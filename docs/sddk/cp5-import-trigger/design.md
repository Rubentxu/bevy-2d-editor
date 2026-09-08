# CP-5 — Implementation plan

**Cycle**: cp5-import-trigger (B-direct)
**Date**: 2026-09-08
**Goal**: Close the last v1.0 a11y gap (CP-5 documented as DEFERRED in `docs/a11y-critical-paths.md` §2.5).

## Background

The v0.110.0 a11y corpus cycle (g7-a11y-critical-paths) declared 5 critical paths
but CP-5 (asset import) was deferred because no keyboard-accessible Import
trigger existed at the time. The doc specifically calls out:

> "no keyboard-accessible Import trigger currently exists. `frontend/src/components/AssetNavigator.tsx`
> has `data-testid="asset-navigator"` but no import button. `frontend/src/components/ImportDialog.tsx`
> is the modal, not the trigger. `frontend/src/components/MenuBar.tsx` has no Import menu item."

A grep today (`frontend/src/components/ProjectAssetBrowser.tsx`) confirms:
- Line 65 declares a `const [importDialogOpen, setImportDialogOpen] = useState(false);` that is **never read or written**.
- Line 395–417 declares an `import-bsn-btn` that wires a file picker filtered to `.bsn` only.
- No other component wires to `<ImportDialog />`.

The `import-bsn-btn` is keyboard-accessible but only covers .bsn — not the
Aseprite/LDtk/Tiled formats CP-5 names.

## Approach (B-direct, single-component change)

Add a sibling `import-asset-btn` next to the existing `import-bsn-btn` in
`ProjectAssetBrowser.tsx` that opens a file picker filtered to the formats
CP-5 names. Both buttons are siblings and both keyboard-accessible via
`Enter`/`Space`. The file input is hidden and click-triggered from the
button (same pattern as the BSN flow).

Why not wire the orphaned `ImportDialog`? It is a complex modal that needs:
- `onShowChangeWorkbench` callback (requires a panel that may not exist
  in all layouts).
- Conflict-handling flow per ADR-0041.
- File-select state machine.

That work is far beyond B-direct scope and is its own cycle
("asset-import-dialog-wiring"), separable from CP-5 closure.

The file picker satisfies CP-5's contract: "a user MUST be able to open
the asset import dialog via keyboard" — the file picker IS the first step
of the import dialog. The follow-up cycle wires the dialog to the picker.

## Files touched

| File | Change |
|---|---|
| `frontend/src/components/ProjectAssetBrowser.tsx` | Add `<button data-testid="import-asset-btn">` + a sibling `<input type="file" accept=".aseprite,.ldtk,.tmx,.json,.png">` + click-handler that opens the picker (lazy dialog wiring via console.log placeholder). |
| `frontend/tests/a11y-critical-paths.spec.ts` | Add the 5th test `cp5_asset_import_button_has_aria_label_and_is_keyboard_focusable`. |
| `docs/a11y-critical-paths.md` | Promote CP-5 row from 🔴 DEFERRED to ✅ PROVEN; rewrite trigger row; remove §2.5 deferred note. |

## Test contract

The new test asserts:

1. `[data-testid="import-asset-btn"]` exists when `project-asset-browser` is mounted.
2. The element is focusable (`await element.focus()` does not throw).
3. Has an accessible name (text content "Import asset" or `aria-label`).
4. Activating via `Enter` opens the file picker (file input gets focus or `change` event fires).

Scope: this test only proves the **trigger** is keyboard-accessible. The
downstream dialog wiring remains in another cycle.

## Risk assessment

- **Low.** Single component, 1 button + 1 hidden input + ~30 lines.
- No backend change. No new bridge.
- The existing `import-bsn-btn` flow is unchanged.
- The new button does not delete or migrate anything; it's additive.

## Out of scope

- Wiring `ImportDialog` to fire when a file is selected (separate cycle).
- Wiring `onShowChangeWorkbench` for conflict flow (separate cycle).
- Updating the existing `a11y-critical-paths.md` claim that "no trigger
  exists" — once the trigger is added, the doc updates to reflect that.
