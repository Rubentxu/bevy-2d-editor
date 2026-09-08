# G1-step4 — Jump mechanic (release-receipt)

**Cycle**: `p-28fce7028ac3c497/g1-step4-bj5-jump`
**Sequence**: 246 (2026-09-08)
**Tag**: v0.109.4
**Phase**: release

## Release summary

| Item | Value |
|------|-------|
| Cycle name | g1-step4-bj5-jump |
| Path | A-lite |
| Phase at release | verify → release transition |
| Tag | v0.109.4 |
| Code-commit SHA | `d2fbff6` |
| Trunk SHA (HEAD) | `d2fbff6` |
| Origin/main SHA | `d2fbff6` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |
| Diff vs. v0.109.3 | +896 / -0 across 9 files |

## Files in code release

```
crates/examples-bevy-harness/src/jump.rs        NEW (83 lines)
crates/examples-bevy-harness/src/lib.rs         +3 / -1 lines
crates/examples-bevy-harness/tests/jump.rs      NEW (188 lines)
```

(plus 6 SDDK artifact docs in `docs/sddk/g1-step4-bj5-jump/`.)

## Test posture

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

Net change vs. v0.109.3: +3 Bevy gameplay tests.

## Workspace

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

```
$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Static analysis

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Public API

- New `pub struct JumpPlugin` (Bevy 0.19 `Plugin` impl).
- New `pub struct JumpState` (Resource, `Default`-derived).
- New `pub const JUMP_FRAME_DT: f32 = 0.1`.
- New `pub fn jump_system`.

No changes to existing public types.

## Cycle closure

| Phase | Sequence |
|-------|----------|
| cycle.start | 237 |
| phase.explore.complete | 238 |
| phase.specify.complete | 240 |
| phase.design.complete.a-lite | 242 |
| phase.build.complete | 244 |
| phase.verify.complete.a-lite | 246 |

## Out-of-scope carried forward

- BJ-3 (UI-creation Playwright test, P2) — still open.
- BJ-5 remaining: enemy patrol, contact-death (logic-graph runtime
  bridge needed).

## Lessons

1. **Rising-edge via resource**: Bevy 0.19's `keyboard_input_system`
   wipes `just_pressed` in PreUpdate. Setting `pressed()` directly
   in tests doesn't survive that wipe. Tracking the previous frame's
   pressed-state in a `Resource` (`JumpState::space_was_pressed`) is
   the robust workaround — version-agnostic and event-pipeline-free.

2. **Constant `dt = 0.1`**: choosing a fixed impulse duration that
   matches the test's `TimeUpdateStrategy::ManualDuration(0.1)` lets
   the test assert exact deltas without depending on Bevy's first-
   frame `Time<Real>::last_update` edge case. Document the choice
   inline.
