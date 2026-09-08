# Implementation Receipt — import-dialog-wiring

## Cycle

- **Cycle ID**: `p-28fce7028ac3c497/import-dialog-wiring`
- **Path**: A-min
- **Sequence**: 360–363
- **Phase**: Build → Verify
- **Head SHA**: (see git log; current commit = `feat(import-dialog): ...`)

## Scope summary

Wire the orphaned `<ImportDialog />` (frontend/src/components/ImportDialog.tsx,
364 LOC, complete since v0.93 ADR-0041) to the keyboard-accessible
`import-asset-btn` trigger added in v0.110.4 (cp5-import-trigger cycle).

The dialog handles the full import flow: 3 importer kinds (Aseprite,
LDtk, Tiled), destination path, success / conflict / error phases, and
the `onShowChangeWorkbench` callback that routes conflicts to the
existing `change-workbench` panel-id.

## Files changed

| File | Change | Lines |
|------|--------|-------|
| `frontend/src/components/ProjectAssetBrowser.tsx` | Imported `ImportDialog`. Replaced placeholder `handleImportAssetFile` with `setImportDialogOpen(true)` + reset input. Added `handleImportDialogImported` (dispatches `bevy-2d-editor:asset-imported` CustomEvent). Added `handleShowChangeWorkbench` (calls `window.__setActiveBottomTab("workbench")`). Rendered `<ImportDialog />` with `isOpen`, `onClose`, `onImported`, `onShowChangeWorkbench`. | +60 / -16 |
| `frontend/src/components/ImportDialog.tsx` | Added `data-testid="import-dialog"` and `tabIndex={-1}` to the dialog `<div>` so tests and keyboard handlers can target it. | +2 |
| `frontend/src/components/Dock/BottomDock.tsx` | Imported `useEffect`. Added `useEffect` that mounts/unmounts `window.__setActiveBottomTab` (test bridge, mirrors `__setEditorMode`). | +13 |
| `frontend/tests/import-dialog.spec.ts` | New Playwright spec, 4 scenarios (S1 happy path + Escape close, S2 success + custom event, S3 conflict routing via bridge, S4 error-phase UX), 8 tests total (× accessibility + full projects). | +270 |

Total: 4 files, +345/-16.

## Requirements satisfied

- ✅ **REQ-1** Dialog opens on file pick via `import-asset-btn`.
- ✅ **REQ-2** Dialog displays 3 importer kinds (Aseprite / LDtk / Tiled).
- ✅ **REQ-3** User picks destination and clicks Import; the dialog calls `importExternalSource(...)`.
- ✅ **REQ-4** Success closes the dialog and dispatches the `bevy-2d-editor:asset-imported` event for catalog refresh.
- ✅ **REQ-5** Conflict routes to Change Workbench via `__setActiveBottomTab("workbench")`.
- ✅ **REQ-6** Error surfaces to user without closing the dialog.
- ✅ **REQ-7** `Escape` and click-outside close the dialog.
- ✅ **REQ-8** `onShowChangeWorkbench` is wired via the test-bridge pattern.
- ✅ **REQ-9** A11y invariants preserved: `role="dialog"`, `aria-modal="true"`, `aria-labelledby`, `Escape`, `tabIndex={-1}`.
- ✅ **REQ-10** Playwright integration smoke test (`tests/import-dialog.spec.ts`) covers all 4 scenarios.

## Out of scope (carry-forward, documented in spec)

- Highlighting a specific `changeSetId` in the ChangeWorkbench panel when redirected from a conflict. The dialog passes the `changeSetId` to `onShowChangeWorkbench`; the workbench's pending-queue refresh is independent today. ADR-0041 §"Conflicts" partially deferred.
- `ImportDialog.handleReimport` flow is a stub; reimport is its own cycle.
- File-format-specific importer plugins (Aseprite binary, full LDtk schema) are Rust-side concerns.

## Verification gates expected

- `tests-pass` — 8/8 import-dialog.spec.ts pass; 10/10 a11y-critical-paths.spec.ts pass (no regression); `tsc --noEmit` clean.
- `policy-compliant` — only added public-API surface in `ProjectAssetBrowser.tsx` + `BottomDock.tsx` (test bridge, parallel to existing `__setEditorMode`). No new dependencies.
- `debt-severity-assigned` — debt-report to follow in verify phase (zero-debt expected; no shortcuts, no TODO/FIXME/HACK added).
- `debt-priority-assigned` — same.

## Evidence map updates required (post-verify)

- §4 G7 row unchanged (CP-5 already ✅; this cycle is its carry-forward).
- §6.1 carry-forward list: remove "ImportDialog wiring (CP-5 carry-forward)" item.
- ROADMAP row: append cycle entry.

## Commits

- (current commit) — `feat(import-dialog): wire <ImportDialog /> to import-asset-btn + add 4-scenario spec` (4 files, +345/-16).

## Author

jcode-j (import-dialog-wiring cycle owner).
