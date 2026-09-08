# BJ-3 — UI entity creation test (archive-manifest)

**Cycle**: `p-28fce7028ac3c497/bj3-ui-entity-creation`
**Sequence**: 274 (2026-09-08)
**Tag**: v0.109.6
**Phase**: archive

## Archive summary

| Item | Value |
|------|-------|
| Cycle | p-28fce7028ac3c497/bj3-ui-entity-creation |
| Status | CLOSED |
| Path | A-min |
| Tag | v0.109.6 |
| Code-commit SHA | `f2ee966` |
| Trunk SHA (HEAD) | `f2ee966` (will become archive commit) |
| Origin/main SHA | `f2ee966` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |
| Cycle span (sequence) | 265 → 274 (10 events) |

## Artifacts archived

```
docs/sddk/bj3-ui-entity-creation/explore-report.md
docs/sddk/bj3-ui-entity-creation/spec.md
docs/sddk/bj3-ui-entity-creation/specification.md (alias)
docs/sddk/bj3-ui-entity-creation/implementation-receipt.md
docs/sddk/bj3-ui-entity-creation/verify-report.md
docs/sddk/bj3-ui-entity-creation/debt-report.json
docs/sddk/bj3-ui-entity-creation/release-receipt.md
docs/sddk/bj3-ui-entity-creation/merge-receipt.md
```

## Code released

```
frontend/tests/ui-entity-creation.spec.ts  NEW (100 lines, 2 tests)
```

## BJ-3 status after this cycle

| Sub-deliverable | Status |
|-----------------|--------|
| BJ-3 UI-creation Playwright test | ✅ (BJ-3 cycle v0.109.6 — this cycle) |

## Carry-forward

- BJ-5 contact-death (logic-graph runtime bridge) — needs schema
  extension + runtime bridge from declarative logic graphs to
  imperative Bevy systems. Out of scope for v0.109.x.

## G1 status after BJ-3

| Sub-deliverable | Status | Tag |
|-----------------|--------|-----|
| BJ-1 test_helpers | ✅ | v0.109.1 |
| BJ-2 archcheck | ✅ | v0.109.1 |
| BJ-3 UI-creation Playwright | ✅ | v0.109.6 |
| BJ-4 player movement | ✅ | v0.109.2 |
| BJ-5 pickup collision | ✅ | v0.109.3 |
| BJ-5 jump | ✅ | v0.109.4 |
| BJ-5 enemy patrol | ✅ | v0.109.5 |
| BJ-5 contact-death | ❌ | (deferred — logic-graph runtime) |

**G1 has 7/8 sub-deliverables closed.** Only BJ-5 contact-death
remains, which is out of scope for the v1.0-stabilization P1
sample game (it requires declarative logic graph runtime
infrastructure that lives in a separate effort).

## Cycle closure event sequence

| Event | Sequence |
|-------|----------|
| cycle.start | 265 |
| phase.explore.complete | 266 |
| phase.specify.complete.a-min | 268 |
| phase.build.complete | 270 |
| phase.verify.complete.a-min | 272 |
| release.complete | 274 |
| archive.complete | (this commit) |

## Lessons (final)

1. **A-min for test-only cycles** is the right shape when the
   cycle's only change is a new test file. The pattern (spec →
   build → verify, no design phase) matches the complexity.

2. **Reusing `waitForEditorReady` and `add-entity-btn` testids**
   keeps new specs consistent with the e2e-game-creation suite.
   Adding tests is now mechanical because the harness is mature.

3. **BJ-3 closure** leaves G1 at 7/8 sub-deliverables. The
   remaining BJ-5 contact-death requires logic-graph runtime
   infrastructure that lives in a separate effort (the
   `rig-agent-runtime-foundation` cycle, which is PAUSED per user
   directive).
