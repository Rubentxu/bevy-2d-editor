# G7 — A11y critical paths corpus (archive-manifest)

**Cycle**: `p-28fce7028ac3c497/g7-a11y-critical-paths`
**Sequence**: 316 (release) → 317 (archive)
**Tag**: v0.110.0
**Phase**: archive

## Archive summary

| Item | Value |
|------|-------|
| Cycle | p-28fce7028ac3c497/g7-a11y-critical-paths |
| Status | CLOSED |
| Path | B-direct |
| Tag | v0.110.0 |
| Code-commit SHA | `6d470e4` |
| Trunk SHA (HEAD) | `6d470e4` (will become archive commit) |
| Origin/main SHA | `6d470e4` |
| HEAD == origin/main | ✅ |
| Tag points at commit | ✅ |
| Cycle span | 311 → 317 (7 events) |

## Artifacts archived

```
docs/a11y-critical-paths.md                                                NEW (180 lines)
docs/sddk/g7-a11y-critical-paths/specification.md                          NEW (84 lines)
docs/sddk/g7-a11y-critical-paths/implementation-receipt.md                 NEW (102 lines)
docs/sddk/g7-a11y-critical-paths/verify-report.md                          NEW (64 lines)
docs/sddk/g7-a11y-critical-paths/release-receipt.md                        NEW (83 lines)
docs/sddk/g7-a11y-critical-paths/merge-receipt.md                          NEW (40 lines)
frontend/tests/a11y-critical-paths.spec.ts                                 NEW (129 lines, 4 tests)
frontend/playwright.a11y.config.ts                                         +1 / -1
```

## v1.0-stabilization gate status after this cycle

| Gate | Status |
|------|--------|
| G1 (canonical playable sample) | ✅ |
| G2 (filesystem/Git workflow) | ✅ |
| G3 (browser-local workflow) | ✅ |
| G4 (round-trip/migration) | 🟡 |
| G5 (crash/data-loss recovery) | 🔴 |
| G6 (performance corpus) | 🔴 |
| G7 (accessibility critical paths) | 🟡 → **✅ (4/5 CPs proven)** |
| G8 (extension compat policy) | ✅ |
| G9 (architecture fitness) | ✅ |

**Formal coverage: 6 ✅ / 1 🟡 / 2 🔴** (was 5/2/2). Only G4 remains 🟡; G5 and G6 still 🔴.

## Critical path coverage

| CP | Trigger | Status |
|----|---------|--------|
| CP-1 | `welcome-skip-btn` | ✅ proven |
| CP-2 | `add-entity-btn` / `-empty` | ✅ proven |
| CP-3 | `save-btn` | ✅ proven |
| CP-4 | `play-btn` / `stop-btn` | ✅ proven |
| CP-5 | (no UI trigger) | 🔴 DEFERRED (UI work) |

## Cycle closure event sequence

| Event | Sequence |
|-------|----------|
| cycle.start | 311 |
| phase.build.complete.b-direct | 312 |
| phase.verify.complete.b-direct | 314 |
| release.complete | 316 |
| archive.complete | (this commit) |

## Lessons

1. **Honest deferral beats hand-waving**: CP-5 couldn't be proven
   because no UI trigger exists. The right move was to mark it
   🔴 DEFERRED with grep-verified rationale rather than write a
   fake test or skip the path entirely. The matrix is honest.
2. **Don't duplicate side-effect assertions**: CP-2/3/4 only assert
   the a11y contract (focus + accessible name). Side-effects are
   proven by `ui-entity-creation.spec.ts`, `keyboard-shortcuts.spec.ts`,
   `runtime-preview-v2.spec.ts`. Re-asserting them couples the a11y
   test to business logic.
3. **Gate receipt lifecycle**: each `phase.evaluate-gate` call
   issues a fresh receipt_id. Re-using old IDs from previous
   transitions fails with `ENGINE_STALE_GATE_RECEIPT` once the
   cycle state advances. Always evaluate fresh for the current
   transition.
4. **Verify phase is required even for B-direct**: after build,
   `phase.verify.complete.b-direct` must fire before
   `release.complete`. Discovered when transition failed with
   `ENGINE_SOURCE_STATE_MISMATCH: expects ReleasePending, found Open/Verify`.

## Carry-forward

- **G4 🟡** (round-trip/migration corpus expansion, A-min).
- **G5 🔴** (crash recovery corpus, A-lite, ~1-2 days).
- **G6 🔴** (performance corpus, A-lite, ~3 days).
- **CP-5 🔴** (asset import trigger UI work — separate cycle).
- **CP-6 → CP-10** (canvas/hover/drag — future cycles).
- **BJ-5 contact-death** (out of scope).
- **M-2, M-3** (minor debt, trivial).
