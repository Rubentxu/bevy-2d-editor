# G1-step5 — Enemy patrol (release-receipt)

**Cycle**: `p-28fce7028ac3c497/g1-step5-bj5-enemy-patrol`
**Sequence**: 260 (2026-09-08)
**Tag**: v0.109.5
**Phase**: release

## Release summary

| Item | Value |
|------|-------|
| Cycle name | g1-step5-bj5-enemy-patrol |
| Path | A-lite |
| Phase at release | verify → release transition |
| Tag | v0.109.5 |
| Code-commit SHA | `7006dc7` |
| Trunk SHA (HEAD) | `7006dc7` |
| Origin/main SHA | `7006dc7` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |
| Diff vs. v0.109.4 | +419 / -1 across 5 files |

## Files in code release

```
crates/examples-bevy-harness/src/components.rs        +18 / -0 lines
crates/examples-bevy-harness/src/enemy_patrol.rs      NEW (71 lines)
crates/examples-bevy-harness/src/lib.rs               +3 / -1 lines
crates/examples-bevy-harness/src/loader.rs            +7 / -1 lines
crates/examples-bevy-harness/tests/enemy_patrol.rs    NEW (224 lines)
```

(plus 6 SDDK artifact docs in `docs/sddk/g1-step5-bj5-enemy-patrol/`.)

## Test posture

```
$ cd crates/examples-bevy-harness
$ cargo test -- --ignored

running 4 tests (enemy_patrol)
test enemy_patrols_right_initially ... ok
test enemy_reverses_at_left_boundary ... ok
test enemy_reverses_at_boundary ... ok
test enemy_unaffected_by_player_movement ... ok
test result: ok. 4 passed; 0 failed

(plus 3 jump + 3 pickup_collision + 4 player_movement + 1 load_sample
 = 15 Bevy runtime tests)
```

Net change vs. v0.109.4: +4 Bevy gameplay tests.

## Workspace

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

```
$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Static analysis

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Public API

- New `pub struct EnemyDirection(pub f32)` (component).
- New `pub struct EnemyPatrolPlugin` (Bevy 0.19 `Plugin` impl).
- New `pub fn enemy_patrol_system`.

No changes to existing public types.

## Cycle closure

| Phase | Sequence |
|-------|----------|
| cycle.start | 251 |
| phase.explore.complete | 252 |
| phase.specify.complete | 254 |
| phase.design.complete.a-lite | 256 |
| phase.build.complete | 258 |
| phase.verify.complete.a-lite | 260 |

## Out-of-scope carried forward

- BJ-3 (UI-creation Playwright test, P2) — still open.
- BJ-5 remaining: contact-death logic-graph runtime (declarative →
  imperative bridge needed).

## Lessons

1. **Per-entity state via component**: For multi-instance state
   (each enemy has its own direction), a component is the right
   shape. A `Local<>` resource would force global state shared
   across all enemies — wrong semantics.

2. **Manual positioning in boundary tests**: Bevy 0.19's
   `TimeUpdateStrategy::ManualDuration` is one-shot — multi-frame
   test advances need to re-set the strategy before each `update()`
   AND the time warm-up only applies the strategy on the second
   update. To bypass this complexity for boundary tests, manually
   position the entity at `patrol_range + 1` and let the system
   flip its direction in a single frame. This trades time-control
   fidelity for test simplicity.

3. **Reflective patrol is sufficient for a witness**: A real game
   would use waypoint patrol with explicit patrol points. For a
   harness that proves the JSON round-trips, reflective boundary
   is enough.
