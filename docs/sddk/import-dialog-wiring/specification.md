# Specification — import-dialog-wiring

## Cycle

- **Cycle ID**: `p-28fce7028ac3c497/import-dialog-wiring`
- **Path**: A-min
- **Sequence**: 361
- **Phase**: Specify → Tasks

## Requirements

### Functional

#### REQ-1 — Dialog opens when a file is selected via `import-asset-btn`

**When** the user clicks `import-asset-btn` and selects an Aseprite / LDtk / Tiled file (or any JSON with the matching extension),
**Then** the `<ImportDialog />` opens with `isOpen=true`.

**Acceptance:**
- The hidden `<input data-testid="asset-file-input">`'s `change` event triggers `handleImportAssetFile`.
- `handleImportAssetFile` calls `setImportDialogOpen(true)`.
- The `<ImportDialog />` becomes visible (`role="dialog"`, `aria-modal="true"`).

#### REQ-2 — Dialog displays 3 importer kinds

**When** the dialog opens,
**Then** the Source Type `<select>` shows all 3 kinds: Aseprite, LDtk, Tiled.

**Acceptance:**
- `listImporters()` resolves with at least 3 descriptors.
- `<select id="import-kind">` contains 3 `<option>` children.
- The currently selected kind is "Aseprite" by default.

#### REQ-3 — User picks destination and triggers import

**When** the dialog is in the `ready` phase and the user has selected a source file + entered a destination path,
**Then** clicking the **Import** button calls `importExternalSource(...)` with the selected kind, source file name, base64-encoded bytes, and destination path.

**Acceptance:**
- The button is `disabled` until both `destinationPath` and `fileName` are set.
- On click, the dialog transitions to `importing` phase.
- `importExternalSource` is called once with all 4 args.

#### REQ-4 — Success closes the dialog and refreshes the catalog

**When** `importExternalSource` returns `{ ok: true, value }`,
**Then** the dialog shows the success phase, the `onImported` callback fires, the dialog closes on **Done** click, and the asset catalog refreshes.

**Acceptance:**
- `onImported(destinationPath)` is called.
- The asset catalog shows the new entry on next refresh.
- The dialog transitions to `success` → on Done click → `isOpen=false`.

#### REQ-5 — Conflict routes the user to the Change Workbench

**When** `importExternalSource` returns a result that the dialog classifies as a conflict (e.g., `result.change_set_id` is set with a non-empty `diff.modified_editor` count),
**Then** the dialog shows the conflict phase, and on **Review in Change Workbench** click, the dialog closes AND the bottom dock's `activeTab` switches to `"workbench"`.

**Acceptance:**
- Conflict phase UI shows: "⚠ Conflicts detected — review required", a diff summary, and two buttons (Cancel, Review in Change Workbench).
- The **Review in Change Workbench** button calls `onShowChangeWorkbench?.(state.changeSetId)` and then `onClose()`.
- The bottom dock's active tab becomes `"workbench"` (assert via `data-testid="bottom-dock-tab-workbench"` `aria-selected="true"`).

#### REQ-6 — Error surfaces to the user without closing the dialog

**When** `importExternalSource` returns `{ ok: false }`,
**Then** the dialog shows the error phase with the message, and stays open until the user clicks **Close**.

**Acceptance:**
- Error phase UI shows the error message in red.
- The dialog does NOT auto-close.
- Clicking **Close** sets `isOpen=false`.

#### REQ-7 — `Escape` and click-outside close the dialog

**When** the dialog is open and the user presses `Escape` (in any phase except `importing`),
**Then** the dialog closes (`isOpen=false`).

**Acceptance:**
- `Escape` does not close during `importing` phase (existing behavior).
- Clicking the `.dialog-overlay` (outside the dialog body) closes the dialog.

### Non-functional

#### REQ-8 — `onShowChangeWorkbench` is wired via the test-bridge pattern

The `onShowChangeWorkbench` callback supplied to `<ImportDialog />` calls `window.__setActiveBottomTab("workbench")` (exposed by BottomDock as a test bridge, parallel to `window.__setEditorMode`).

**Acceptance:**
- `window.__setActiveBottomTab` exists when `BottomDock` is mounted.
- The function accepts a `BottomDockTab` argument and updates the local `activeTab` state.
- Calling it from outside (test) updates the rendered tab.

#### REQ-9 — A11y invariants preserved

`<ImportDialog />` already passes a11y checks (CP-5 cycle verified). This cycle must not regress them.

