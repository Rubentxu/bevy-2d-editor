# G1-step2 — Player Movement System (implementation-receipt)

**Cycle**: `p-28fce7028ac3c497/g1-step2-player-movement`
**Sequence**: 214 (2026-09-08)
**Phase**: build
**Path**: A-lite

## Implementation summary

Implements BJ-4: Bevy play_mode runtime state assertions for the v1.0
canonical sample. The harness crate (`crates/examples-bevy-harness/`)
gains a Bevy `Plugin` that moves the player entity in response to arrow
key input, plus an integration test that asserts the runtime behaves
as expected.

### Files changed

#### 1. `crates/examples-bevy-harness/src/movement.rs` (NEW, 52 lines)

```rust
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
    if input.pressed(KeyCode::ArrowLeft)  { dir -= 1.0; }
    if dir == 0.0 { return; }
    let dt = time.delta_secs();
    for (mut t, ctrl) in &mut q {
        t.translation.x += dir * ctrl.speed * dt;
    }
}
```

Speed is read from the `PlayerController` component, which round-trips
from the editor's `game.PlayerController` schema (default 200.0).
The system is gated by `With<PlayerController>` (via the query) so
the enemy entity (which carries `EnemyPatrol`) is unaffected.

#### 2. `crates/examples-bevy-harness/src/lib.rs` (+2 lines)

```diff
 pub mod components;
 pub mod loader;
+pub mod movement;

 pub use components::{EditorSpriteAsset, EnemyPatrol, PlayerController, Visible};
 pub use loader::SpawnReport;
+pub use movement::{PlayerMovementPlugin, player_movement_system};
```

#### 3. `crates/examples-bevy-harness/tests/player_movement.rs` (NEW, ~210 lines)

Four `#[ignore]` integration tests, mirroring the G1-step1 pattern:

1. `player_translates_right_on_arrow_right` — ArrowRight pressed, frame
   advances 0.1 s, asserts `translation.x ≈ 20.0`.
2. `player_stays_still_with_no_input` — no input, frame advances, asserts
   `translation.x == 0.0`.
3. `player_translates_left_on_arrow_left` — ArrowLeft pressed, frame
   advances 0.1 s, asserts `translation.x ≈ -20.0`.
4. `enemy_is_unaffected_by_player_input` — ArrowRight pressed, asserts
   `enemy.translation.x == 0.0` (the filter `With<PlayerController>`).

### Bevy 0.19 test-time gotchas resolved

1. **`MinimalPlugins` does not include `InputPlugin`** — Bevy 0.19's
   `MinimalPlugins` is `TaskPoolPlugin + FrameCountPlugin + TimePlugin +
   ScheduleRunnerPlugin`. Keyboard input requires explicitly adding
   `InputPlugin`. The `setup_app` helper does this.

2. **`Time<Real>::last_update` is `None` by default** — the first
   `time_system` run sets it without applying a delta, so the first
   frame's `delta_secs == 0`. The `advance` helper runs two
   `app.update()` calls: the first warms up `Time<Real>`'s context;
   the second applies the configured `ManualDuration`. Subsequent
   tests share the same pattern (always 2 updates per advance).

3. **`TimeUpdateStrategy::ManualDuration(secs)`** — required to control
   `delta_secs` deterministically; the default `Automatic` strategy
   reads `Instant::now()` which is non-deterministic in tests.

## Validation

### Bevy runtime tests

```
$ cd crates/examples-bevy-harness
$ cargo test -- --ignored --nocapture

running 4 tests
test player_stays_still_with_no_input ... ok
test player_translates_right_on_arrow_right ... ok
test player_translates_left_on_arrow_left ... ok
test enemy_is_unaffected_by_player_input ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Cargo tests — no regressions

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

### Workspace check

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
 crates/examples-bevy-harness/src/lib.rs    |  2 +
 crates/examples-bevy-harness/src/movement.rs               | 52 +++++++++++
 crates/examples-bevy-harness/tests/player_movement.rs      | 210 ++++++++++++++
```

(Plus the cycle's `docs/sddk/g1-step2-player-movement/{explore,spec,design}.md`.)

## Public API

- New `pub mod movement` with `pub struct PlayerMovementPlugin` +
  `pub fn player_movement_system`.
- Re-exports from `lib.rs`.
- No changes to existing public types (`components.rs`, `loader.rs`).

## ADR references

- No new ADR needed. BJ-4 is a witness extension to G1-step1, not a
  contract change.

## Risks

None identified:

- Test scaffolding is gated by `#[ignore]` (does not run in default
  `cargo test`; the Bevy 0.19 compile cost is paid only by explicit
  invocation, same pattern as `load_sample.rs`).
- Production code path of `player_movement_system` is plain Bevy —
  no WASM, no editor-model dependency beyond the `PlayerController`
  component mirror.
