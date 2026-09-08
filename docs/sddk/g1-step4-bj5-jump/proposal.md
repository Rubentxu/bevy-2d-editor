# G1-step4 — Jump mechanic (proposal)

**Cycle**: `p-28fce7028ac3c497/g1-step4-bj5-jump`
**Sequence**: 238 (2026-09-08)
**Path**: A-lite
**Phase**: propose

## Why

BJ-5 partial is the second-to-last G1 sub-deliverable. After G1-step3
closed pickup collision, the runtime sample needs the jump mechanic
to prove the schema-declared `PlayerController.jump_force` is wired
into the harness.

## What

A `JumpPlugin` + `jump_system` in
`crates/examples-bevy-harness/src/jump.rs` that:

1. Reads `Res<ButtonInput<KeyCode>>` for `KeyCode::Space`.
2. Reads `ResMut<JumpState>` for the previous frame's press state
   (rising-edge detection).
3. On the rising edge, iterates `Query<(&mut Transform,
   &PlayerController)>` and applies an upward impulse
   `translation.y += jump_force * dt`.

Where `dt` is read from the previous frame's `Time<Virtual>::delta_seconds`
or a fixed constant matching the test's `ManualDuration` setup
(0.1 s). We pick the constant approach — `0.1` — to keep the test
deterministic without depending on Bevy's clock across the warm-up.

### Wire format

```rust
// crates/examples-bevy-harness/src/jump.rs
use bevy::prelude::*;
use crate::components::PlayerController;

#[derive(Resource, Default)]
pub struct JumpState {
    pub space_was_pressed: bool,
}

pub const JUMP_FRAME_DT: f32 = 0.1; // matches test ManualDuration

pub struct JumpPlugin;

impl Plugin for JumpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JumpState>();
        app.add_systems(Update, jump_system);
    }
}

pub fn jump_system(
    input: Res<ButtonInput<KeyCode>>,
    mut jump_state: ResMut<JumpState>,
    mut q: Query<(&mut Transform, &PlayerController)>,
) {
    let pressed_now = input.pressed(KeyCode::Space);
    if pressed_now && !jump_state.space_was_pressed {
        for (mut t, ctrl) in &mut q {
            t.translation.y += ctrl.jump_force * JUMP_FRAME_DT;
        }
    }
    jump_state.space_was_pressed = pressed_now;
}
```

```rust
// crates/examples-bevy-harness/src/lib.rs (additions)
pub mod jump;
pub use jump::{JumpPlugin, JumpState, JUMP_FRAME_DT};
```

### Tests

```rust
// crates/examples-bevy-harness/tests/jump.rs
// (helpers shared with player_movement.rs / pickup_collision.rs)

#[test]
#[ignore]
fn player_jumps_on_space_press() { ... }

#[test]
#[ignore]
fn player_does_not_jump_without_input() { ... }

#[test]
#[ignore]
fn jump_only_affects_player() { ... }
```

The third test moves an enemy (or a non-player entity) into the same
coordinates and confirms it does not move.

## Approach

A-lite because we add a new Bevy system + plugin (architectural
shade). It's still bounded — no schema changes, no new external
dependencies, no fork. Same shape as G1-step2 (player movement)
and G1-step3 (pickup collision).

## Out of scope

- Gravity (would require per-frame velocity integration, contact
  with ground entity).
- Variable jump height (release-of-Space cuts the impulse short).
- Mid-air double jump (no airborne state tracking).
- Enemy contact death (BJ-5 remaining — separate cycle).
- Enemy patrol (BJ-5 remaining — separate cycle).

## Risk

Low. The pattern mirrors G1-step2 exactly. The only novelty is the
rising-edge detection via `JumpState` resource, which is a 6-line
addition.

## Acceptance

- `cargo test -p examples-bevy-harness -- --ignored` shows 11/11
  pass (3 new + 8 prior).
- `cargo test -p editor-bevy --lib` shows 414/0/1.
- `bun run tools/archcheck/check.ts` is green.
- `cargo check` is clean.
