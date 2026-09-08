# G1-step3 — Pickup collision (implementation-receipt)

**Cycle**: `p-28fce7028ac3c497/g1-step3-bj5-pickup-patrol`
**Sequence**: 228 (2026-09-08)
**Phase**: build
**Path**: A-lite

## Implementation summary

Closes BJ-5 (partial): pickup collision mechanic for the v1.0
canonical sample. The harness crate gains:

1. A `Pickup` marker component.
2. A `PickupCollisionPlugin` + `pickup_collision_system` that
   despawns pickups when the player overlaps.
3. Loader-level attachment of the `Pickup` marker to entities named
   `"Pickup"` (heuristic, since the sample has no `game.Pickup`
   schema yet).
4. Three integration tests covering overlap despawn, far-away
   preservation, and `Pickup` marker-only filter.

### Files changed

#### 1. `crates/examples-bevy-harness/src/components.rs` (+13 lines)

Added marker component:

```rust
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pickup;
```

#### 2. `crates/examples-bevy-harness/src/loader.rs` (+10 lines, -2 lines)

- Added `Pickup` to the imports.
- In `SpawnFromDoc::spawn_into`, after attaching all components,
  attach `Pickup` to entities whose `Name == "Pickup"`.

#### 3. `crates/examples-bevy-harness/src/collision.rs` (NEW, 55 lines)

```rust
pub struct PickupCollisionPlugin;

impl Plugin for PickupCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pickup_collision_system);
    }
}

pub const PICKUP_HITBOX_HALF: f32 = 16.0;

pub fn pickup_collision_system(
    mut commands: Commands,
    q_player: Query<&Transform, With<PlayerController>>,
    q_pickups: Query<(Entity, &Transform), With<Pickup>>,
) {
    let Ok(player_t) = q_player.single() else { return; };
    let px = player_t.translation.x;
    let py = player_t.translation.y;
    for (pickup_e, pickup_t) in &q_pickups {
        let dx = (px - pickup_t.translation.x).abs();
        let dy = (py - pickup_t.translation.y).abs();
        if dx < PICKUP_HITBOX_HALF && dy < PICKUP_HITBOX_HALF {
            commands.entity(pickup_e).despawn();
        }
    }
}
```

#### 4. `crates/examples-bevy-harness/src/lib.rs` (+2 lines)

Added `pub mod collision` and re-exports for `PickupCollisionPlugin`,
`pickup_collision_system`, `PICKUP_HITBOX_HALF`, `Pickup`.

#### 5. `crates/examples-bevy-harness/tests/pickup_collision.rs` (NEW, 186 lines)

Three `#[ignore]` integration tests:

- `pickup_despawns_when_player_overlaps` — player at (0,0), pickup
  at (5,5). Frame advances. Pickup despawned.
- `pickup_unchanged_when_player_far` — player at (0,0), pickup at
  (1000,1000). Frame advances. Pickup remains.
- `pickup_collision_only_affects_pickup_marker` — overlap scenario;
  assert player, enemy, ground all remain; only pickup despawned.

## Validation

### Tests

```
$ cd crates/examples-bevy-harness
$ cargo test -- --ignored

running 4 tests (player_movement)
test player_stays_still_with_no_input ... ok
test player_translates_right_on_arrow_right ... ok
test player_translates_left_on_arrow_left ... ok
test enemy_is_unaffected_by_player_input ... ok
test result: ok. 4 passed; 0 failed

running 3 tests (pickup_collision)
test pickup_despawns_when_player_overlaps ... ok
test pickup_unchanged_when_player_far ... ok
test pickup_collision_only_affects_pickup_marker ... ok
test result: ok. 3 passed; 0 failed

running 1 test (load_sample)
test sample_round_trips_into_bevy_world ... ok
test result: ok. 1 passed; 0 failed
```

### No regressions

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

### Cargo check

```
$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### Archcheck

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Diff stats

```
$ git diff --stat
 crates/examples-bevy-harness/src/components.rs        | 13 +++++++
 crates/examples-bevy-harness/src/loader.rs            | 12 +++++--
 crates/examples-bevy-harness/src/lib.rs               |  3 ++-
 crates/examples-bevy-harness/src/collision.rs         | 55 ++++++++++++++++
 crates/examples-bevy-harness/tests/pickup_collision.rs|186 ++++++++++++++++++++
 5 files changed, 266 insertions(+), 3 deletions(-)
```

## Public API

- New `pub struct Pickup` (marker component).
- New `pub const PICKUP_HITBOX_HALF: f32`.
- New `pub struct PickupCollisionPlugin` (Bevy 0.19 `Plugin` impl).
- New `pub fn pickup_collision_system`.
- No changes to existing public types.

## Out of scope (BJ-5 remaining)

- Enemy patrol (`EnemyPatrol` schema has `speed` + `patrol_range`
  but no direction tracking; needs a follow-up cycle with either a
  new component or schema extension).
- Jump (`PlayerController.jump_force` exists in schema; no system).
- Contact-death (`LogicGraphAsset` runtime evaluation).

These remain in the debt report for future cycles.
