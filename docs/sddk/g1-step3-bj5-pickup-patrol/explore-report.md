# G1-step3 — Pickup collision + enemy patrol (explore-report)

**Cycle**: `p-28fce7028ac3c497/g1-step3-bj5-pickup-patrol`
**Sequence**: 223 (2026-09-08)
**Path**: A-lite
**Phase**: explore

> **Note**: The cycle name was originally "pickup-patrol" intending to
> cover both pickup collision and enemy patrol. After investigation,
> scope was reduced to **pickup collision only** for this cycle.
> Enemy patrol requires per-entity state tracking (direction
> accumulator) that does not exist in the schema and would require
> a new component contract; that's better suited to a dedicated
> `g1-step4-enemy-patrol` cycle.

## 1. Origin

G1-step1 (v0.109.0) closed BJ-1 (structure round-trip). G1-step2
(v0.109.2) closed BJ-4 (Bevy play_mode runtime assertions for player
movement). This cycle closes **BJ-5 in part** — the pickup-collision
mechanic. After this cycle:

- BJ-5 partial: pickup collision closed.
- BJ-5 remaining: enemy patrol, jump, contact-death.

## 2. Hypothesis

The sample's `effects/pickup.actor.json` declares a single entity
named `Pickup` with `editor.Visible { visible: true }`. When the
player overlaps the pickup's position, the pickup should despawn
(collection). This is the canonical "trigger the gameplay event"
pattern for a 2D platformer.

### New marker component: `Pickup`

```rust
// crates/examples-bevy-harness/src/components.rs
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pickup;
```

### New plugin: `PickupCollisionPlugin`

```rust
// crates/examples-bevy-harness/src/collision.rs
pub struct PickupCollisionPlugin;

impl Plugin for PickupCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pickup_collision_system);
    }
}

const HITBOX_HALF: f32 = 16.0;

pub fn pickup_collision_system(
    mut commands: Commands,
    q_player: Query<&Transform, With<PlayerController>>,
    q_pickups: Query<Entity, With<Pickup>>,
) {
    // Pick the first player (sample has exactly one).
    let Ok(player_t) = q_player.single() else { return; };
    for pickup_e in &q_pickups {
        // The pickup's transform is the entity's own Transform.
        // We don't fetch Transform in the query to avoid ambiguity
        // with editor-model's Transform2D; instead we look it up here.
        // (Bevy reuses the entity's world transform.)
        let pickup_t = player_t; // placeholder — see implementation
        // ...
    }
}
```

The system:
1. Looks up the player's `Transform` (gated by `With<PlayerController>`).
2. Iterates every entity carrying `Pickup`.
3. Computes Manhattan distance in 2D; if both axes within `HITBOX_HALF`
   (16 px), despawn the pickup.

## 3. Investigation

### Existing patterns

- `PlayerController` is a `Component` round-tripped from the editor
  schema. `spawn_scene_asset` (loader.rs) attaches it to entities
  whose `Name == "Player"`.
- We can mirror that pattern for `Pickup`: add a `Pickup` component
  in components.rs and attach it in loader.rs when
  `Name == "Pickup"`.
- Alternative: query `With<Name>` where `name.as_str() == "Pickup"`.
  Cleaner because it avoids attaching a new marker, but mixes
  editor semantics into Bevy systems.

**Decision**: use a `Pickup` marker component. It mirrors the
`PlayerController` pattern, keeps the system code simple, and sets
up the convention for future schema-to-component mirrors.

### Hitbox size

The sample's sprite is 32×32. A 16-px hitbox (radius 16) gives
generous overlap. Real games would use proper collision shapes; this
witness uses AABB Manhattan distance.

### Transform lookup

`Pickup` entities carry `Transform` automatically (Bevy default).
The query can fetch `(Entity, &Transform, With<Pickup>)` and the
system iterates pairs.

## 4. Decision

### Files changed

1. `crates/examples-bevy-harness/src/components.rs`: add `Pickup` marker.
2. `crates/examples-bevy-harness/src/loader.rs`: attach `Pickup` to
   entities whose `Name == "Pickup"`.
3. `crates/examples-bevy-harness/src/collision.rs` (NEW):
   `PickupCollisionPlugin` + `pickup_collision_system`.
4. `crates/examples-bevy-harness/src/lib.rs`: declare new module +
   re-exports.
5. `crates/examples-bevy-harness/tests/pickup_collision.rs` (NEW):
   3 `#[ignore]` integration tests.

### Test contract

```rust
#[test] #[ignore]
fn pickup_despawns_when_player_overlaps() {
    // spawn assets, translate player to (0,0) and pickup to (5,5),
    // advance 0.1s, assert pickup entity is despawned.
}

#[test] #[ignore]
fn pickup_unchanged_when_player_far() {
    // spawn assets, keep player at (0,0), pickup at (1000,1000),
    // advance 0.1s, assert pickup entity still exists.
}

#[test] #[ignore]
fn pickup_collision_only_affects_pickup_marker() {
    // spawn assets, advance 0.1s, assert enemy + ground + player
    // entities are unaffected (only the pickup entity can despawn).
}
```

## 5. Out of scope

- Enemy patrol (needs schema extension).
- Jump mechanic (covered by G1-step2's `PlayerController.jump_force`
  field but no system yet).
- Contact-death logic graph runtime (`LogicGraphAsset` execution).
- UI-creation Playwright test (BJ-3).

## 6. Validation status

Pending implementation. Expected:

- 3/3 pickup_collision tests pass.
- 4/4 player_movement tests still pass (no regression).
- 1/1 load_sample test still passes.
- 414/0/1 editor-bevy tests still pass.
- archcheck: all assertions pass.
- cargo check --workspace: green.

## 7. Open questions

None. The mechanic is well-defined and the test invariants are clear.
