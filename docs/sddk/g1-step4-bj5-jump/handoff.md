# G1-step4 handoff — Jump mechanic (BJ-5 partial)

**Cycle**: `p-28fce7028ac3c497/g1-step4-bj5-jump`
**Tag**: v0.109.4 (commit `d2fbff6`)
**Closed at**: 2026-09-08, sequence 237→250

## TL;DR

A-lite cycle closing **BJ-5 partial** — the jump mechanic for the
v1.0 canonical sample. The harness crate gains:

- A `JumpPlugin` + `jump_system` that listens for `KeyCode::Space`
  and applies a `jump_force * JUMP_FRAME_DT` upward impulse on the
  rising edge.
- A `JumpState` resource for rising-edge detection (works around
  Bevy 0.19's `keyboard_input_system.clear()` wiping `just_pressed`
  before Update runs).
- Three integration tests proving the press impulse, no-input
  stillness, and `With<PlayerController>` filter.

3 files / +271 / -1. 11 Bevy runtime tests now pass deterministically
(3 new + 4 player_movement + 3 pickup_collision + 1 load_sample).

## What shipped

### Code (single commit `d2fbff6`)

- `crates/examples-bevy-harness/src/jump.rs` (NEW, 83): `JumpState`
  resource, `JUMP_FRAME_DT = 0.1`, `JumpPlugin` (Bevy 0.19 `Plugin`
  impl), `jump_system`.
- `crates/examples-bevy-harness/src/lib.rs` (+3 / -1): module
  declaration and re-exports.
- `crates/examples-bevy-harness/tests/jump.rs` (NEW, 188): 3 `#[ignore]`
  integration tests.

## Cycle flow

| Phase | Sequence | Status |
|-------|----------|--------|
| cycle.start | 237 | OPEN/explore |
| phase.explore.complete | 238 | OPEN/specify |
| phase.specify.complete | 240 | OPEN/design |
| phase.design.complete.a-lite | 242 | OPEN/build |
| phase.build.complete | 244 | OPEN/verify |
| phase.verify.complete.a-lite | 246 | RELEASE_PENDING |
| release.complete | 248 | RELEASED |
| archive.complete | 250 | CLOSED |

## Validation evidence

```
$ cd crates/examples-bevy-harness
$ cargo test -- --ignored

running 3 tests (jump)
test player_jumps_on_space_press ... ok
test player_does_not_jump_without_input ... ok
test jump_only_affects_player ... ok
test result: ok. 3 passed; 0 failed

(plus 4 player_movement + 3 pickup_collision + 1 load_sample
 = 11 Bevy runtime tests)
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

1. **Rising-edge via resource**: Bevy 0.19's `keyboard_input_system`
   runs in PreUpdate and calls `ButtonInput::clear()` (wiping
   `just_pressed`) before Update. Setting `pressed()` directly in
   tests doesn't survive that wipe. The robust pattern is to track
   the previous frame's pressed-state in a `Resource` and detect the
   rising edge via `pressed() && !was_pressed`.

2. **Constant `dt = 0.1`**: choosing a fixed impulse duration that
   matches the test's `TimeUpdateStrategy::ManualDuration(0.1)` lets
   the test assert exact deltas without depending on Bevy's first-
   frame `Time<Real>::last_update` edge case. Document the choice
   inline.

3. **Single-commit cycle**: A-min/A-lite cycles that meet the gate
   criteria (architecture-consistent, implementation-complete,
   tests-pass, policy-compliant, debt-severity-assigned,
   debt-priority-assigned) skip the PR review. The cumulative
   evidence is in the cycle's artifact docs.

4. **Cycle throughput with mirroring**: G1-step4 took ~10 minutes
   because the structure mirrored G1-step2/G1-step3 exactly. Each
   new system + 3 tests + release + archive follows the same recipe.

## G1 status

| Sub-deliverable | Status | Cycle | Tag |
|-----------------|--------|-------|-----|
| BJ-1 test_helpers | ✅ | block-j-bj1-bj2-cleanup | v0.109.1 |
| BJ-2 archcheck | ✅ | block-j-bj1-bj2-cleanup | v0.109.1 |
| BJ-3 UI-creation Playwright | ❌ | (deferred) | — |
| BJ-4 player movement | ✅ | g1-step2-player-movement | v0.109.2 |
| BJ-5 pickup collision | ✅ | g1-step3-bj5-pickup-patrol | v0.109.3 |
| BJ-5 jump | ✅ | g1-step4-bj5-jump | v0.109.4 |
| BJ-5 enemy patrol | ❌ | (deferred — schema extension) | — |
| BJ-5 contact-death | ❌ | (deferred — logic-graph runtime) | — |

G1 has 5/8 sub-deliverables closed. BJ-3 and BJ-5 enemy/contact-death
remain.

## Carry-forward

- **BJ-3**: UI-creation Playwright test (P2) — requires Playwright
  orchestration.
- **BJ-5 enemy patrol**: needs `game.EnemyPatrol` schema fields
  (already declared) wired into a `EnemyPatrolPlugin` with
  per-entity timer state.
- **BJ-5 contact-death**: requires declarative → imperative bridge
  for the logic-graph runtime (out of scope for v0.109.x).
