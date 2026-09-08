# G1-step5 — Enemy patrol (implementation-receipt)

**Cycle**: `p-28fce7028ac3c497/g1-step5-bj5-enemy-patrol`
**Sequence**: 256 (2026-09-08)
**Path**: A-lite
**Phase**: build

## Files created / modified

```
crates/examples-bevy-harness/src/components.rs    +18/-0   (EnemyDirection component)
crates/examples-bevy-harness/src/enemy_patrol.rs NEW 71    (plugin + system)
crates/examples-bevy-harness/src/lib.rs         +3/-1    (register + re-export)
crates/examples-bevy-harness/src/loader.rs       +7/-1    (heuristic for Enemy name)
crates/examples-bevy-harness/tests/enemy_patrol.rs NEW 224 (4 #[ignore] integration tests)
```

## Implementation

`crates/examples-bevy-harness/src/components.rs`:
- `pub struct EnemyDirection(pub f32)` — per-entity patrol direction
  state. Default `+1.0` (start moving right).

`crates/examples-bevy-harness/src/enemy_patrol.rs`:
- `pub struct EnemyPatrolPlugin` — Bevy 0.19 `Plugin` impl that
  adds `enemy_patrol_system` to `Update`.
- `pub fn enemy_patrol_system` — for each `EnemyPatrol` entity,
  reflect at `±patrol_range` and apply
  `translation.x += direction * speed * dt`.

`crates/examples-bevy-harness/src/lib.rs`:
- `pub mod enemy_patrol;`
- `pub use enemy_patrol::{EnemyPatrolPlugin, enemy_patrol_system};`
- `pub use components::EnemyDirection` (added to existing re-export).

`crates/examples-bevy-harness/src/loader.rs`:
- After existing heuristics, attach `EnemyDirection::default()` to
  entities whose `Name == "Enemy"`.

`crates/examples-bevy-harness/tests/enemy_patrol.rs`:
- `enemy_patrols_right_initially` (default +1.0 → x ≈ 6.0 after
  one frame)
- `enemy_reverses_at_boundary` (manually position past right
  boundary; verify direction flips to -1.0 and entity moves back)
- `enemy_reverses_at_left_boundary` (mirror of above)
- `enemy_unaffected_by_player_movement` (both plugins; player
  moves at 20.0, enemy moves at 6.0)

## Build verification

```
$ cd crates/examples-bevy-harness && cargo build
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 23.25s
```

## Test results

```
$ cd crates/examples-bevy-harness && cargo test --test enemy_patrol -- --ignored

running 4 tests
test enemy_patrols_right_initially ... ok
test enemy_reverses_at_left_boundary ... ok
test enemy_reverses_at_boundary ... ok
test enemy_unaffected_by_player_movement ... ok

test result: ok. 4 passed; 0 failed
```

## Workspace

```
$ cargo test -p examples-bevy-harness -- --ignored
4 (enemy_patrol) + 3 (jump) + 1 (load_sample) + 3 (pickup_collision) + 4 (player_movement) = 15/15
```

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored
```

```
$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Code quality

- Zero new dependencies.
- Zero changes to public types outside `examples-bevy-harness`.
- `EnemyDirection` component is `Default`-derived.
- `cargo clippy -p examples-bevy-harness --tests --no-deps`: no new
  warnings in enemy_patrol.rs or tests/enemy_patrol.rs.

## Acceptance

✓ `cargo test -p examples-bevy-harness -- --ignored`: 15/15 pass
✓ `cargo test -p editor-bevy --lib`: 414/0/1 unchanged
✓ `bun run tools/archcheck/check.ts`: green
✓ `cargo check --workspace --tests`: clean
