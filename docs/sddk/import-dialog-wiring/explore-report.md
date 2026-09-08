# Explore Report — import-dialog-wiring

## Cycle

- **Cycle ID**: `p-28fce7028ac3c497/import-dialog-wiring`
- **Path**: A-min
- **Sequence**: 360
- **Phase**: Explore → Spec
- **Pre-flight**: `HEAD = 8a13533` (v0.110.4, cp5-import-trigger archive); trunk clean.

## Intent

Wire the orphaned `<ImportDialog />` to the `import-asset-btn` trigger
added in v0.110.4. The dialog must:

1. Open when the user picks a file via `import-asset-btn`.
2. Show the 3 importer kinds (Aseprite / LDtk / Tiled).
3. Route a `conflict` result to the **Change Workbench** panel
   (existing `change-workbench` panel-id, mounted in the bottom dock).
4. Refresh the asset catalog on success.

## Codebase taxonomy

### Already wired

| Piece | Path | Status |
|-------|------|--------|
| `import-asset-btn` UI trigger | `frontend/src/components/ProjectAssetBrowser.tsx` (CP-5 cycle, v0.110.4) | ✅ |
| `assetFileInputRef` + hidden `<input type="file">` | `ProjectAssetBrowser.tsx:72, 437` | ✅ |
| `handleImportAssetFile` placeholder | `ProjectAssetBrowser.tsx:308-323` | ✅ (logs only) |
| `importDialogOpen` state | `ProjectAssetBrowser.tsx:65` | ✅ (orphaned) |
| `ImportDialog` component | `frontend/src/components/ImportDialog.tsx` (364 LOC) | ✅ complete (ADR-0041) |
| `importers` service (typed bridge bindings) | `frontend/src/services/importers.ts` (270 LOC) | ✅ complete |
| WASM bridge exports | `list_importers_wasm`, `import_external_source_wasm`, `get_external_source_wasm`, `reimport_external_source_wasm`, `register_importer_wasm` | ✅ |
| `ChangeWorkbenchPanel` | `frontend/src/components/ChangeWorkbenchPanel.tsx` (already mounted in BottomDock via `change-workbench` panel-id) | ✅ |
| `useChangeWorkbench` hook (approval / rejection) | `frontend/src/hooks/useChangeWorkbench.ts` | ✅ |
| BottomDock with `activeTab` state | `frontend/src/components/Dock/BottomDock.tsx` | ✅ local state |

### Missing

| Piece | Notes |
|-------|-------|
| `<ImportDialog />` rendered in DOM | No file imports it. |
| `onShowChangeWorkbench` callback wired | The dialog calls `onShowChangeWorkbench?.(state.changeSetId)` but no parent supplies it. |
| BottomDock `activeTab` exposed globally | `activeTab` is component-local state. To switch to `"workbench"` from outside, we need a global setter (test bridge + state context). |
| Conflict path end-to-end | Dialog → `onShowChangeWorkbench(changeSetId)` → ChangeWorkbench opens with that changeset highlighted. |
| Playwright test | No test asserts the dialog flow today. |

## Constraints (from existing code)

- `<ImportDialog />` requires `isOpen`, `onClose`, `onImported`, `onShowChangeWorkbench` props. ADR-0041 declares all 4.
- `<ImportDialog />` accepts a `file` parameter via `document.querySelector<HTMLInputElement>('input[type="file"]')` (line 125) — fragile but works in practice. The dialog's `handleImport` reads from the first `<input type="file">` in the document. We'll ensure the dialog is only rendered when `isOpen=true` and there is exactly one matching input in scope.
- The dialog state machine (idle / loading / ready / importing / success / conflict / error) is implemented.
- The dialog has `role="dialog"`, `aria-modal="true"`, `aria-labelledby="import-dialog-title"`, `Escape` key, click-outside. **A11y-clean.**

## Architectural choices

### A. `onShowChangeWorkbench` callback strategy

Two viable approaches:

1. **Lifting state** — lift `BottomDock`'s `activeTab` to `useDockController` (new hook) or `useWorkspaceController` (existing). ProjectAssetBrowser receives `onShowChangeWorkbench` via `useEditorWorkspaceController` (similar to how it gets `editorMode`).
2. **Test bridge only** — expose `window.__setActiveBottomTab(tab)` mirroring the `__setEditorMode` pattern. ProjectAssetBrowser calls it from `onShowChangeWorkbench` directly. Simpler, consistent with the codebase's test-bridge convention.

