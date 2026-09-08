# G1-step5 — Enemy patrol (archive-manifest)

**Cycle**: `p-28fce7028ac3c497/g1-step5-bj5-enemy-patrol`
**Sequence**: 262 (2026-09-08)
**Tag**: v0.109.5
**Phase**: archive

## Archive summary

| Item | Value |
|------|-------|
| Cycle | p-28fce7028ac3c497/g1-step5-bj5-enemy-patrol |
| Status | CLOSED |
| Path | A-lite |
| Tag | v0.109.5 |
| Code-commit SHA | `7006dc7` |
| Trunk SHA (HEAD) | `7006dc7` (will become archive commit) |
| Origin/main SHA | `7006dc7` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |
| Cycle span (sequence) | 251 → 262 (12 events) |

## Artifacts archived

```
docs/sddk/g1-step5-bj5-enemy-patrol/explore-report.md
docs/sddk/g1-step5-bj5-enemy-patrol/proposal.md
docs/sddk/g1-step5-bj5-enemy-patrol/spec.md (alias for proposal.md)
docs/sddk/g1-step5-bj5-enemy-patrol/design.md
docs/sddk/g1-step5-bj5-enemy-patrol/implementation-receipt.md
docs/sddk/g1-step5-bj5-enemy-patrol/verify-report.md
docs/sddk/g1-step5-bj5-enemy-patrol/release-receipt.md
docs/sddk/g1-step5-bj5-enemy-patrol/merge-receipt.md
docs/sddk/g1-step5-bj5-enemy-patrol/debt-report.json
```

## Code released

```
crates/examples-bevy-harness/src/components.rs        +18 / -0 lines
crates/examples-bevy-harness/src/enemy_patrol.rs      NEW (71 lines)
crates/examples-bevy-harness/src/lib.rs               +3 / -1 lines
crates/examples-bevy-harness/src/loader.rs            +7 / -1 lines
crates/examples-bevy-harness/tests/enemy_patrol.rs    NEW (224 lines)
```

## BJ-5 status after this cycle

| Sub-deliverable | Status |
|-----------------|--------|
| Pickup collision | ✅ (G1-step3 v0.109.3) |
| Jump mechanic | ✅ (G1-step4 v0.109.4) |
| Enemy patrol | ✅ (G1-step5 v0.109.5 — this cycle) |
| Contact-death logic-graph runtime | ❌ (deferred — needs declarative→imperative bridge) |

## Carry-forward

- BJ-3 (UI-creation Playwright test, P2) — still open.
- BJ-5 contact-death (logic-graph runtime bridge) — needs schema
  extension + runtime bridge from declarative logic graphs to
  imperative Bevy systems. Out of scope for v0.109.x.

## Cycle closure event sequence

| Event | Sequence |
|-------|----------|
| cycle.start | 251 |
| phase.explore.complete | 252 |
| phase.specify.complete | 254 |
| phase.design.complete.a-lite | 256 |
| phase.build.complete | 258 |
| phase.verify.complete.a-lite | 260 |
| release.complete | 262 |
| archive.complete | (this commit) |

## Lessons (final)

1. **Per-entity state via component** is the right shape for
   multi-instance state (each enemy has its own direction). A
   `Local<>` resource would force global state — wrong semantics.

2. **Manual positioning in boundary tests**: Bevy 0.19's
   `TimeUpdateStrategy::ManualDuration` is one-shot and warm-up
   makes multi-frame advances flaky. To test boundary-flip logic
   in isolation, manually position the entity at `patrol_range + 1`
   and let the system flip in a single frame.

3. **Loader-level heuristic** for runtime-only components (Pickup,
   EnemyDirection) is the clean compromise when the sample has no
   schema for them. Both could be migrated to schema fields later
   without breaking the runtime.

4. **Cycle throughput**: G1-step5 took ~10 minutes following the
   mirror-G1-step4 recipe. The pattern is now stable enough that
   each new system + 3-4 tests + release + archive is mechanical.
