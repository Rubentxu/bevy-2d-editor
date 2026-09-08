# Handoff — import-dialog-wiring

## Cycle

- **Cycle ID**: `p-28fce7028ac3c497/import-dialog-wiring`
- **Tag**: `v0.110.5`
- **Closed**: 2026-09-08T16:53Z
- **Closed by**: jcode-j

## What this cycle delivered

Wires the orphaned `<ImportDialog />` (complete since v0.93 ADR-0041) to the
keyboard-accessible `import-asset-btn` trigger added in v0.110.4
(cp5-import-trigger cycle). The dialog now opens when the user picks a
file via `import-asset-btn`, displays 3 importer kinds (Aseprite / LDtk /
Tiled), and supports all phases (idle / loading / ready / importing /
success / conflict / error). Conflict flow routes to the Change Workbench
panel via the new `window.__setActiveBottomTab('workbench')` test bridge.

## Coverage score

- Before: 9 ✅ / 0 🟡 / 0 🔴 (terminal v0.110.4 state).
- After: 9 ✅ / 0 🟡 / 0 🔴 — same numeric; **CP-5 carry-forward closed**.

## Files added/modified

- `frontend/src/components/ProjectAssetBrowser.tsx` (+60/-16) — Imported
  `ImportDialog`, replaced placeholder `handleImportAssetFile`, added
  `handleImportDialogImported` + `handleShowChangeWorkbench`, rendered
  `<ImportDialog />`.
- `frontend/src/components/ImportDialog.tsx` (+2) — Added
  `data-testid="import-dialog"` and `tabIndex={-1}` for test targeting
  and a11y focus.
- `frontend/src/components/Dock/BottomDock.tsx` (+13) — Imported
  `useEffect`, mounted/unmounted `window.__setActiveBottomTab` test
  bridge (parallels `__setEditorMode`).
- `frontend/tests/import-dialog.spec.ts` (+270) — New spec, 4 scenarios
  (S1 happy path + Escape, S2 success + custom event, S3 conflict
  routing via bridge, S4 error-phase UX), 8 tests total.
- 8 SDDK artifacts (explore, spec, implementation-receipt, verify-report,
  debt-report, release-receipt, merge-receipt, archive-manifest).

## Commits

- (build) — `feat(import-dialog): wire <ImportDialog /> to import-asset-btn + add 4-scenario spec`
- `d24086c` — implementation receipt
- (verify) — verify + debt reports
- `043b86e` — release + merge receipts

## What did NOT change

- No new dependencies added.
- No Rust / WASM changes.
- No DB migrations.
- No configuration changes.
- No new public APIs (only an internal `useEffect` for the test bridge).

## Carry-forward (closed by this cycle)

- ✅ **CP-5 full `<ImportDialog />` wiring** — closed. Trigger + dialog +
  state machine + change-workbench routing + 3 importer kinds + a11y
  contract + test coverage.

## Carry-forward (still open, separate cycles)

- Highlight specific `changeSetId` in ChangeWorkbench panel when
  redirected from a conflict.
- `ImportDialog.handleReimport` flow (stub today).
- Rust-side importer plugins (Aseprite binary, full LDtk schema).

## Next recommended cycles (per ROADMAP)

Carry-forward queue order (CP-5 carry-forward is now drained):

1. Tutorial sample (P1) — guided UX onboarding.
2. Highlight specific `changeSetId` in ChangeWorkbench (deferred from
   this cycle; small follow-up).
3. `ImportDialog.handleReimport` flow (deferred from this cycle).
4. Nightly perf CI workflow.
5. MAX_SCENES lift (currently hard-coded to 16).
6. Criterion Rust benchmarks.
7. M-2/M-3 soft warnings (perf baselines).
8. BJ-5 contact-death.
9. v1.0 declaration PR (terminal milestone after carry-forward drains).
10. `rig-agent-runtime-foundation` (user-paused).

## Verification

- 8/8 `import-dialog.spec.ts` tests pass (× accessibility + full projects).
- 10/10 `a11y-critical-paths.spec.ts` tests pass (no regression).
- TypeScript clean (`tsc --noEmit -p .`).

## Author

jcode-j (import-dialog-wiring cycle owner).
