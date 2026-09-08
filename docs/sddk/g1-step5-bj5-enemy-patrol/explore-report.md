# G1-step5 — Enemy patrol (explore-report)

**Cycle**: `p-28fce7028ac3c497/g1-step5-bj5-enemy-patrol`
**Sequence**: 251 (2026-09-08)
**Path**: A-lite
**Phase**: explore

## 1. Origin

Continuing BJ-5 closure. G1-step4 (v0.109.4) closed the jump
mechanic. This cycle closes the **enemy patrol** — the `Enemy`
entity oscillates between `-patrol_range` and `+patrol_range` along
the x axis at the speed declared by its `EnemyPatrol` component.

After this cycle, BJ-5 remaining:
- contact-death logic graph runtime (declarative → imperative bridge)

## 2. Hypothesis

A Bevy system that reads `&EnemyPatrol` (speed, patrol_range) and
moves the entity along x at `speed * dt`. When the entity crosses
`±patrol_range`, the direction reverses.

### State tracking

Each enemy needs per-entity state (current direction). Two options:

**Option A** — `Local<EnemyDirection>` as a component (struct with
`{ direction: f32 }`). Pro: explicit state. Con: new component +
loader wiring.

**Option B** — derive direction from current position. If `x >= patrol_range`,
direction is negative; if `x <= -patrol_range`, direction is positive.
Otherwise, keep current direction. Pro: no new component. Con: needs
the direction stored somewhere (component or global resource).

**Option C** — derive direction from current position without storing
it. If `x >= patrol_range`, direction = -1; if `x <= -patrol_range`,
direction = +1; otherwise direction = previous frame's value. But
"previous frame's value" still needs storage.

**Decision**: Option A. Add `EnemyDirection(pub f32)` component
(signed scalar: +1.0 or -1.0). The loader attaches it to entities
named `Enemy`. Default `+1.0` (start moving right).

This mirrors the `Pickup` marker pattern from G1-step3: schema-less
heuristic via `editor.Name`.

### Boundary detection

```rust
if t.translation.x >= patrol_range {
    dir.x = -1.0;
} else if t.translation.x <= -patrol_range {
    dir.x = 1.0;
}
t.translation.x += dir.x * ctrl.speed * dt;
```

This is a "reflective" boundary: the moment the entity reaches the
edge, direction flips. The position doesn't overshoot the boundary
because dt is small (0.1) and the entity starts at x=0.

Actually wait — the boundary check happens BEFORE the move, so on
the frame the entity is at x=patrol_range, direction flips to -1,
then the move adds `-speed * dt` (negative), so the entity goes
back. The boundary check is exclusive: `>= patrol_range` flips.

To avoid drift past boundary over many frames, the condition should
use strict `>=` (not `>`), which I have. After the boundary flip,
the entity moves back into the interior.

## 3. Investigation

### Component reuse

- `EnemyPatrol { speed: f32, patrol_range: f32 }` — already exists
  in `components.rs` (G1 step 1).
- `EnemyDirection(pub f32)` — NEW component, attached by loader
  on entities named `Enemy`.

### Loader wiring

In `crates/examples-bevy-harness/src/loader.rs`, the loader
currently attaches `Pickup` to entities with `Name == "Pickup"`. We
add a similar heuristic for `EnemyDirection` on entities with
`Name == "Enemy"`, with default direction `+1.0`.

### Test pattern

Same shape as `player_movement.rs`, `pickup_collision.rs`,
`jump.rs`:
- `MinimalPlugins + InputPlugin + EnemyPatrolPlugin`
- `Time<Virtual>::set_strategy(ManualDuration(0.1))`
- Sample assets spawned; enemy at `(0, 0)` with `EnemyPatrol`
  defaults (`speed = 60.0`, `patrol_range = 150.0`).

After 1 frame: `enemy.x = 60.0 * 0.1 = 6.0`.
After enough frames to reach `patrol_range = 150.0`: 150.0 / 6.0 = 25
frames. Then direction flips.

### Three tests

1. `enemy_patrols_right_initially` — first frame moves enemy to
   `x ≈ 6.0`.
2. `enemy_reverses_at_boundary` — advance enough frames to reach
   `patrol_range`, then advance one more frame; the entity moves
   back toward x=0.
3. `enemy_unaffected_by_player_input` — press ArrowRight (a player
   input) and confirm enemy x is still 0 (not affected by player
   movement system).

Actually, test 3 is tricky because if I add `PlayerMovementPlugin`,
the player also moves. Let me redesign:

3. `enemy_unaffected_by_player_movement` — app with
   `PlayerMovementPlugin + EnemyPatrolPlugin`. Player at x=0 moves
   right (x=20), enemy at x=0 moves via patrol (x=6). Confirm
   enemy.x ≈ 6.0 (not 20.0 or 26.0).

This proves the `With<EnemyPatrol>` filter on `player_movement_system`
plus the `With<EnemyPatrol>` filter on `enemy_patrol_system`.

## 4. Decision

- **New module**: `crates/examples-bevy-harness/src/enemy_patrol.rs`.
- **New component**: `EnemyDirection(pub f32)` in `components.rs`.
- **Loader wiring**: attach `EnemyDirection(1.0)` to entities named
  `Enemy` (heuristic).
- **System**: `enemy_patrol_system` queries `(&mut Transform,
  &EnemyPatrol, &EnemyDirection)` with `With<EnemyPatrol>` and
  reverses at boundaries.
- **Plugin**: `EnemyPatrolPlugin` wires the system into Update.

## 5. Validation status

Pending implementation. Expected:

- 3/3 enemy patrol tests pass under `--ignored`.
- 11 prior Bevy runtime tests still pass.
- 414/0/1 editor-bevy tests still pass.
- archcheck + cargo check green.

## 6. Open questions

None. The boundary-reflective patrol pattern is simpler than a
patrol with explicit "waypoints" or "patrol state machine" — we
just track the current direction and bounce at edges.
