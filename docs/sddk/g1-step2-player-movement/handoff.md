# G1-step2 handoff — Player Movement System

**Cycle**: `p-28fce7028ac3c497/g1-step2-player-movement`
**Tag**: v0.109.2 (commit `98323a4`)
**Closed at**: 2026-09-08, sequence 209→222

## TL;DR

A-lite cycle closing BJ-4 (Bevy play_mode runtime state assertions)
from the G1 debt ledger. New `PlayerMovementPlugin` +
`player_movement_system` + 4 integration tests prove that the player
entity spawned from `examples/platformer-minimal/` translates in
response to arrow-key input. 3 files, +279 lines, no public API
breakage to existing types.

## What shipped

### Code (single commit `98323a4`)

- `crates/examples-bevy-harness/src/movement.rs` (NEW, 52 lines):
  `PlayerMovementPlugin` + `player_movement_system`. The system reads
  `Res<Time>`, `Res<ButtonInput<KeyCode>>`, and a query gated by
  `With<PlayerController>` (so the enemy entity carrying
  `EnemyPatrol` is unaffected).

- `crates/examples-bevy-harness/src/lib.rs` (+2 lines): declares
  `pub mod movement` and re-exports.

- `crates/examples-bevy-harness/tests/player_movement.rs` (NEW,
  225 lines): 4 `#[ignore]` integration tests using
  `MinimalPlugins + InputPlugin + PlayerMovementPlugin`. Includes
  test helpers (`setup_app`, `find_named`, `press`, `release`,
  `advance`) that handle Bevy 0.19 test-time gotchas.

## Cycle flow

| Phase | Sequence | Status |
|-------|----------|--------|
| cycle.start | 209 | OPEN/explore |
| phase.explore.complete | 210 | OPEN/specify |
| phase.specify.complete | 212 | OPEN/design |
| phase.design.complete.a-lite | 214 | OPEN/build |
| phase.build.complete | 216 | OPEN/verify |
| phase.verify.complete.a-lite | 218 | RELEASE_PENDING |
| release.complete | 220 | RELEASED |
| archive.complete | 222 | CLOSED |

## Validation evidence

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

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Lessons learned

1. **Bevy 0.19 `MinimalPlugins` does NOT include `InputPlugin`**. Tests
   that simulate keyboard input must add it explicitly:
   `app.add_plugins(InputPlugin);`. This is a behavioural change
   relative to Bevy 0.18 (where InputPlugin was part of
   MinimalPlugins).

2. **`Time<Real>::last_update` is `None` on the first frame**. The
   first `time_system` run sets it without applying a delta; the
   second run applies the configured `ManualDuration` duration as
   `delta_secs`. Tests must run two `app.update()` calls per simulated
   frame to make `delta_secs` deterministic.

3. **`TimeUpdateStrategy::ManualDuration(secs)`** is the deterministic
   mechanism for advancing virtual time in unit tests. The default
   `Automatic` strategy reads `Instant::now()`, which is
   non-deterministic and slows CI.

4. **Bevy 0.19 `TimeUpdateStrategy` does NOT implement `Clone`** —
   this blocks the pattern of "save strategy → run update → restore".
   We instead leave the strategy as `ManualDuration` and re-set it on
   every advance.

## Carry-forward

- BJ-3 (UI-creation Playwright test for sample, P2)
- BJ-5 (gameplay: jump, patrol, pickup, P3)

These remain open in `docs/sddk/g1-step2-player-movement/debt-report.json`.

## Reference

- Spec: `docs/sddk/g1-step2-player-movement/spec.md`
- Design: `docs/sddk/g1-step2-player-movement/design.md`
- Implementation receipt: `docs/sddk/g1-step2-player-movement/implementation-receipt.md`
- Verify report: `docs/sddk/g1-step2-player-movement/verify-report.md`
- Debt report: `docs/sddk/g1-step2-player-movement/debt-report.json`
- Release receipt: `docs/sddk/g1-step2-player-movement/release-receipt.md`
- Merge receipt: `docs/sddk/g1-step2-player-movement/merge-receipt.md`
- Archive manifest: `docs/sddk/g1-step2-player-movement/archive-manifest.md`
