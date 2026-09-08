# G1-step3 handoff — Pickup collision (BJ-5 partial)

**Cycle**: `p-28fce7028ac3c497/g1-step3-bj5-pickup-patrol`
**Tag**: v0.109.3 (commit `c46effc`)
**Closed at**: 2026-09-08, sequence 223→236

## TL;DR

A-lite cycle closing **BJ-5 partial** — the pickup-collision mechanic
for the v1.0 canonical sample. The harness crate gains:

- A `Pickup` marker component (heuristic-attached to entities named
  `"Pickup"` by the loader).
- A `PickupCollisionPlugin` + `pickup_collision_system` that despawns
  pickups within a 16×16 hitbox of the player.
- Three integration tests proving the despawn, the far-away preservation,
  and the marker-only filter.

5 files / +264 / -3. 8 Bevy runtime tests now pass deterministically
(3 new + 4 player_movement + 1 load_sample).

## What shipped

### Code (single commit `c46effc`)

- `crates/examples-bevy-harness/src/components.rs` (+11): `Pickup`
  marker.
- `crates/examples-bevy-harness/src/loader.rs` (+9 / -2): attach
  `Pickup` to entities whose `Name == "Pickup"`.
- `crates/examples-bevy-harness/src/collision.rs` (NEW, 55):
  `PickupCollisionPlugin` + `pickup_collision_system` +
  `PICKUP_HITBOX_HALF` constant.
- `crates/examples-bevy-harness/src/lib.rs` (+3 / -1): module
  declaration and re-exports.
- `crates/examples-bevy-harness/tests/pickup_collision.rs` (NEW,
  186): 3 `#[ignore]` integration tests using the same
  `MinimalPlugins + InputPlugin + ManualDuration` helpers as
  G1-step2's `player_movement.rs`.

## Cycle flow

| Phase | Sequence | Status |
|-------|----------|--------|
| cycle.start | 223 | OPEN/explore |
| phase.explore.complete | 224 | OPEN/specify |
| phase.specify.complete | 226 | OPEN/design |
| phase.design.complete.a-lite | 228 | OPEN/build |
| phase.build.complete | 230 | OPEN/verify |
| phase.verify.complete.a-lite | 232 | RELEASE_PENDING |
| release.complete | 234 | RELEASED |
| archive.complete | 236 | CLOSED |

## Validation evidence

```
$ cd crates/examples-bevy-harness
$ cargo test -- --ignored

running 3 tests (pickup_collision)
test pickup_despawns_when_player_overlaps ... ok
test pickup_unchanged_when_player_far ... ok
test pickup_collision_only_affects_pickup_marker ... ok
test result: ok. 3 passed; 0 failed

(plus 4 player_movement + 1 load_sample = 8 Bevy runtime tests)
```

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Lessons learned

1. **Loader-level heuristic for marker components** is a clean
   way to attach runtime-only Bevy components when the editor sample
   has no schema. The `editor.Name == "Pickup"` check inside
   `SpawnFromDoc::spawn_into` is the discriminator.

2. **`World::get_entity` returns `Result<EntityRef<'_>,
   EntityNotSpawnedError>`, not `Option`**. Use `.is_ok()` not
   `.is_some()`. This differs from some older Bevy patterns.

## Carry-forward

- BJ-3 (UI-creation Playwright test, P2) — independent cycle.
- BJ-5 remaining:
  - **enemy patrol** — needs schema extension (`direction` field) or
    a per-entity `PatrolState` resource.
  - **jump** — needs a `KeyCode::Space` listener + gravity system;
    `PlayerController.jump_force` already in schema.
  - **contact-death logic graph runtime** — needs Bevy execution of
    the `LogicGraphAsset` (declarative → imperative bridge).

## Reference

- Spec: `docs/sddk/g1-step3-bj5-pickup-patrol/spec.md`
- Implementation receipt: `docs/sddk/g1-step3-bj5-pickup-patrol/implementation-receipt.md`
- Verify report: `docs/sddk/g1-step3-bj5-pickup-patrol/verify-report.md`
- Debt report: `docs/sddk/g1-step3-bj5-pickup-patrol/debt-report.json`
- Release receipt: `docs/sddk/g1-step3-bj5-pickup-patrol/release-receipt.md`
- Merge receipt: `docs/sddk/g1-step3-bj5-pickup-patrol/merge-receipt.md`
- Archive manifest: `docs/sddk/g1-step3-bj5-pickup-patrol/archive-manifest.md`
