# Archive Report — `v0.82-p2-floating-multi-select`

- Change: `v0.82-p2-floating-multi-select`
- Status: ✅ SHIPPED
- Version: `v0.82.0` (tag pending release)
- Roadmap entry: Hito 7 (carried from `docs/ROADMAP_addendum_v0.81.md` lines 109–118)
- ADR: [ADR-0025](../../adr/0025-floating-panels-multi-select.md)
- PRs: #117 (PR1/2), #118 (PR2/2) — both stacked-to-main, squash-merged
- Merge commits: `abde2cb` (PR1), `364cc32` (PR2)

## Artifacts archived

- `docs/sddk/archive/2026-07-24-floating-multi-select-v0.82-p2/source/explore-report.md`
- `docs/sddk/archive/2026-07-24-floating-multi-select-v0.82-p2/source/proposal.md`
- `docs/sddk/archive/2026-07-24-floating-multi-select-v0.82-p2/source/spec.md`
- `docs/sddk/archive/2026-07-24-floating-multi-select-v0.82-p2/source/design.md`
- `docs/sddk/archive/2026-07-24-floating-multi-select-v0.82-p2/source/tasks.md` (all 57 phases `[x]`)

## Source of truth updated

- `docs/adr/0025-floating-panels-multi-select.md` accepted (status: Accepted).
- `docs/ROADMAP_addendum_v0.81.md` v0.82 candidates #2 (Floating panels) and #3 (Inspector multi-select) marked ✅ DONE with PR/Version links.
- `docs/ROADMAP.md` Last-updated footer rolled forward to v0.82.0.

## Code changes shipped

| PR | Scope | Files |
|----|-------|-------|
| #117 (PR1/2) | Floating panels (React Portal + pointer-drag) + DockPrefs v3 schema (`floats` field) + OPFS round-trip + v2→v3 migration | 10 files |
| #118 (PR2/2) | Inspector multi-select (`Set<StableId>` + `lastClickedId`), Shift/Ctrl/Cmd click modifiers, Esc/Ctrl+A, mixed-value markers, `SetComponentFieldOnMultiple` Rust command, batched Delete | 10 files, +1455 / -4 |

## Verification snapshot

- Rust: 536/536 editor-core tests pass (7 new `multi_select_command` tests).
- Clippy: 0 new errors introduced (5 pre-existing baseline only — `BsnIrNode.kind` fixture + PI approximations + `scene_instance_resync` minimum-max comparison — all documented in ADR-0025 §Consequences).
- Frontend lint (`npm run lint`): 0 warnings.
- TypeScript (`tsc --noEmit`): 0 errors.
- Vite build: clean.
- Bundle delta (gzipped):
  - Pre-cycle (v0.82 P1): 352.70 KB
  - Post-PR1 (#117): 354.80 KB (+2.10 KB)
  - Post-PR2 (#118): 346.18 KB (gzip of PR2 `index-*.js`; cumulative overage +3.48 KB above 350 KB target — within ADR-0025 estimate).
- Playwright:
  - `tests/ux-floating-panel.spec.ts` — 5/5 (PR1)
  - `tests/ux-multi-select.spec.ts` — 6/6 (PR2): F6 modifiers, F7 Ctrl+A+Esc, F7b Esc no-op in input, F8 mixed marker, F9 multi-edit dispatch, F10 batched Delete.

## Architectural deltas worth noting

1. **Selection state moved to `App.tsx`** — single-source-of-truth `Set<StableId>` + `lastClickedId`. Derived `selectedEntityId` preserves backward compat for Toolbar / StatusBar / inspector rename paths.
2. **`CommandError::InvalidArgument(String)` variant added** — required by `SetComponentFieldOnMultiple` validation but available to any future caller that needs to reject malformed input with a human-readable reason.
3. **`Batch` is the canonical atomic dispatcher** — multi-set and multi-delete both go through `apply_batch`, getting partial-failure rollback and a single OperationLog entry for free.
4. **`DockPrefs.schemaVersion` = 3** — additive migration fills `floats = {}` on first load of any v2 prefs file.
5. **Z-index scale formalized** — `--z-floating-panel: 100`, `--z-floating-panel-focused: 101`, `--z-modal: 1000` reserved for future overlays.

## Carried debt (out of scope for this cycle)

- Pre-existing clippy baseline (5 errors) — see ADR-0025 §Consequences.
- Bundle delta +3.48 KB above 350 KB target — chunk-splitting refactor deferred (carried from ADR-0024).
- `Set<StableId>` re-render fan-out at >1000 entities — acceptable at current scale; revisit if virtualization lands.

## Next cycle candidates

From `ROADMAP_addendum_v0.81.md` v0.82 list, the remaining open items after this cycle are:

1. Drag-and-dock region-swap runtime hook (already shipped in v0.82 P1 — `9b076e1`)
2. Tab groups inside docks (#7) — risky, needs UX spike
3. Asset browser thumbnails (#8) — small scope
4. Welcome tour step-through (#9) — onboarding
5. Chunk-splitting refactor to claw back the +3.48 KB bundle overage

`v0.82-p3` candidates not yet named.
