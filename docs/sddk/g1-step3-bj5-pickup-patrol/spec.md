# G1-step3 — Pickup collision (spec)

**Cycle**: `p-28fce7028ac3c497/g1-step3-bj5-pickup-patrol`
**Sequence**: 224 (2026-09-08)
**Path**: A-lite
**Phase**: specify

## Acceptance criteria

### AC-PC1.1 — Pickup despawns when player overlaps

```
$ cd crates/examples-bevy-harness
$ cargo test --test pickup_collision -- --ignored --nocapture

test pickup_collision::pickup_despawns_when_player_overlaps ... ok
```

**Given**: A Bevy `App` with `MinimalPlugins + InputPlugin +
PlayerMovementPlugin + PickupCollisionPlugin`, sample assets spawned,
player at `(0, 0)` and pickup at `(5, 5)` (manually translated).
**When**: One frame advances 0.1 s.
**Then**: The `Pickup` entity has been despawned (no longer in the
world's entities iterator).

### AC-PC1.2 — Pickup unchanged when player is far

```
test pickup_unchanged_when_player_far ... ok
```

**Given**: Same setup as AC-PC1.1 but player at `(0, 0)` and pickup
at `(1000, 1000)`.
**When**: One frame advances.
**Then**: The `Pickup` entity still exists (not despawned).

### AC-PC1.3 — Collision only affects `Pickup` entities

```
test pickup_collision_only_affects_pickup_marker ... ok
```

**Given**: Sample assets spawned (4 entities: Player, Enemy, Ground,
Pickup). Pickup translated to overlap with player.
**When**: One frame advances.
**Then**: Only the `Pickup` entity is despawned. Player, Enemy,
Ground entities remain.

### AC-PC1.4 — No regressions in player_movement tests

```
$ cargo test -p examples-bevy-harness --test player_movement -- --ignored

test result: ok. 4 passed; 0 failed
```

### AC-PC1.5 — No regressions in editor-bevy lib tests

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored
```

### AC-PC1.6 — Workspace check green

```
$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### AC-PC1.7 — Archcheck green

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Behavior

`pickup_collision_system` runs in `Update`. It:

1. Queries `&Transform, With<PlayerController>` for the player.
2. Queries `Entity, &Transform, With<Pickup>` for every pickup.
3. Computes Manhattan distance: `dx = |player.x - pickup.x|`,
   `dy = |player.y - pickup.y|`.
4. If `dx < 16.0 && dy < 16.0`, issues `commands.entity(e).despawn()`.
5. Pickup entities also have `Visible(false)` semantics handled by
   the editor, but at the Bevy level we despawn (the witness treats
   despawn as "collected").

The hitbox half-extent is `HITBOX_HALF: f32 = 16.0` (constant in the
harness). Generous for a 32×32 sprite; precise enough for tests.

## Public API

- New `pub struct Pickup` (marker component) in `components.rs`.
- New `pub struct PickupCollisionPlugin` (Bevy `Plugin` impl) in
  `collision.rs`.
- New `pub fn pickup_collision_system` in `collision.rs`.
- Re-exports from `lib.rs`.
- No changes to existing public types (`PlayerController`,
  `EnemyPatrol`, `Visible`, `EditorSpriteAsset`,
  `PlayerMovementPlugin`, etc.).

## Out of scope

- Enemy patrol (separate cycle).
- Jump mechanic (no system yet; field exists in `PlayerController`).
- Contact-death logic graph runtime.
- UI-creation Playwright test (BJ-3).
