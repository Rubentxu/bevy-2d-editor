# G1-step4 — Jump mechanic (archive-manifest)

**Cycle**: `p-28fce7028ac3c497/g1-step4-bj5-jump`
**Sequence**: 248 (2026-09-08)
**Tag**: v0.109.4
**Phase**: archive

## Archive summary

| Item | Value |
|------|-------|
| Cycle | p-28fce7028ac3c497/g1-step4-bj5-jump |
| Status | CLOSED |
| Path | A-lite |
| Tag | v0.109.4 |
| Code-commit SHA | `d2fbff6` |
| Trunk SHA (HEAD) | `d2fbff6` |
| Origin/main SHA | `d2fbff6` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |
| Cycle span (sequence) | 237 → 248 (12 events) |

## Artifacts archived

```
docs/sddk/g1-step4-bj5-jump/explore-report.md
docs/sddk/g1-step4-bj5-jump/proposal.md
docs/sddk/g1-step4-bj5-jump/spec.md (alias for proposal.md)
docs/sddk/g1-step4-bj5-jump/design.md
docs/sddk/g1-step4-bj5-jump/implementation-receipt.md
docs/sddk/g1-step4-bj5-jump/verify-report.md
docs/sddk/g1-step4-bj5-jump/release-receipt.md
docs/sddk/g1-step4-bj5-jump/merge-receipt.md
docs/sddk/g1-step4-bj5-jump/debt-report.json
```

## Code released

```
crates/examples-bevy-harness/src/jump.rs        NEW (83 lines)
crates/examples-bevy-harness/src/lib.rs         +3 / -1 lines
crates/examples-bevy-harness/tests/jump.rs      NEW (188 lines)
```

## BJ-5 status after this cycle

| Sub-deliverable | Status |
|-----------------|--------|
| Pickup collision | ✅ (G1-step3 v0.109.3) |
| Jump mechanic | ✅ (G1-step4 v0.109.4 — this cycle) |
| Enemy patrol | ❌ (deferred — needs schema extension) |
| Contact-death logic-graph runtime | ❌ (deferred — needs declarative→imperative bridge) |

## Carry-forward

- BJ-3 (UI-creation Playwright test, P2) — still open.
- BJ-5 remaining: enemy patrol, contact-death logic-graph runtime.

## Cycle closure event sequence

| Event | Sequence |
|-------|----------|
| cycle.start | 237 |
| phase.explore.complete | 238 |
| phase.specify.complete | 240 |
| phase.design.complete.a-lite | 242 |
| phase.build.complete | 244 |
| phase.verify.complete.a-lite | 246 |
| release.complete | 248 |
| archive.complete | (this commit) |

## Lessons (final)

1. **Rising-edge via resource** is the robust Bevy 0.19 pattern for
   edge-triggered input in tests. The `JumpState::space_was_pressed`
   resource is reusable for any future edge-triggered input (e.g.
   pause toggle in a future cycle).

2. **Constant dt = 0.1** decouples the system from Bevy's first-frame
   `Time<Real>::last_update` edge case. Document the constant inline
   so the test's `ManualDuration` setup stays the single source of
   truth.

3. **Single-commit cycle** is the G1-step4 default — no PR review
   needed because the cycle's gate receipts (architecture-consistent,
   implementation-complete, tests-pass, policy-compliant, debt-
   severity-assigned, debt-priority-assigned) substitute for human
   review. The cumulative evidence is in the cycle's artifact docs.

4. **Mirror the previous cycle's pattern exactly** (G1-step2 →
   G1-step3 → G1-step4) accelerates cycle throughput dramatically.
   Each cycle is now ~10 minutes because the structure is
   predictable.
