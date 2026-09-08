# G1-step2 — Player Movement System (spec)

**Cycle**: `p-28fce7028ac3c497/g1-step2-player-movement`
**Sequence**: 210 (2026-09-08)
**Path**: A-lite
**Phase**: specify

## Acceptance criteria

### AC-PM1.1 — Player translates right when ArrowRight pressed

```
$ cargo test -p examples-bevy-harness --test player_movement -- --nocapture

test player_movement::test_player_translates_right_on_arrow_right ... ok
```

**Given**: A Bevy `App` is built with `MinimalPlugins` +
`PlayerMovementPlugin`, and the player entity (carrying
`PlayerController { speed: 200.0, .. }`) is spawned via
`spawn_all_assets`.
**When**: The test simulates a `KeyCode::ArrowRight` press, then
advances the schedule by one frame (`Time<Virtual>::advance_by(0.1 s)`).
**Then**: The player entity's `Transform.translation.x` is in
`[19.5, 20.5]` (allowing for one frame of motion at speed 200).

### AC-PM1.2 — Player stays still when no input

**Given**: Same setup as AC-PM1.1 with no key pressed.
**When**: The schedule advances by one frame.
**Then**: `Transform.translation.x` stays at `0.0`.

### AC-PM1.3 — Player reverses on ArrowLeft

**Given**: Player spawned, frame 0 baseline.
**When**: `KeyCode::ArrowLeft` pressed, frame advances 0.1 s.
**Then**: `Transform.translation.x ≈ -20.0`.

### AC-PM1.4 — System is gated by `With<PlayerController>`

**Given**: An entity without `PlayerController` (e.g. `Enemy` with
`EnemyPatrol`) and the player entity.
**When**: `ArrowRight` pressed, frame advances 0.1 s.
**Then**: Only the player entity translates; enemy stays at x=0.

### AC-PM1.5 — No regressions in existing test suite

```
$ cargo test --workspace
```

All previously-passing tests still pass (414 in editor-bevy, 1 in
examples-bevy-harness load_sample, plus the new player_movement test).

### AC-PM1.6 — Workspace check green

```
$ cargo check --workspace
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### AC-PM1.7 — archcheck green

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Behavior

The system runs in Bevy's `Update` schedule and queries
`(&mut Transform, With<PlayerController>)`. Speed is read from the
component instance so the schema-declared `200.0` is the source of
truth (no magic number in the system).

```rust
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

## Out of scope

- Jump mechanic (`jump_force` field exists in schema but not wired).
- Enemy patrol (`EnemyPatrol` component, `speed` + `patrol_range`).
- Pickup collision (`Visible(false)` toggle on contact).
- Contact-death logic graph execution (the `LogicGraphAsset` is
  declarative; runtime evaluation of the graph is a separate concern).
- WASM-side playback (the harness runs as a standalone Bevy app, not
  in the browser).

## Public API

- New module `pub mod movement` in `crates/examples-bevy-harness/src/lib.rs`.
- New `pub struct PlayerMovementPlugin` implementing `Plugin`.
- No changes to existing public types.