**Acceptance:**
- `role="dialog"`, `aria-modal="true"`, `aria-labelledby="import-dialog-title"` remain.
- `Escape` key still closes.
- The dialog overlay has `data-testid="import-dialog"` (new in this cycle) so tests can target it.

#### REQ-10 — Playwright integration smoke test

A new test in `frontend/tests/import-dialog.spec.ts` (or `a11y-critical-paths.spec.ts` extension) verifies the end-to-end happy-path:

1. Switch to `asset-authoring` mode.
2. Programmatically open the dialog via the trigger's click (or via `window.__openImportDialog` test bridge).
3. Assert dialog is visible, `role="dialog"`, has 3 importer kinds.
4. Assert dialog can be closed via `Escape`.
5. Assert `data-testid="import-dialog"` exists.

**Acceptance:**
- Test passes in `accessibility` and `full` projects.
- Test runs in <30 s.

### Out of scope (carry-forward)

- Highlighting a specific `changeSetId` in the ChangeWorkbench panel when redirected from a conflict. The dialog passes the `changeSetId` to `onShowChangeWorkbench` but the workbench's pending-queue refresh is independent today.
- The `ImportDialog.handleReimport` flow is a stub; reimport is its own cycle.
- File-format-specific importer plugins (Aseprite binary, full LDtk schema) are Rust-side concerns out of scope for this UI cycle.

## Scenarios (acceptance scenarios)

### S1 — Happy path open and close (no actual import)

```
GIVEN the editor is in asset-authoring mode
WHEN the user clicks `import-asset-btn` and selects any matching file
THEN the <ImportDialog /> opens
  AND the dialog is `role="dialog" aria-modal="true"`
  AND the Source Type <select> shows 3 options (Aseprite, LDtk, Tiled)
WHEN the user presses Escape
THEN the dialog closes (isOpen=false)
```

### S2 — Success import (mocked bridge)

```
GIVEN the editor is in asset-authoring mode
AND list_importers_wasm returns 3 descriptors
AND import_external_source_wasm returns { ok: true, change_set_id: "abc", sidecar_path: "..." }
WHEN the user clicks Import
THEN the dialog shows the success phase
WHEN the user clicks Done
THEN the dialog closes
  AND onImported was called with the destination path
```

### S3 — Conflict routes to Change Workbench

```
GIVEN a reimport produces a result with non-empty diff.modified_editor
WHEN the user clicks Review in Change Workbench
THEN the dialog closes
  AND window.__setActiveBottomTab("workbench") was called
  AND the bottom dock's active tab is "workbench"
```

### S4 — Error phase stays open until user closes

```
GIVEN import_external_source_wasm returns { ok: false, error: "..." }
WHEN the dialog transitions to the error phase
THEN the error message is shown in red
AND the dialog does NOT auto-close
WHEN the user clicks Close
THEN the dialog closes
```

## Files touched

| File | Change | LOC |
|------|--------|-----|
| `frontend/src/components/ProjectAssetBrowser.tsx` | Render `<ImportDialog />`, supply `onClose`, `onImported`, `onShowChangeWorkbench`. Replace placeholder `handleImportAssetFile` with `setImportDialogOpen(true)` + reset input. | ~+30 |
| `frontend/src/components/Dock/BottomDock.tsx` | Expose `window.__setActiveBottomTab` test bridge. | +6 |
| `frontend/tests/import-dialog.spec.ts` | New spec (4 tests, S1–S4). | +120 |
| `frontend/tests/a11y-critical-paths.spec.ts` | Optionally extend with `import-dialog a11y` test (S1 asserts). | +20 |

## Verification gates (A-min)

- `specification-coverage` — each REQ has at least one scenario mapping.
- `requirements-clarity` — no REQ has ambiguity beyond a 1-line clarification.
- `design-coverage` — Design.md (Tasks phase) covers every REQ.
- `tasks-coverage` — Tasks.md maps every REQ → implementation task.
- `tests-pass` — Playwright `import-dialog.spec.ts` passes.
- `policy-compliant` — no new public API outside `ProjectAssetBrowser` + `BottomDock`.
- `debt-severity-assigned` — debt-report assigns severity.
- `debt-priority-assigned` — debt-report assigns priority.
- `no-pending-effects` — clean working tree at release.
- `release-uat-approved` — verify-report confirms all S1–S4 scenarios.
- `ledger-valid` — cycle ledger complete.
- `vault-index-current` — vault updated for new cycle.

## Summary

12 REQs, 4 scenarios, 4 files. Scope: UI plumbing only (no Rust, no schema, no migration).
