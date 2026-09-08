# G1-step2 — Player Movement System (release-receipt)

**Cycle**: `p-28fce7028ac3c497/g1-step2-player-movement`
**Sequence**: 218 (2026-09-08)
**Tag**: v0.109.2
**Phase**: release

## Release summary

| Item | Value |
|------|-------|
| Cycle name | g1-step2-player-movement |
| Path | A-lite |
| Phase at release | verify → release transition |
| Tag | v0.109.2 |
| Code-commit SHA | `98323a4` |
| Trunk SHA (HEAD) | `98323a4` |
| Origin/main SHA | `98323a4` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |
| Diff vs. v0.109.1 | +279 / -0 across 3 files (1 lib, 1 module, 1 test) |

## Files in code release

```
crates/examples-bevy-harness/src/lib.rs    |  +2 lines
crates/examples-bevy-harness/src/movement.rs | NEW (52 lines)
crates/examples-bevy-harness/tests/player_movement.rs | NEW (225 lines)
```

## Test posture

```
$ cd crates/examples-bevy-harness
$ cargo test -- --ignored

running 4 tests
test player_stays_still_with_no_input ... ok
test player_translates_right_on_arrow_right ... ok
test player_translates_left_on_arrow_left ... ok
test enemy_is_unaffected_by_player_input ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Net change vs. v0.109.1: +4 Bevy runtime tests.

## Workspace

```
$ cargo test --workspace
```

All crates report `test result: ok. <n> passed; 0 failed`.

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

- New `pub mod movement` in `examples-bevy-harness`.
- New `pub struct PlayerMovementPlugin` (Bevy 0.19 `Plugin` impl).
- New `pub fn player_movement_system`.
- Re-exports from `lib.rs`.

No changes to existing public types (`components.rs`, `loader.rs`,
`spawn_all_assets`, `SpawnReport`).

## Cycle closure

| Phase | Sequence |
|-------|----------|
| cycle.start | 209 |
| phase.explore.complete | 210 |
| phase.specify.complete | 212 |
| phase.design.complete.a-lite | 214 |
| phase.build.complete | 216 |
| phase.verify.complete.a-lite | 218 |

(sequence 211, 213, 215, 217 are ledger internal events.)

## Out-of-scope carried forward

- BJ-3 (UI-creation Playwright test, P2) — still open.
- BJ-5 (gameplay: jump, patrol, pickup) — still open.

## Lessons

1. Bevy 0.19 `MinimalPlugins` does NOT include `InputPlugin`. Tests
   that simulate input must add it explicitly.

2. `Time<Real>::last_update` is `None` on first frame; the first
   `time_system` run sets it without applying a delta. Tests must run
   two `app.update()` calls before `delta_secs` is meaningful under
   `TimeUpdateStrategy::ManualDuration`.

3. `TimeUpdateStrategy::ManualDuration(secs)` is the deterministic
   mechanism for advancing virtual time in unit tests.
