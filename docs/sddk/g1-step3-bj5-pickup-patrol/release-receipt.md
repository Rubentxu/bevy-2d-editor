# G1-step3 — Pickup collision (release-receipt)

**Cycle**: `p-28fce7028ac3c497/g1-step3-bj5-pickup-patrol`
**Sequence**: 232 (2026-09-08)
**Tag**: v0.109.3
**Phase**: release

## Release summary

| Item | Value |
|------|-------|
| Cycle name | g1-step3-bj5-pickup-patrol |
| Path | A-lite |
| Phase at release | verify → release transition |
| Tag | v0.109.3 |
| Code-commit SHA | `c46effc` |
| Trunk SHA (HEAD) | `c46effc` |
| Origin/main SHA | `c46effc` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |
| Diff vs. v0.109.2 | +264 / -3 across 5 files |

## Files in code release

```
crates/examples-bevy-harness/src/components.rs        | +11 lines
crates/examples-bevy-harness/src/loader.rs            | +9 / -2 lines
crates/examples-bevy-harness/src/lib.rs               | +3 / -1 lines
crates/examples-bevy-harness/src/collision.rs         | NEW (55 lines)
crates/examples-bevy-harness/tests/pickup_collision.rs| NEW (186 lines)
```

## Test posture

```
$ cd crates/examples-bevy-harness
$ cargo test -- --ignored

running 3 tests (pickup_collision)
test pickup_despawns_when_player_overlaps ... ok
test pickup_unchanged_when_player_far ... ok
test pickup_collision_only_affects_pickup_marker ... ok
test result: ok. 3 passed; 0 failed

(plus 4 player_movement + 1 load_sample = 8 Bevy runtime tests)
```

Net change vs. v0.109.2: +3 Bevy gameplay tests.

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

- New `pub struct Pickup` (marker).
- New `pub const PICKUP_HITBOX_HALF: f32 = 16.0`.
- New `pub struct PickupCollisionPlugin` (Bevy 0.19 `Plugin` impl).
- New `pub fn pickup_collision_system`.

No changes to existing public types.

## Cycle closure

| Phase | Sequence |
|-------|----------|
| cycle.start | 223 |
| phase.explore.complete | 224 |
| phase.specify.complete | 226 |
| phase.design.complete.a-lite | 228 |
| phase.build.complete | 230 |
| phase.verify.complete.a-lite | 232 |

(sequence 225, 227, 229, 231 are ledger internal events.)

## Out-of-scope carried forward

- BJ-3 (UI-creation Playwright test, P2) — still open.
- BJ-5 partial: enemy patrol, jump, contact-death — still open.

## Lessons

1. **Loader-level heuristic for marker components**: when the sample
   has no schema for a category (e.g. `game.Pickup`), attaching a
   marker component by `editor.Name` is a clean compromise. The
   schema can be added later without breaking the runtime.

2. **`World::get_entity` returns `Result`, not `Option`**: Bevy 0.19
   uses `EntityNotSpawnedError`. Use `.is_ok()` not `.is_some()`.
