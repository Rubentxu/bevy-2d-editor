# G1-step5 handoff — Enemy patrol (BJ-5 partial)

**Cycle**: `p-28fce7028ac3c497/g1-step5-bj5-enemy-patrol`
**Tag**: v0.109.5 (commit `7006dc7`)
**Closed at**: 2026-09-08, sequence 251→264

## TL;DR

A-lite cycle closing **BJ-5 partial** — the enemy patrol mechanic
for the v1.0 canonical sample. The harness crate gains:

- An `EnemyDirection(pub f32)` component (per-entity state, default
  `+1.0`).
- An `EnemyPatrolPlugin` + `enemy_patrol_system` that oscillates the
  enemy horizontally between `-patrol_range` and `+patrol_range` at
  the schema-declared speed.
- A loader-level heuristic attaching `EnemyDirection::default()` to
  entities named `Enemy`.
- Four integration tests covering initial patrol, both boundary
  flips, and player-vs-enemy filter independence.

5 files / +419 / -1. 15 Bevy runtime tests now pass deterministically
(4 new + 4 player_movement + 3 pickup_collision + 3 jump + 1 load_sample).

## What shipped

### Code (single commit `7006dc7`)

- `crates/examples-bevy-harness/src/components.rs` (+18):
  `EnemyDirection` component + `Default` impl.
- `crates/examples-bevy-harness/src/enemy_patrol.rs` (NEW, 71):
  `EnemyPatrolPlugin` + `enemy_patrol_system`.
- `crates/examples-bevy-harness/src/lib.rs` (+3 / -1): module
  declaration + re-export.
- `crates/examples-bevy-harness/src/loader.rs` (+7 / -1):
  `EnemyDirection::default()` heuristic for entities named `Enemy`.
- `crates/examples-bevy-harness/tests/enemy_patrol.rs` (NEW, 224):
  4 `#[ignore]` integration tests.

## Cycle flow

| Phase | Sequence | Status |
|-------|----------|--------|
| cycle.start | 251 | OPEN/explore |
| phase.explore.complete | 252 | OPEN/specify |
| phase.specify.complete | 254 | OPEN/design |
| phase.design.complete.a-lite | 256 | OPEN/build |
| phase.build.complete | 258 | OPEN/verify |
| phase.verify.complete.a-lite | 260 | RELEASE_PENDING |
| release.complete | 262 | RELEASED |
| archive.complete | 264 | CLOSED |

## Validation evidence

```
$ cd crates/examples-bevy-harness
$ cargo test -- --ignored

running 4 tests (enemy_patrol)
test enemy_patrols_right_initially ... ok
test enemy_reverses_at_boundary ... ok
test enemy_reverses_at_left_boundary ... ok
test enemy_unaffected_by_player_movement ... ok
test result: ok. 4 passed; 0 failed

(plus 3 jump + 3 pickup_collision + 4 player_movement + 1 load_sample
 = 15 Bevy runtime tests)
```

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored
```

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Lessons learned

1. **Per-entity state via component** is the right shape for
   multi-instance state. A `Local<>` resource would force global
   state — wrong for multiple enemies.

2. **Manual positioning in boundary tests** is a pragmatic shortcut
   when Bevy 0.19's `TimeUpdateStrategy::ManualDuration` makes
   multi-frame advances flaky. Position the entity at
   `patrol_range + 1` and let the system flip in a single frame.

3. **Loader-level heuristic** continues to work cleanly for
   runtime-only components (`Pickup`, `EnemyDirection`).

## G1 status

| Sub-deliverable | Status | Cycle | Tag |
|-----------------|--------|-------|-----|
| BJ-1 test_helpers | ✅ | block-j-bj1-bj2-cleanup | v0.109.1 |
| BJ-2 archcheck | ✅ | block-j-bj1-bj2-cleanup | v0.109.1 |
| BJ-3 UI-creation Playwright | ❌ | (deferred) | — |
| BJ-4 player movement | ✅ | g1-step2-player-movement | v0.109.2 |
| BJ-5 pickup collision | ✅ | g1-step3-bj5-pickup-patrol | v0.109.3 |
| BJ-5 jump | ✅ | g1-step4-bj5-jump | v0.109.4 |
| BJ-5 enemy patrol | ✅ | g1-step5-bj5-enemy-patrol | v0.109.5 |
| BJ-5 contact-death | ❌ | (deferred — logic-graph runtime) | — |

G1 has 6/8 sub-deliverables closed. BJ-3 and BJ-5 contact-death
remain.

## Carry-forward

- **BJ-3**: UI-creation Playwright test (P2) — requires Playwright
  orchestration.
- **BJ-5 contact-death**: requires declarative → imperative bridge
  for the logic-graph runtime (out of scope for v0.109.x).
