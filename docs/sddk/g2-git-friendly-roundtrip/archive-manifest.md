# G2 — Git-friendly round-trip test (archive-manifest)

**Cycle**: `p-28fce7028ac3c497/g2-git-friendly-roundtrip`
**Sequence**: 286 (2026-09-08)
**Tag**: v0.109.7
**Phase**: archive

## Archive summary

| Item | Value |
|------|-------|
| Cycle | p-28fce7028ac3c497/g2-git-friendly-roundtrip |
| Status | CLOSED |
| Path | A-min |
| Tag | v0.109.7 |
| Code-commit SHA | `827250f` |
| Trunk SHA (HEAD) | `827250f` (will become archive commit) |
| Origin/main SHA | `827250f` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |
| Cycle span (sequence) | 277 → 286 (10 events) |

## Artifacts archived

```
docs/sddk/g2-git-friendly-roundtrip/explore-report.md
docs/sddk/g2-git-friendly-roundtrip/spec.md
docs/sddk/g2-git-friendly-roundtrip/specification.md (alias)
docs/sddk/g2-git-friendly-roundtrip/implementation-receipt.md
docs/sddk/g2-git-friendly-roundtrip/verify-report.md
docs/sddk/g2-git-friendly-roundtrip/debt-report.json
docs/sddk/g2-git-friendly-roundtrip/release-receipt.md
docs/sddk/g2-git-friendly-roundtrip/merge-receipt.md
```

## Code released

```
frontend/tests/git-friendly-roundtrip.spec.ts  NEW (182 lines, 2 tests)
```

## v1.0-stabilization gate status after this cycle

| Gate | Status |
|------|--------|
| G1 (canonical playable sample) | ✅ evidence rich (G1-step1 through G1-step5 + BJ-3) |
| G2 (filesystem/Git workflow) | 🟡 → ready to upgrade to ✅ (ADR-0045 + this test + branch protection rule) |
| G3 (browser-local workflow) | ✅ |
| G4 (round-trip/migration) | 🟡 (separate cycle) |
| G5 (crash/data-loss recovery) | 🔴 |
| G6 (performance corpus) | 🔴 |
| G7 (accessibility critical paths) | 🟡 |
| G8 (extension/agent compat policy) | 🔴 |
| G9 (architecture fitness) | ✅ |

**Coverage score after this cycle: 3 ✅ / 3 🟡 / 3 🔴 → 3 ✅ / 3 🟡 / 3 🔴**
(formally unchanged until evidence map is updated; G2 should now
upgrade to ✅).

## Carry-forward

- **BJ-5 contact-death** (out of scope, requires logic-graph runtime).
- **G4 🟡** (round-trip/migration test, separate cycle).
- **G5 🔴** (crash recovery, high scope).
- **G8 🔴** (extension compat policy, doc-heavy).
- **M-2, M-3** (minor debt, trivial).

## Cycle closure event sequence

| Event | Sequence |
|-------|----------|
| cycle.start | 277 |
| phase.explore.complete | 278 |
| phase.specify.complete.a-min | 280 |
| phase.build.complete | 282 |
| phase.verify.complete.a-min | 284 |
| release.complete | 286 |
| archive.complete | (this commit) |

## Lessons (final)

1. **Text-determinism** is the load-bearing property for Git
   friendliness. The round-trip test pins it down so future changes
   can't regress it silently.

2. **A-min for test-only cycles** continues to be the right
   shape: spec → build → verify. No design phase needed when the
   test pattern is already established by sibling specs.

3. **Reusing the OPFS_FILES table** from `e2e-game-creation.spec.ts`
   keeps the new spec consistent. The minor duplication is
   acceptable; future cycle could extract a shared helper.