**Decision**: approach 2 (test bridge) — matches the `__setEditorMode` pattern already in use (`useEditorWorkspaceController.ts:178`). Keeps state ownership local to BottomDock (single source of truth), avoids a new cross-cutting controller. The bridge is the documented test seam.

### B. Conflict routing — does `changeSetId` reach the workbench?

`<ChangeWorkbenchPanel />` reads pending changesets via `useChangeWorkbench()` (which fetches from a WASM-side pending queue, not from the import result). The dialog's `changeSetId` from the import result won't auto-populate the workbench — they're separate flows today.

**Pragmatic decision**: this cycle routes the user to the workbench tab; **carrying the changeSetId is a future-work item**. The user can review the new changeset from the workbench's pending list. This matches ADR-0041's "redirect to Change Workbench for review" wording — the user lands on the panel; the panel itself shows the new changeset via the existing pending-queue refresh.

### C. Where does `<ImportDialog />` mount?

Three options:

1. Inside `ProjectAssetBrowser` — closest to the trigger. Pros: simple. Cons: dialog is conditional on `asset-authoring` mode being active (won't render in other modes).
2. Inside `useEditorWorkspaceController` — alongside other mode-aware panels. Pros: dialog available in any mode. Cons: requires lifting trigger state out of ProjectAssetBrowser.
3. Inside `App.tsx` — always rendered, conditional on global `importDialogOpen` state. Pros: universal availability. Cons: requires lifting state + trigger out of ProjectAssetBrowser.

**Decision**: option 1 — mount inside `ProjectAssetBrowser`. The `import-asset-btn` lives in this component; the dialog only opens when this component is mounted. The dialog is a modal that overlays the panel, not a separate UI region; this matches `create-asset-btn` → `createDialogOpen` → dialog-in-place pattern (lines 62, 152).

## Spec satisfaction map

| Spec | Source | In this cycle? |
|------|--------|----------------|
| Dialog opens on file pick | design §1 | ✅ |
| 3 importer kinds listed | design §2 | ✅ |
| Success closes dialog + refreshes catalog | design §3 | ✅ |
| Conflict routes to ChangeWorkbench | design §4 | ✅ (tab switch; carry-forward for changeSetId highlighting) |
| Error surfaces to user | design §5 | ✅ (already implemented) |
| `onShowChangeWorkbench` wired | design §6 | ✅ (via test bridge) |
| A11y: `role=dialog`, `aria-modal`, `Escape`, focus trap | design §7 | ✅ (already implemented; we verify it stays so) |

## Carried-forward (NOT in this cycle)

- Highlighting a specific `changeSetId` in the ChangeWorkbench when redirected from a conflict (ADR-0041 §"Conflicts" partially deferred).
- Aseprite binary importer (only LDtk JSON is exercised today by tests). The dialog's importer list comes from `list_importers_wasm`, which is the typed bridge — the actual importer availability is a Rust-side concern.

## Risk register

| Risk | Mitigation |
|------|-----------|
| `<ImportDialog />`'s `document.querySelector` for `<input type="file">` might pick up the wrong input (e.g., `bsn-file-input`). | We scope the dialog's `handleImport` to use the same `assetFileInputRef` we already have. We'll pass the file via a `pendingFile` prop instead of relying on DOM queries. **Actually, simpler**: dialog will get a `defaultFileName` prop from the trigger, and on open the trigger resets its input. The dialog's own file input is the source of truth for re-imports. |
| BottomDock might not be visible (user has docked it elsewhere) when conflict happens. | Add a test that asserts the workbench tab is activated AND `data-testid="dock-bottom"` becomes visible. If the dock is hidden, we surface a toast or status bar notification (out of scope; document as carry-forward). |
| `<ImportDialog />`'s `handleReimport` is a stub today (line 159-180). | Don't expose reimport in this cycle; the dialog's import button (not reimport) is what the trigger calls. Reimport is its own cycle. |

## Conclusion

This cycle is **A-min**, scope is **simple (UI plumbing, no Rust)**. All
required service + state + UI pieces exist; the missing wiring is
narrow and deterministic. Ready to spec.
