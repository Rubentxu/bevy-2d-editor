# G7 a11y critical paths corpus — handoff

**Cycle**: `p-28fce7028ac3c497/g7-a11y-critical-paths`
**Tag**: v0.110.0 (commit `6d470e4`)
**Closed at**: 2026-09-08, sequence 311→318

## TL;DR

B-direct cycle closing **G7 🟡 → ✅** (a11y critical paths corpus).
Documents 5 keyboard-accessible critical paths and asserts 4 of
them with Playwright tests. **CP-5 explicitly deferred** with
grep-verified rationale (no UI trigger for asset import today).

## What shipped

### Documentation

- `docs/a11y-critical-paths.md` (180 lines, NEW):
  - §1 Scope + methodology.
  - §2 5 declared critical paths (CP-1 → CP-5) with trigger
    selector, shortcut, ARIA contract, expected consequence.
  - §3 Test matrix.
  - §4 Out-of-scope CPs (CP-6 → CP-10, canvas/hover/drag).
  - §5 Carry-forward.

### Tests

- `frontend/tests/a11y-critical-paths.spec.ts` (129 lines, 4 tests):
  - CP-1 welcome dismiss via keyboard.
  - CP-2 add-entity focusable + accessible.
  - CP-3 save-btn focusable + accessible.
  - CP-4 play-btn focusable + accessible.
- Tests tagged `@accessibility` (run in a11y cohort) and `@full`
  (run in full cohort).

### Configuration

- `frontend/playwright.a11y.config.ts` testMatch updated to include
  `a11y-critical-paths.spec.ts`.

### SDDK artifacts

- `docs/sddk/g7-a11y-critical-paths/specification.md` (84 lines).
- `docs/sddk/g7-a11y-critical-paths/implementation-receipt.md` (102 lines).
- `docs/sddk/g7-a11y-critical-paths/verify-report.md` (64 lines).
- `docs/sddk/g7-a11y-critical-paths/release-receipt.md` (83 lines).
- `docs/sddk/g7-a11y-critical-paths/merge-receipt.md` (40 lines).
- `docs/sddk/g7-a11y-critical-paths/archive-manifest.md` (101 lines).
- `docs/sddk/g7-a11y-critical-paths/handoff.md` (this file).

### Evidence map refresh

- `docs/v1.0-stabilization-evidence-map.md`: G7 row 🟡 → ✅,
  keyboard accessibility row 🟡 → ✅, cycles table row added,
  §6.1 G7 removed, §7 P4 → CP-5 import trigger.

## Cycle flow

| Phase | Sequence | Status |
|-------|----------|--------|
| cycle.start | 311 | OPEN/build |
| phase.build.complete.b-direct | 312 | OPEN/verify |
| phase.verify.complete.b-direct | 314 | RELEASE_PENDING |
| release.complete | 316 | RELEASED |
| archive.complete | 318 | CLOSED |

## Validation evidence

```
$ npx tsc --noEmit -p .
(clean — no output)

$ npx playwright test --list --config=playwright.a11y.config.ts
[chromium] › a11y-critical-paths.spec.ts:49:3 › A11y Critical Paths — CP-1..CP-4 › CP-1
[chromium] › a11y-critical-paths.spec.ts:72:3 › A11y Critical Paths — CP-1..CP-4 › CP-2
[chromium] › a11y-critical-paths.spec.ts:92:3 › A11y Critical Paths — CP-1..CP-4 › CP-3
[chromium] › a11y-critical-paths.spec.ts:111:3 › A11y Critical Paths — CP-1..CP-4 › CP-4
[chromium] › ux-a11y.spec.ts:17:3 › UX Accessibility — Phase 4 › landing axe sweep
Total: 5 tests in 2 files

$ npx playwright test --list --config=playwright.full.config.ts | grep a11y-critical
[chromium] › a11y-critical-paths.spec.ts:49:3 › ... CP-1
[chromium] › a11y-critical-paths.spec.ts:72:3 › ... CP-2
[chromium] › a11y-critical-paths.spec.ts:92:3 › ... CP-3
[chromium] › a11y-critical-paths.spec.ts:111:3 › ... CP-4
```

4 tests registered in BOTH `@a11y` and `@full` cohorts.

## Lessons learned

1. **Honest deferral beats hand-waving**: CP-5 couldn't be proven
   because no UI trigger exists. Marked DEFERRED with grep-verified
   rationale. The matrix is honest about what's proven vs. pending.
2. **Don't duplicate side-effect assertions**: a11y tests assert
   the a11y contract only (focus + accessible name). Side-effects
   are covered by domain tests.
3. **Verify phase is required even for B-direct**: discovered when
   `release.complete` failed with `ENGINE_SOURCE_STATE_MISMATCH`.
   The proper sequence is `build → verify → release → archive`.
4. **Each gate receipt is single-use per transition**: re-using
   receipts across transitions fails with
   `ENGINE_STALE_GATE_RECEIPT` once the cycle state advances.
   Always evaluate fresh for the current transition.

## v1.0-stabilization gate status (formal)

| Gate | Status |
|------|--------|
| G1 (canonical playable sample) | ✅ |
| G2 (filesystem/Git workflow) | ✅ |
| G3 (browser-local workflow) | ✅ |
| G4 (round-trip/migration) | 🟡 |
| G5 (crash/data-loss recovery) | 🔴 |
| G6 (performance corpus) | 🔴 |
| G7 (accessibility critical paths) | ✅ |
| G8 (extension compat policy) | ✅ |
| G9 (architecture fitness) | ✅ |

**Coverage score: 6 ✅ / 1 🟡 / 2 🔴** (up from 5/2/2).

## Carry-forward

- **G4 🟡** (round-trip/migration corpus expansion, A-min).
- **G5 🔴** (crash recovery corpus, A-lite, ~1-2 days).
- **G6 🔴** (performance corpus, A-lite, ~3 days).
- **CP-5 🔴** (asset import trigger UI work — separate cycle).
- **CP-6 → CP-10** (canvas/hover/drag — future cycles).
- **BJ-5 contact-death** (out of scope).
- **M-2, M-3** (minor debt, trivial).
