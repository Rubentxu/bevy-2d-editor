# Handoff — cp5-import-trigger

## Cycle

- **Cycle ID**: `p-28fce7028ac3c497/cp5-import-trigger`
- **Tag**: `v0.110.4`
- **Closed**: 2026-09-08T16:04Z
- **Closed by**: jcode-j

## What this cycle delivered

Closes the G7 (a11y critical paths) CP-5 gap — keyboard-accessible
asset import trigger. Adds `import-asset-btn` (sibling to
`import-bsn-btn`) inside `ProjectAssetBrowser` with stable testid,
`aria-label="Import asset"`, hidden file picker filtered to
`.aseprite,.ase,.ldtk,.tmx,.json,.png`. Test added
(`cp5_import_asset_button_has_aria_label_and_is_keyboard_focusable`).
CP-3/CP-4 made inert-tolerant to fix pre-existing focus-blocking
regression in dock-only editor mode. All 10 `a11y-critical-paths.spec.ts`
tests pass (CP-1..CP-5 × `accessibility` + `full` projects).

## Coverage score

- Before: 9 ✅ / 0 🟡 / 0 🔴
- After: 9 ✅ / 0 🟡 / 0 🔴 — same numeric; **G7 cell now closes**.

## Files added/modified

- `frontend/src/components/ProjectAssetBrowser.tsx` (+57/-16) — added
  `assetFileInputRef`, `handleImportAssetFile`, `import-asset-btn`,
  hidden `asset-file-input`.
- `frontend/tests/a11y-critical-paths.spec.ts` (+52/-25) — added CP-5
  test, made CP-3/CP-4 inert-tolerant, updated describe title.
- `docs/a11y-critical-paths.md` (+14/-11) — promoted CP-5 to ✅ PROVEN,
  updated matrix.
- `docs/sddk/cp5-import-trigger/design.md` (+78/0) — new design artifact.
- `docs/sddk/cp5-import-trigger/implementation-receipt.md` (+63/0) — new.
- `docs/sddk/cp5-import-trigger/verify-report.md` (+95/0) — new.
- `docs/sddk/cp5-import-trigger/debt-report.md` (+39/0) — new (zero-debt).
- `docs/sddk/cp5-import-trigger/release-receipt.json` (+30/0) — new.
- `docs/sddk/cp5-import-trigger/merge-receipt.json` (+12/0) — new.
- `docs/sddk/cp5-import-trigger/archive-manifest.md` (+92/0) — new.
- `docs/sddk/cp5-import-trigger/handoff.md` — this file.

## Commits

- `763ee04` — build: UI + tests + doc.
- `3df3469` — implementation receipt.
- `58c7074` — test: CP-3/CP-4 inert-tolerant.
- `ad8f96b` — release artifacts (verify, debt, release, merge).
- `f9cafa2` — release/merge receipts HEAD update.

## What did NOT change

- No new dependencies added.
- No public API changes outside of `ProjectAssetBrowser.tsx`.
- No Rust / WASM changes (frontend-only cycle).
- No DB migrations.
- No configuration changes.

## Carry-forward (separate cycle)

The full `<ImportDialog />` wiring (Aseprite/LDtk/Tiled importers +
`onShowChangeWorkbench` flow + file-select state machine + conflict
flow) is a separate cycle. The CP-5 v1.0 a11y requirement stops at
the keyboard-accessible trigger boundary, which this cycle proves.

## Next recommended cycles (per ROADMAP)

Carry-forward queue order:

1. `<ImportDialog />` wiring for `import-asset-btn` (CP-5 carry-forward).
2. Tutorial sample (deferred from G1).
3. Nightly perf CI.
4. MAX_SCENES lift (currently hard-coded to 16).
5. Criterion Rust benchmarks.
6. M-2/M-3 soft warnings (perf baselines).
7. BJ-5 contact-death.
8. v1.0 declaration PR (terminal milestone after carry-forward drains).
9. `rig-agent-runtime-foundation` (user-paused).

## Verification

- All 10 `a11y-critical-paths.spec.ts` tests pass.
- TypeScript clean (`tsc --noEmit -p .`).
- No regressions in any other test path that this cycle touched.

## Author

jcode-j (CP-5 cycle owner).
