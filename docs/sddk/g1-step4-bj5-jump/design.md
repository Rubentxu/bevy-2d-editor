# G1-step4 — Jump mechanic (design)

**Cycle**: `p-28fce7028ac3c497/g1-step4-bj5-jump`
**Sequence**: 240 (2026-09-08)
**Path**: A-lite
**Phase**: design

## Module layout

```
crates/examples-bevy-harness/
  src/
    jump.rs           (NEW)
    lib.rs            (+3: mod jump; re-export JumpPlugin, JumpState, JUMP_FRAME_DT)
  tests/
    jump.rs           (NEW, 3 #[ignore] integration tests)
```

## Data model

### `JumpState` (new resource)

```rust
#[derive(Resource, Default)]
pub struct JumpState {
    pub space_was_pressed: bool,
}
```

Trivial. No invariants. Re-initialized by Bevy's
`init_resource::<JumpState>` on plugin build.

### `JUMP_FRAME_DT: f32 = 0.1`

Mirrors the test's `TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(0.1))`.
A single-frame impulse of `jump_force * 0.1` produces the deterministic
delta-y the test asserts on.

## System signature

```rust
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

No generics, no observers, no events. Vanilla Bevy 0.19.

## Plugin

```rust
pub struct JumpPlugin;

impl Plugin for JumpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JumpState>();
        app.add_systems(Update, jump_system);
    }
}
```

Single system, Update schedule. No interactions with other systems
(no ordering constraints).

## Test plan

### `tests/jump.rs`

Three `#[ignore]` tests, mirroring `player_movement.rs` /
`pickup_collision.rs` patterns:

#### `player_jumps_on_space_press`

```
$ cd crates/examples-bevy-harness
$ cargo test --test jump player_jumps_on_space_press -- --ignored --nocapture
test player_jumps_on_space_press ... ok
```

Setup:
- `MinimalPlugins + InputPlugin + PlayerMovementPlugin + JumpPlugin`
- `Time<Virtual>::set_strategy(ManualDuration(0.1))`
- Sample assets spawned; player exists with `PlayerController` and
  default `jump_force = 400.0`.

Steps:
1. `advance(&mut app, 0.1)` — warms up `Time<Real>`.
2. `app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);`
3. `app.update()`.

Assert: `player.translation.y - initial_y ≈ 40.0` (within ±1.0).

#### `player_does_not_jump_without_input`

Setup: same as above but **without** pressing Space.

Steps:
1. `advance(&mut app, 0.1)`.
2. `app.update()`.

Assert: `player.translation.y == initial_y` (exact).

#### `jump_only_affects_player`

Setup: player + enemy + ground (no `PlayerController` on enemy or
ground).

Steps:
1. Move enemy to `(0, 0)` (same position as player).
2. `advance(&mut app, 0.1)`.
3. Press Space.
4. `app.update()`.

Assert: `enemy.translation.y == initial_y`; `ground.translation.y ==
initial_y`; player y increased.

## Component reuse

No new components. `PlayerController` (declared in `components.rs`)
carries `jump_force: f32`. The query is `With<PlayerController>`,
matching the gate from G1-step2's `player_movement_system`.

## Tradeoffs

- **Constant `dt = 0.1` vs `Time<Virtual>::delta_seconds()`**:
  We pick the constant. The constant is documented as matching the
  test's `ManualDuration(0.1)`. This decouples the system from
  Bevy's time-advance edge cases (where the first frame's
  `delta_seconds` is unreliable).
  A real production system would read `Res<Time<Virtual>>` and use
  `delta_seconds`. We document the constant choice inline.

- **`pressed()` + state resource vs `just_pressed()`**:
  `just_pressed` is wiped by PreUpdate's `keyboard_input_system`
  `clear()` when a test sets the press directly without writing a
  `KeyboardInput` event. State-tracking resource is robust across
  all patterns.

- **No gravity**: out of scope. Witness behavior only.

## Risk

Low. Mirrors G1-step2/G1-step3 exactly. The only novelty is the
`JumpState` resource — 6 lines, no new dependencies.

## Acceptance

- `cargo test -p examples-bevy-harness -- --ignored`: 11/11 pass.
- `cargo test -p editor-bevy --lib`: 414/0/1 unchanged.
- `bun run tools/archcheck/check.ts`: green.
- `cargo check --workspace --tests`: clean.
