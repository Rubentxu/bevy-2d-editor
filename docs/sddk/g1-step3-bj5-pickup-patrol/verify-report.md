# G1-step3 — Pickup collision (verify-report)

**Cycle**: `p-28fce7028ac3c497/g1-step3-bj5-pickup-patrol`
**Sequence**: 230 (2026-09-08)
**Path**: A-lite
**Phase**: verify

## Acceptance criteria — verification

### AC-PC1.1 — Pickup despawns when player overlaps ✅

```
$ cd crates/examples-bevy-harness
$ cargo test --test pickup_collision pickup_despawns -- --ignored --nocapture

test pickup_despawns_when_player_overlaps ... ok
```

**PASS.** Player at (0,0), pickup at (5,5), 0.1s advance. Pickup
entity despawned.

### AC-PC1.2 — Pickup unchanged when player far ✅

```
test pickup_unchanged_when_player_far ... ok
```

**PASS.** Player at (0,0), pickup at (1000,1000). Pickup remains.

### AC-PC1.3 — Collision only affects `Pickup` entities ✅

```
test pickup_collision_only_affects_pickup_marker ... ok
```

**PASS.** Overlap scenario; player, enemy, ground all survive; only
pickup despawned.

### AC-PC1.4 — No regressions in player_movement tests ✅

```
$ cargo test -p examples-bevy-harness --test player_movement -- --ignored

test result: ok. 4 passed; 0 failed
```

### AC-PC1.5 — No regressions in editor-bevy lib tests ✅

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

### AC-PC1.6 — Workspace check green ✅

```
$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### AC-PC1.7 — Archcheck green ✅

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Side checks

### Full Bevy harness test suite

```
$ cd crates/examples-bevy-harness
$ cargo test -- --ignored --nocapture

running 4 tests (player_movement)
test result: ok. 4 passed; 0 failed

running 3 tests (pickup_collision)
test result: ok. 3 passed; 0 failed

running 1 test (load_sample)
test result: ok. 1 passed; 0 failed
```

Total: 8 Bevy runtime tests pass deterministically under `--ignored`.

## Risks identified

None. The collision system is small, isolated, and its invariants
are verifiable from the editor-side metadata (entity `Name == "Pickup"`
is the documented trigger).

## Summary

**PASS** — 7/7 acceptance criteria met. Ready to advance to release.
