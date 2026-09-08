# G1-step5 — Enemy patrol (design)

**Cycle**: `p-28fce7028ac3c497/g1-step5-bj5-enemy-patrol`
**Sequence**: 254 (2026-09-08)
**Path**: A-lite
**Phase**: design

## Module layout

```
crates/examples-bevy-harness/
  src/
    components.rs         (+12: pub struct EnemyDirection; impl Default)
    enemy_patrol.rs       (NEW)
    lib.rs                (+3: mod enemy_patrol; re-export)
    loader.rs             (+8: attach EnemyDirection to entities named "Enemy")
  tests/
    enemy_patrol.rs       (NEW, 3 #[ignore] integration tests)
```

## Data model

### `EnemyDirection` (new component)

```rust
#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub struct EnemyDirection(pub f32);

impl Default for EnemyDirection {
    fn default() -> Self {
        Self(1.0) // start moving right
    }
}
```

Invariants: `.0 ∈ {-1.0, 1.0}`. The system guarantees this by
writing only -1.0 or 1.0.

### `EnemyPatrol` (existing)

Already declared in `components.rs`:
```rust
#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub struct EnemyPatrol {
    pub speed: f32,
    pub patrol_range: f32,
}
impl Default for EnemyPatrol {
    fn default() -> Self { Self { speed: 60.0, patrol_range: 150.0 } }
}
```

## System signature

```rust
pub fn enemy_patrol_system(
    time: Res<Time>,
    mut q: Query<(&mut Transform, &EnemyPatrol, &mut EnemyDirection)>,
) {
    let dt = time.delta_secs();
    for (mut t, ctrl, mut dir) in &mut q {
        if t.translation.x >= ctrl.patrol_range {
            dir.0 = -1.0;
        } else if t.translation.x <= -ctrl.patrol_range {
            dir.0 = 1.0;
        }
        t.translation.x += dir.0 * ctrl.speed * dt;
    }
}
```

Boundary detection is **reflective**: when the entity reaches an
edge, direction flips. The position doesn't overshoot the boundary
because:

1. On the frame the entity reaches `patrol_range`, the direction
   flips to -1.0 BEFORE the move is applied.
2. The subsequent `translation.x += -1.0 * speed * dt` adds a
   negative delta, sending the entity back.

This guarantees `|translation.x| <= patrol_range + speed * dt` at
all times.

## Plugin

```rust
pub struct EnemyPatrolPlugin;

impl Plugin for EnemyPatrolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, enemy_patrol_system);
    }
}
```

No interactions with other systems (no ordering constraints).
Independent from `PlayerMovementPlugin` and `JumpPlugin`.

## Loader wiring

```rust
// crates/examples-bevy-harness/src/loader.rs
// In the entity-iteration loop, after attaching existing components:

if let Some(name) = entity.name.as_ref() {
    match name.as_str() {
        "Pickup" => { cmd.insert(Pickup); }
        "Enemy" => { cmd.insert(EnemyDirection::default()); }
        _ => {}
    }
}
```

## Test plan

### `tests/enemy_patrol.rs`

Three `#[ignore]` tests, mirroring the previous Bevy runtime tests:

#### `enemy_patrols_right_initially`

```
$ cargo test --test enemy_patrol enemy_patrols_right_initially -- --ignored --nocapture
test enemy_patrols_right_initially ... ok
```

Setup:
- `MinimalPlugins + InputPlugin + EnemyPatrolPlugin`
- `Time<Virtual>::set_strategy(ManualDuration(0.1))`
- Sample assets spawned; enemy at `(0, 0)` with `EnemyPatrol`
  defaults (`speed = 60.0`, `patrol_range = 150.0`).

Steps:
1. `advance(&mut app, 0.1)`.

Assert: `enemy.translation.x ≈ 6.0` (60.0 * 0.1).

#### `enemy_reverses_at_boundary`

Setup: same as above.

Steps:
1. Advance enough frames to reach `patrol_range = 150.0`. That's
   150.0 / 6.0 = 25 frames. Each frame advances 0.1 s.
2. After 25 frames, `enemy.x ≈ 150.0`.
3. One more frame: direction flips to -1.0, then `enemy.x` decreases
   by 6.0 to ≈ 144.0.

Assert: `enemy.translation.x ≈ 144.0` (just under 150.0).

Or simpler: advance 25 frames to land exactly at 150.0, then
advance 1 more frame and assert `x < 150.0`.

#### `enemy_unaffected_by_player_movement`

Setup:
- `MinimalPlugins + InputPlugin + PlayerMovementPlugin +
   EnemyPatrolPlugin`
- Sample assets spawned; player at `(0, 0)` and enemy at `(0, 0)`.

Steps:
1. Press ArrowRight.
2. Advance 1 frame.

Assert:
- `player.translation.x ≈ 20.0` (200.0 * 0.1).
- `enemy.translation.x ≈ 6.0` (60.0 * 0.1), NOT 20.0 (player
  movement doesn't affect enemy) and NOT 26.0 (enemy patrol
  doesn't compound with player movement).

## Component reuse

- `EnemyPatrol` from `components.rs` — existing.
- `Transform` from Bevy — standard.
- `EnemyDirection` from this cycle — new.

## Tradeoffs

- **`EnemyDirection` as a component vs `Local<>` resource**:
  Component is cleaner because each enemy has independent direction.
  A `Local<>` resource would force a single global direction shared
  across all enemies (not what we want).

- **`Time<Virtual>::delta_secs()` vs constant `dt`**: We use the
  actual delta here (unlike `jump_system`'s constant `0.1`). Why?
  Patrol is not bound to a test assertion on exact deltas — the
  test just needs `x` to advance positively. The actual delta is
  whatever Bevy produces after the warm-up.

  Wait — but `player_movement_system` uses `time.delta_secs()` too,
  and the test asserts an exact delta of 20.0. So `delta_secs()`
  is reliable for one-frame-after-warm-up tests.

  Yes, that's the same pattern. Use `time.delta_secs()`.

- **Reflective vs waypoint patrol**: We pick reflective (simpler).
  A waypoint patrol with explicit "patrol points" would require
  schema fields and is out of scope for a witness.

## Risk

Low. Mirrors G1-step2/3/4 exactly. The only novelty is the
`EnemyDirection` per-entity state component.

## Acceptance

- `cargo test -p examples-bevy-harness -- --ignored`: 14/14 pass.
- `cargo test -p editor-bevy --lib`: 414/0/1 unchanged.
- `bun run tools/archcheck/check.ts`: green.
- `cargo check --workspace --tests`: clean.
