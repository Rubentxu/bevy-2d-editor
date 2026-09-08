# G1-step4 — Jump mechanic (verify-report)

**Cycle**: `p-28fce7028ac3c497/g1-step4-bj5-jump`
**Sequence**: 244 (2026-09-08)
**Path**: A-lite
**Phase**: verify

## Test posture

### Bevy runtime tests (examples-bevy-harness, `--ignored`)

```
running 3 tests (jump)
test player_jumps_on_space_press ... ok
test player_does_not_jump_without_input ... ok
test jump_only_affects_player ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 1 tests (load_sample)
test sample_round_trips_into_bevy_world ... ok
test result: ok. 1 passed; 0 failed; 0 ignored

running 3 tests (pickup_collision)
test pickup_despawns_when_player_overlaps ... ok
test pickup_unchanged_when_player_far ... ok
test pickup_collision_only_affects_pickup_marker ... ok
test result: ok. 3 passed; 0 failed; 0 ignored

running 4 tests (player_movement)
test enemy_is_unaffected_by_player_input ... ok
test player_translates_right_on_arrow_right ... ok
test player_translates_left_on_arrow_left ... ok
test player_stays_still_with_no_input ... ok
test result: ok. 4 passed; 0 failed; 0 ignored
```

**Net Bevy runtime**: 11/11 pass (3 new + 8 prior).

### Workspace tests (editor-bevy)

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

### Static analysis

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

### Cargo check

```
$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Acceptance criteria review

| AC | Description | Status |
|----|-------------|--------|
| AC-J1.1 | `player_jumps_on_space_press` passes — y delta = 40.0 | ✅ |
| AC-J1.2 | `player_does_not_jump_without_input` passes — y unchanged | ✅ |
| AC-J1.3 | `jump_only_affects_player` passes — enemy y unchanged | ✅ |
| AC-J2 | 8 prior Bevy runtime tests still pass (no regression) | ✅ |
| AC-J3 | 414/0/1 editor-bevy tests still pass (no regression) | ✅ |
| AC-J4 | `bun run tools/archcheck/check.ts` green | ✅ |
| AC-J5 | `cargo check --workspace --tests` clean | ✅ |

## Risk

Low. Mirrors G1-step2/G1-step3 exactly. The rising-edge detection
via `JumpState` resource is reusable for any future edge-triggered
input.

## Open issues

None.

## Ready for release

Yes. Code is committed locally; release phase will produce the
`release-receipt.md` and tag v0.109.4 at the archive commit.
