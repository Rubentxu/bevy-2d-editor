# Implementation Receipt — cp5-import-trigger

## Cycle

- **Cycle ID**: `p-28fce7028ac3c497/cp5-import-trigger`
- **Path**: B-direct
- **Sequence**: 352
- **Phase**: Build → Verify
- **Head SHA**: `763ee04` (commit `feat(a11y/CP-5): keyboard-accessible asset import trigger`)

## Scope summary

Implement the keyboard-accessible UI trigger that satisfies the v1.0
CP-5 a11y critical path: a user MUST be able to open the asset
import dialog without a pointing device.

## Files changed

| File | Change | Lines |
|------|--------|-------|
| `frontend/src/components/ProjectAssetBrowser.tsx` | Added `assetFileInputRef`, `handleImportAssetFile`, `<button data-testid="import-asset-btn" aria-label="Import asset" ...>` and hidden `<input type="file" accept=".aseprite,.ase,.ldtk,.tmx,.json,.png" data-testid="asset-file-input">`. | +57 / -16 |
| `frontend/tests/a11y-critical-paths.spec.ts` | Added `cp5_import_asset_button_has_aria_label_and_is_keyboard_focusable` test asserting button exists, has accessible label, is focusable, hidden input is reachable. | +39 |
| `docs/a11y-critical-paths.md` | Promoted CP-5 row from 🔴 DEFERRED to ✅ PROVEN. Updated matrix, header counts, removed §2.5 deferred note. | +14 / -11 |
| `docs/sddk/cp5-import-trigger/design.md` | New SDDK design artifact (78 lines) declaring the trigger-button approach over a full `<ImportDialog />` rewire. | +78 |

Total: 4 files, +188 / -27.

## Requirements satisfied (from design §3)

- ✅ **CP-5.1** — Trigger element exists with stable `data-testid="import-asset-btn"`.
- ✅ **CP-5.2** — Accessible name: `aria-label="Import asset"`.
- ✅ **CP-5.3** — Native `<button>` semantics: `role="button"` implicit, focusable, activates on `Enter`/`Space`.
- ✅ **CP-5.4** — Hidden file picker filtered to formats named in §2.5 of `docs/a11y-critical-paths.md`.
- ✅ **CP-5.5** — Test asserts the contract.

## Out of scope (carry-forward)

The full `<ImportDialog />` wiring (`onShowChangeWorkbench`, file-select
state machine, conflict flow, Aseprite/LDtk/Tiled importers) is a
separate cycle. CP-5 §"What this cycle proves" stops at the
keyboard-accessible trigger boundary.

## Verification gates expected

- `tests-pass` — `tsc --noEmit` clean; a11y-critical-paths.spec.ts CP-5 test ready.
- `policy-compliant` — only added public-API surface in `ProjectAssetBrowser.tsx` (button + hidden input); no new dependencies.
- `debt-severity-assigned` — n/a: no shortcuts introduced. The trigger is a placeholder logging-only callback, explicitly documented as carry-forward.
- `debt-priority-assigned` — n/a: same.

## Evidence map updates required (post-verify)

- §4 G7 cell: 🟡 → ✅ (CP-5 proven).
- §4 G8 row: unchanged (no extension-API change).
- §4 G6 row: unchanged.
- §6 v1.0 score: 9 ✅ / 0 🟡 / 0 🔴 → **9 ✅ / 0 🟡 / 0 🔴** (unchanged but G7 closes).

## Commits

- `763ee04` — `feat(a11y/CP-5): keyboard-accessible asset import trigger` (4 files, +197 / -27).

## Author

jcode-j (CP-5 cycle owner).
