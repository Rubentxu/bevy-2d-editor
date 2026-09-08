# G1-step4 — Jump mechanic (explore-report)

**Cycle**: `p-28fce7028ac3c497/g1-step4-bj5-jump`
**Sequence**: 237 (2026-09-08)
**Path**: A-lite
**Phase**: explore

## 1. Origin

Continuing BJ-5 closure. G1-step3 (v0.109.3) closed pickup collision.
This cycle closes the **jump mechanic** — pressing `KeyCode::Space`
applies an upward impulse to the player using the schema-declared
`jump_force` (default 400.0 px/s).

After this cycle, BJ-5 remaining:
- enemy patrol (needs schema extension or per-entity state)
- contact-death logic graph runtime (declarative → imperative bridge)

## 2. Hypothesis

A Bevy system that listens for `KeyCode::Space` and on the rising
edge sets `player.translation.y += jump_force * dt` (a single-frame
impulse). Without gravity, the player remains at the new height
forever — this is a witness, not a physics simulator.

## 3. Investigation

### `just_pressed` vs `pressed`

The previous systems (G1-step2 player movement) read `pressed()`
because the player can hold the key to keep moving. For jump, we
want edge detection — the impulse fires once on the rising edge
(key goes from released to pressed), not every frame the key is
held.

### Test-time gotcha: `clear()` on `just_pressed`

Bevy 0.19's `keyboard_input_system` runs in `PreUpdate` and calls
`ButtonInput::clear()` which zeroes `just_pressed` and
`just_released`. If a unit test calls `press()` directly (no
`KeyboardInput` event), the press lands in `pressed` but
`just_pressed` is wiped before the jump system runs in `Update`.

Two ways around this:

**Option A** — track state in a resource: read `pressed()` and
remember the previous frame's state. Rising edge = pressed now,
not pressed before.

```rust
#[derive(Resource, Default)]
pub struct JumpState {
    pub space_was_pressed: bool,
}

pub fn jump_system(
    input: Res<ButtonInput<KeyCode>>,
    mut jump_state: ResMut<JumpState>,
    mut q: Query<(&mut Transform, &PlayerController)>,
) {
    let pressed_now = input.pressed(KeyCode::Space);
    if pressed_now && !jump_state.space_was_pressed {
        for (mut t, ctrl) in &mut q {
            t.translation.y += ctrl.jump_force * 0.1;
        }
    }
    jump_state.space_was_pressed = pressed_now;
}
```

**Option B** — write a `KeyboardInput` event manually. The
PreUpdate `keyboard_input_system` reads the event and calls
`press()` *after* `clear()`, so `just_pressed` ends up set when
Update runs.

**Decision**: Option A. It's simpler (no event plumbing), doesn't
depend on `KeyboardInput`'s exact struct fields (Bevy 0.19 might
evolve them), and the `JumpState` resource is a reusable pattern
for any future edge-triggered input.

### Test pattern

```rust
fn press_space(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
}

#[test]
#[ignore]
fn player_jumps_on_space() {
    let mut app = setup_app();
    let player = find_named(&mut app, "Player");
    let y0 = app.world().get::<Transform>(player).unwrap().translation.y;

    // Warm up Time<Real>.
    advance(&mut app, 0.1);

    // Press Space; update once; jump fires on the rising edge.
    press_space(&mut app);
    app.update();

    let y1 = app.world().get::<Transform>(player).unwrap().translation.y;
    let expected_delta = 400.0 * 0.1; // 40.0
    assert!(
        (y1 - (y0 + expected_delta)).abs() < 1.0,
        "expected y delta ≈ {}, got {}",
        expected_delta,
        y1 - y0
    );
}
```

### Component reuse

No new components needed. `PlayerController` already carries
`jump_force`. The query is gated by `With<PlayerController>`, so the
enemy (carrying `EnemyPatrol`) is unaffected — same pattern as
`player_movement_system` (G1-step2).

## 4. Decision

- **New module**: `crates/examples-bevy-harness/src/jump.rs`.
- **New resource**: `JumpState` (tracks `space_was_pressed` for
  edge detection).
- **Reuse**: `PlayerController.jump_force` from schema (default
  400.0).
- **Edge trigger**: rising-edge detection via `pressed()` +
  `JumpState.space_was_pressed`.

## 5. Validation status

Pending implementation. Expected:

- 3/3 jump tests pass under `--ignored`.
- 8/8 prior Bevy runtime tests still pass.
- 414/0/1 editor-bevy tests still pass.
- archcheck + cargo check green.

## 6. Open questions

None. The rising-edge pattern is reusable and Bevy 0.19 version-
agnostic (doesn't depend on `KeyboardInput` event shape).
