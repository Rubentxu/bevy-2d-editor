# G1-step5 — Enemy patrol (proposal)

**Cycle**: `p-28fce7028ac3c497/g1-step5-bj5-enemy-patrol`
**Sequence**: 252 (2026-09-08)
**Path**: A-lite
**Phase**: specify (proposal)

## Why

BJ-5 partial has 1 sub-deliverable remaining (contact-death). Before
that, the harness should have **enemy patrol** — the `Enemy` entity
should oscillate horizontally, proving the schema-declared
`game.EnemyPatrol.speed` and `patrol_range` are wired into the
runtime.

After this cycle, BJ-5 remaining:
- contact-death logic graph runtime (declarative → imperative bridge)

## What

A `EnemyPatrolPlugin` + `enemy_patrol_system` in
`crates/examples-bevy-harness/src/enemy_patrol.rs` that:

1. Reads `Query<(&mut Transform, &EnemyPatrol, &EnemyDirection)>`.
2. For each enemy:
   - If `translation.x >= patrol_range`, set `direction = -1.0`.
   - If `translation.x <= -patrol_range`, set `direction = +1.0`.
   - `translation.x += direction * speed * dt`.

### New component

```rust
// crates/examples-bevy-harness/src/components.rs
#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub struct EnemyDirection(pub f32);

impl Default for EnemyDirection {
    fn default() -> Self {
        Self(1.0) // start moving right
    }
}
```

### Loader wiring

```rust
// crates/examples-bevy-harness/src/loader.rs
// Attach EnemyDirection to entities whose Name == "Enemy".
// (Mirrors the Pickup heuristic from G1-step3.)
```

### Wire format

```rust
// crates/examples-bevy-harness/src/enemy_patrol.rs
use bevy::prelude::*;
use crate::components::{EnemyDirection, EnemyPatrol};

pub struct EnemyPatrolPlugin;

impl Plugin for EnemyPatrolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, enemy_patrol_system);
    }
}

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

### Tests

```rust
// crates/examples-bevy-harness/tests/enemy_patrol.rs

#[test]
#[ignore]
fn enemy_patrols_right_initially() { ... }

#[test]
#[ignore]
fn enemy_reverses_at_boundary() { ... }

#[test]
#[ignore]
fn enemy_unaffected_by_player_movement() { ... }
```

## Approach

A-lite. Adds a new Bevy system + plugin (architectural shade), one
new component (`EnemyDirection`), and a loader heuristic. Same
shape as G1-step2/3/4.

## Out of scope

- Vertical patrol (out-of-axis oscillation).
- Random patrol / patrol state machine.
- Acceleration / deceleration at boundary.
- Contact-death (separate cycle).

## Risk

Low. Mirrors G1-step2/3/4 exactly. The only novelty is per-entity
state via `EnemyDirection` component.

## Acceptance

- `cargo test -p examples-bevy-harness -- --ignored` shows 14/14
  pass (3 new + 11 prior).
- `cargo test -p editor-bevy --lib` shows 414/0/1.
- `bun run tools/archcheck/check.ts` is green.
- `cargo check` is clean.
