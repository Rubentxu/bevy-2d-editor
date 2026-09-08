# G1-step2 — Player Movement System (explore-report)

**Cycle**: `p-28fce7028ac3c497/g1-step2-player-movement`
**Sequence**: 209 (2026-09-08)
**Path**: A-lite
**Phase**: explore

## 1. Origin

`docs/v1.0-stabilization-evidence-map.md` §7-P1 declares the canonical
sample game is the smallest acceptable v1.0 evidence packet. Block
G1-step1 (cycle 184→195, tag v0.109.0) closed 1/4 sub-deliverables: the
**structure** of the sample round-trips through Bevy 0.19 (entities +
components present).

The remaining 3 sub-deliverables tracked in `debt-report.json` from
G1-step1:

- **BJ-3**: UI-creation Playwright test for sample (P2) — out of scope for
  this cycle (requires UI orchestration of asset creation).
- **BJ-4**: Bevy play_mode runtime state assertions (P2) — **this cycle**.
- **BJ-5**: Gameplay assertions (jump, patrol, pickup) (P3) — deferred.

This cycle closes BJ-4 in the minimal form: a single Bevy system
(`player_movement_system`) that moves the player on `ArrowRight` press,
plus one integration test asserting the entity translates.

## 2. Hypothesis

Bevy 0.19 ships `bevy::input::ButtonInput<KeyCode>` and the
`Transform.translation` mutator. A minimal system:

```rust
fn player_movement_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut Transform, With<PlayerController>>,
) {
    let mut dir = 0.0;
    if input.pressed(KeyCode::ArrowRight) { dir += 1.0; }
    if input.pressed(KeyCode::ArrowLeft) { dir -= 1.0; }
    let dt = time.delta_secs();
    for mut t in &mut q {
        t.translation.x += dir * 200.0 * dt;
    }
}
```

…will move the player when ArrowRight is pressed. The integration test:

```rust
let mut app = App::new();
app.add_plugins(MinimalPlugins);
app.add_systems(Update, player_movement_system);
// spawn player entity with PlayerController { speed: 200.0, jump_force: 400.0 }
// press ArrowRight, advance time 0.1s, assert translation.x ≈ 20.0
```

## 3. Investigation

### Bevy 0.19 API check

- `MinimalPlugins` provides `Time` (without rendering window).
- `ButtonInput<KeyCode>` lives in `bevy::input::InputPlugin`.
- The harness already depends on `bevy = "0.19"` so no new dependency.

### Spawn verification

The existing integration test `crates/examples-bevy-harness/tests/load_sample.rs`
demonstrates that `spawn_all_assets` produces 4 entities with `Name`
matching `{Enemy, Ground, Pickup, Player}`. The Player entity carries
`PlayerController { speed: 200.0, jump_force: 400.0 }` from the JSON
schema. Reusing `spawn_all_assets` from a new test is straightforward.

### Why A-lite, not A-full

- New Bevy system: 1 file (extend `lib.rs` or new `movement.rs`).
- 1 new integration test file.
- No public API change (the harness is `pub(crate)` consumers only at
  the moment; only the public surface is the `spawn_all_assets` and
  `SpawnReport` types).
- Architecture: introduces the Bevy system + schedule pattern in this
  crate for the first time — small but architectural.

## 4. Decision

**Scope**: minimum viable Bevy play_mode assertion.

- Add `crates/examples-bevy-harness/src/movement.rs` with
  `PlayerMovementPlugin` and `player_movement_system`.
- Re-export `PlayerMovementPlugin` from `lib.rs`.
- Add `crates/examples-bevy-harness/tests/player_movement.rs` (#[ignore]
  integration test, mirrors the G1-step1 pattern).
- The test:
  1. Loads 4 JSON files via `spawn_all_assets`.
  2. Locates the `Player` entity.
  3. Builds a Bevy `App` with `MinimalPlugins` +
     `PlayerMovementPlugin`.
  4. Presses `ArrowRight`, advances one frame.
  5. Asserts the player's `Transform.translation.x > 0.0`.

**Out of scope**: jump (jump_force), enemy patrol, pickup collision,
contact-death logic. Each is a candidate for a future cycle.

## 5. Validation status

Pending implementation. Expected:

- `cargo test -p examples-bevy-harness --test player_movement` → 1/1
- `cargo check --workspace` → green
- `cargo test -p editor-bevy --lib` → 414/414 (no regressions)

## 6. Open questions

None. The path is mechanical: hook up Bevy's input + transform systems
to a query gated by `PlayerController`.

## 7. Carry-forward reminder

After this cycle:

- BJ-3 (UI-creation Playwright test) — still open
- BJ-5 (gameplay: jump, patrol, pickup) — still open

These can be tackled in subsequent A-min cycles once BJ-4 is closed.
