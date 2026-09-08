# G1-step2 — Player Movement System (design)

**Cycle**: `p-28fce7028ac3c497/g1-step2-player-movement`
**Sequence**: 212 (2026-09-08)
**Path**: A-lite
**Phase**: design

## Architecture

### Module layout

```
crates/examples-bevy-harness/src/
├── components.rs     (existing — Bevy component mirrors)
├── lib.rs            (existing — public API)
├── loader.rs         (existing — JSON → Bevy World)
└── movement.rs       (NEW — player_movement_system + PlayerMovementPlugin)
```

### Bevy 0.19 plugin pattern

```rust
// crates/examples-bevy-harness/src/movement.rs
use bevy::prelude::*;
use crate::components::PlayerController;

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_movement_system);
    }
}

pub fn player_movement_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut Transform, &PlayerController)>,
) {
    let mut dir = 0.0;
    if input.pressed(KeyCode::ArrowRight) { dir += 1.0; }
    if input.pressed(KeyCode::ArrowLeft) { dir -= 1.0; }
    if dir == 0.0 { return; }
    let dt = time.delta_secs();
    for (mut t, ctrl) in &mut q {
        t.translation.x += dir * ctrl.speed * dt;
    }
}
```

### Why a plugin wrapper

A-lite introduces Bevy's first plugin in this crate. Wrapping in a
`PlayerMovementPlugin` (vs. exporting the system function directly) sets
up the pattern for future cycles that add more systems (jump, enemy
patrol, pickup collision). The plugin is `pub` so consumers of the
harness can wire the schedule with one `app.add_plugins(PlayerMovementPlugin)`
call.

### Time resource

Bevy 0.19 `MinimalPlugins` registers `Time<Real>` and `Time<Virtual>`.
The system reads `Res<Time>` (defaults to `Time<Virtual>`). In the
integration test, we advance virtual time:

```rust
fn advance(app: &mut App, secs: f32) {
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .advance_by(Duration::from_secs_f32(secs));
    app.update();
}
```

`Duration` is `std::time::Duration`. `Time::advance_by` is a method on
the underlying generic.

### Input simulation

In tests, Bevy 0.19 supports:

```rust
let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
input.press(KeyCode::ArrowRight);
input.release(KeyCode::ArrowLeft);
```

This avoids depending on the rendering window or `InputPlugin`'s
mocking layer.

### Integration test structure

```rust
// crates/examples-bevy-harness/tests/player_movement.rs
use bevy::prelude::*;
use examples_bevy_harness::PlayerMovementPlugin;
use examples_bevy_harness::loader; // re-export
use std::time::Duration;

const PLAYER_SPEED: f32 = 200.0;

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(PlayerMovementPlugin);
    // spawn assets
    let assets = loader::load_sample_assets();
    examples_bevy_harness::spawn_all_assets(app.world_mut(), &assets);
    app
}

#[test]
fn test_player_translates_right_on_arrow_right() {
    let mut app = setup_app();
    let player = app.world_mut()
        .query_filtered::<Entity, With<PlayerController>>()
        .single(app.world());
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowRight);
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .advance_by(Duration::from_secs_f32(0.1));
    app.update();
    let t = app.world().entity(player).get::<Transform>().unwrap();
    let expected = PLAYER_SPEED * 0.1;
    assert!(
        (t.translation.x - expected).abs() < 0.5,
        "expected x ≈ {}, got {}", expected, t.translation.x
    );
}
```

(Filter and assertions follow the same pattern as
`tests/load_sample.rs`.)

## Schema-to-component contract

The system trusts `PlayerController.speed` from the JSON schema
(200.0). The harness's loader currently assigns the schema default
when the field is omitted; we keep that behaviour. This cycle does
NOT alter the schema, the loader, or `components.rs`.

## Risks

1. **Time API churn**: Bevy 0.19 has had `Time::advance_by` semantics
   shift across patch versions. Mitigation: the test asserts an
   approximate range `[expected - 0.5, expected + 0.5]` rather than
   equality.

2. **Filter on `With<PlayerController>`**: enemies carry
   `EnemyPatrol`, not `PlayerController`. The query filter guarantees
   only the player responds. The spec's AC-PM1.4 validates this.

3. **`MinimalPlugins` resource registration order**: Bevy 0.19 plugins
   register `Time<Virtual>` and `ButtonInput<KeyCode>` via
   `InputPlugin` (part of `MinimalPlugins`). Confirmed.

## Out-of-scope deferred to future cycles

- `jump_force` (PlayerController field) → future `g1-step3-jump`.
- Enemy patrol (EnemyPatrol.speed + patrol_range) → future
  `g1-step4-enemy-patrol`.
- Pickup collision (sensor.contact_enter + actuator.destroy_entity)
  → future `g1-step5-contact-death`.
- UI-creation Playwright test (BJ-3) → independent of Bevy runtime
  work; can run in parallel.
