# G1-step5 — Enemy patrol (verify-report)

**Cycle**: `p-28fce7028ac3c497/g1-step5-bj5-enemy-patrol`
**Sequence**: 258 (2026-09-08)
**Path**: A-lite
**Phase**: verify

## Test posture

### Bevy runtime tests (examples-bevy-harness, `--ignored`)

```
running 4 tests (enemy_patrol)
test enemy_patrols_right_initially ... ok
test enemy_reverses_at_left_boundary ... ok
test enemy_reverses_at_boundary ... ok
test enemy_unaffected_by_player_movement ... ok
test result: ok. 4 passed; 0 failed

running 3 tests (jump)
test jump_only_affects_player ... ok
test player_does_not_jump_without_input ... ok
test player_jumps_on_space_press ... ok
test result: ok. 3 passed; 0 failed

running 1 tests (load_sample)
test sample_round_trips_into_bevy_world ... ok
test result: ok. 1 passed; 0 failed

running 3 tests (pickup_collision)
test pickup_unchanged_when_player_far ... ok
test pickup_despawns_when_player_overlaps ... ok
test pickup_collision_only_affects_pickup_marker ... ok
test result: ok. 3 passed; 0 failed

running 4 tests (player_movement)
test player_stays_still_with_no_input ... ok
test player_translates_left_on_arrow_left ... ok
test player_translates_right_on_arrow_right ... ok
test enemy_is_unaffected_by_player_input ... ok
test result: ok. 4 passed; 0 failed
```

**Net Bevy runtime**: 15/15 pass (4 new + 11 prior).

### Workspace tests (editor-bevy)

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored
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
| AC-EP1.1 | `enemy_patrols_right_initially` passes — direction default +1.0, x delta = 6.0 | ✅ |
| AC-EP1.2 | `enemy_reverses_at_boundary` passes — direction flips to -1.0 at right boundary | ✅ |
| AC-EP1.3 | `enemy_reverses_at_left_boundary` passes — direction flips to +1.0 at left boundary | ✅ |
| AC-EP1.4 | `enemy_unaffected_by_player_movement` passes — enemy at x=6.0, player at x=20.0 | ✅ |
| AC-EP2 | 11 prior Bevy runtime tests still pass (no regression) | ✅ |
| AC-EP3 | 414/0/1 editor-bevy tests still pass (no regression) | ✅ |
| AC-EP4 | `bun run tools/archcheck/check.ts` green | ✅ |
| AC-EP5 | `cargo check --workspace --tests` clean | ✅ |

## Risk

Low. Mirrors G1-step2/3/4 exactly. The only novelty is per-entity
state via `EnemyDirection` component, which is a 12-line addition
to components.rs.

## Lessons

1. **Per-entity state via component** is the right shape when each
   instance has independent state (enemy direction). A `Local<>`
   resource would force global state — wrong for multiple enemies.

2. **Manual positioning in boundary tests**: when multi-frame time
   advances are flaky in Bevy 0.19 (because `ManualDuration` is
   one-shot), manually positioning the entity at `patrol_range + 1`
   bypasses the time-control complexity and tests the boundary-flip
   logic in isolation. This is a pragmatic test simplification.

3. **Loader-level heuristic** for runtime-only components (Pickup,
   EnemyDirection) is the clean compromise when the sample has no
   schema for them. Both components are attached by `editor.Name`
   and could be migrated to a schema later without breaking the
   runtime.

## Open issues

None.

## Ready for release

Yes. Code is committed locally; release phase will produce the
`release-receipt.md` and tag v0.109.5 at the archive commit.
